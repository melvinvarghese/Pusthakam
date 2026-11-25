//! Console host for a persistent, operator-selected library directory.
use pusthakam_lib::{
    db,
    error::{AppError, AppResult},
    logging,
    paths::AppPaths,
    services::{backup, lan, lan_runtime, settings, staff_web},
    state::AppState,
};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

struct Options {
    data: PathBuf,
    bind: String,
    port: u16,
    public: Option<String>,
    full_staff: bool,
}
fn options(args: impl Iterator<Item = String>) -> AppResult<Option<Options>> {
    let mut args = args;
    let mut data = None;
    let mut bind = "127.0.0.1".to_owned();
    let mut port = 4318;
    let mut public = None;
    let mut full_staff = false;
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(None);
        }
        if arg == "--enable-full-staff" {
            full_staff = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| AppError::Validation(format!("Missing value for {arg}.")))?;
        match arg.as_str() {
            "--data-dir" => data = Some(PathBuf::from(value)),
            "--bind" => {
                value.parse::<std::net::IpAddr>().map_err(|_| {
                    AppError::Validation("Bind address must be an IP address.".into())
                })?;
                bind = value;
            }
            "--port" => {
                port = value.parse().map_err(|_| {
                    AppError::Validation("Port must be between 1024 and 65535.".into())
                })?
            }
            "--public-url" => public = Some(staff_web::normalize_public_url(&value)?),
            _ => return Err(AppError::Validation(format!("Unknown option {arg}."))),
        }
    }
    let data = data
        .ok_or_else(|| AppError::Validation("Choose a persistent absolute --data-dir.".into()))?;
    if !data.is_absolute() || port < 1024 {
        return Err(AppError::Validation(
            "Use an absolute data directory and a port between 1024 and 65535.".into(),
        ));
    }
    Ok(Some(Options {
        data,
        bind,
        port,
        public,
        full_staff,
    }))
}
fn run(options: Options) -> AppResult<()> {
    let paths = AppPaths::ensure(&options.data)?;
    let guard = logging::init(&paths.logs);
    let (lock, exclusive) = backup::startup_lock(&paths)?;
    let mut conn = db::open_with_pre_upgrade(&paths.database, |source| {
        backup::create(source, &paths.backups)?;
        Ok(())
    })?;
    if exclusive {
        backup::apply_pending_restore(&mut conn, &paths)?;
    }
    let _library_lock = backup::hold_library_lock(lock, exclusive)?;
    lan::get_or_init_config(&conn)?;
    // Local console arguments are operator configuration, never a remote setup bypass.
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE lan_config SET enabled=1, bind_address=?1, port=?2, opac_enabled=1, browser_staff_enabled=1",rusqlite::params![options.bind,options.port])?;
    if let Some(url) = options.public {
        settings::set(&tx, "web.public_url", &url)?;
    }
    if options.full_staff {
        settings::set(&tx, "web.full_staff_enabled", "true")?;
    }
    tx.commit()?;
    let state = AppState {
        paths,
        db: Arc::new(Mutex::new(conn)),
        lan: Arc::new(Mutex::new(Default::default())),
        session: Arc::new(Mutex::new(None)),
        pending_import: Arc::new(Mutex::new(None)),
        _log_guard: Arc::new(guard),
    };
    let config = lan_runtime::config(&state)?;
    if let Some(error) = config.listener_error {
        return Err(AppError::Internal(error));
    }
    println!(
        "Pusthakam server listening on {}:{}",
        config.bind_address, config.port
    );
    println!("OPAC: /opac   Staff: /staff");
    println!("For first setup, open /staff on this server using 127.0.0.1. Type quit to stop.");
    let stop = Arc::new(AtomicBool::new(false));
    let input_stop = Arc::clone(&stop);
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if line.trim().eq_ignore_ascii_case("quit") {
                        input_stop.store(true, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }
    });
    let mut last_backup = Instant::now() - Duration::from_secs(60);
    while !stop.load(Ordering::SeqCst) {
        if last_backup.elapsed() >= Duration::from_secs(60) {
            if let Err(error) = state.with_conn(|conn| backup::run_daily(conn, &state.paths)) {
                tracing::warn!(%error,"Scheduled backup failed");
            }
            last_backup = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let listener = state
        .lan
        .lock()
        .map_err(|_| AppError::Internal("Host runtime lock poisoned".into()))?
        .take_listener();
    if let Some(mut listener) = listener {
        listener.drain();
    }
    drop(state);
    Ok(())
}
fn main() {
    let result=options(std::env::args().skip(1)).and_then(|options|match options {
        Some(options)=>run(options),
        None=>{println!("pusthakam-server --data-dir <absolute persistent folder> [--bind 127.0.0.1] [--port 4318] [--public-url https://library.example]\nHTTPS requires a separately configured reverse proxy.");Ok(())}
    });
    if let Err(error) = result {
        eprintln!("Server could not run: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> AppResult<Option<Options>> {
        options(args.iter().map(|s| s.to_string()))
    }
    #[test]
    fn hosting_requires_persistent_directory_and_valid_origin() {
        let dir = std::env::temp_dir();
        let absolute = dir.to_str().unwrap();
        let parsed = parse(&["--data-dir", absolute]).unwrap().unwrap();
        assert_eq!(parsed.bind, "127.0.0.1");
        assert!(!parsed.full_staff);
        assert!(parse(&[]).is_err());
        assert!(parse(&["--data-dir", "relative"]).is_err());
        assert!(parse(&["--data-dir", absolute, "--public-url", "http://example.com"]).is_err());
        assert!(parse(&["--data-dir", absolute, "--port", "80"]).is_err());
    }
}
