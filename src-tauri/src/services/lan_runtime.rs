//! Owns the listener for the desktop application's lifetime.
use crate::{
    error::AppResult,
    services::lan::{self, LanConfig, LanServerHandle},
};
use std::path::Path;

#[derive(Default)]
pub struct LanRuntime {
    listener: Option<(String, u16, LanServerHandle)>,
}
impl LanRuntime {
    pub fn reconcile(&mut self, database: &Path, config: &mut LanConfig) {
        let matches = self
            .listener
            .as_ref()
            .is_some_and(|(bind, port, _)| bind == &config.bind_address && *port == config.port);
        if !config.enabled || !matches {
            self.listener.take();
        }
        config.listener_error = None;
        if config.enabled && self.listener.is_none() {
            match lan::start_lan_listener(database.to_path_buf(), &config.bind_address, config.port)
            {
                Ok(handle) => {
                    self.listener = Some((config.bind_address.clone(), config.port, handle))
                }
                Err(error) => config.listener_error = Some(error.to_string()),
            }
        }
        config.listener_running = self.listener.is_some();
    }
}

pub fn config(state: &crate::state::AppState) -> AppResult<LanConfig> {
    let mut runtime = state
        .lan
        .lock()
        .map_err(|_| crate::error::AppError::Internal("LAN runtime lock poisoned".into()))?;
    let mut config = state.with_conn(lan::get_or_init_config)?;
    runtime.reconcile(&state.paths.database, &mut config);
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{TcpListener, TcpStream};
    #[test]
    fn owns_listener_until_disabled_and_reports_bind_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.db");
        let conn = crate::db::open(&path).unwrap();
        let mut config = lan::get_or_init_config(&conn).unwrap();
        config.enabled = true;
        config.bind_address = "127.0.0.1".into();
        config.port = 0;
        let mut runtime = LanRuntime::default();
        runtime.reconcile(&path, &mut config);
        assert!(config.listener_running);
        let first = runtime.listener.as_ref().unwrap().2.port;
        assert!(TcpStream::connect(("127.0.0.1", first)).is_ok());
        runtime.reconcile(&path, &mut config);
        assert_eq!(first, runtime.listener.as_ref().unwrap().2.port);
        config.enabled = false;
        runtime.reconcile(&path, &mut config);
        assert!(!config.listener_running);
        assert!(TcpListener::bind(("127.0.0.1", first)).is_ok());
        let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
        config.enabled = true;
        config.port = occupied.local_addr().unwrap().port();
        runtime.reconcile(&path, &mut config);
        assert!(!config.listener_running);
        assert!(config.listener_error.is_some());
        drop(occupied);
        runtime.reconcile(&path, &mut config);
        assert!(config.listener_running);
        assert!(config.listener_error.is_none());
        config.port = 0;
        runtime.reconcile(&path, &mut config);
        assert!(config.listener_running);
        assert_ne!(first, runtime.listener.as_ref().unwrap().2.port);
    }
}
