//! Optional real HTTP transport for the existing command contract/workflow suite.
use pusthakam_lib::{
    services::{lan, lan_runtime, settings},
    state::AppState,
};
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::Duration,
};

pub struct StaffClient {
    port: u16,
    credentials: Mutex<(String, String)>,
}
impl StaffClient {
    pub fn start(state: &AppState) -> Self {
        let reserved = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = reserved.local_addr().unwrap().port();
        drop(reserved);
        state.with_conn(|conn| {
            lan::get_or_init_config(conn)?;
            conn.execute("UPDATE lan_config SET enabled=1, browser_staff_enabled=1, bind_address='127.0.0.1', port=?1",[port])?;
            settings::set(conn,"web.full_staff_enabled","true")
        }).unwrap();
        let config = lan_runtime::config(state).unwrap();
        assert!(config.listener_error.is_none());
        let response = request(port, "GET", "/staff", "", "");
        let cookie = header(&response, "set-cookie")
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let csrf = response
            .split("name=\"pusthakam-staff-csrf\" content=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_owned();
        Self {
            port,
            credentials: Mutex::new((cookie, csrf)),
        }
    }
    pub fn call(&self, command: &str, args: Value) -> Result<Value, Value> {
        let mut credentials = self.credentials.lock().unwrap();
        let response=request(self.port,"POST","/api/staff/commands",&format!("Cookie: {}\r\nX-CSRF-Token: {}\r\nContent-Type: application/json\r\nOrigin: http://127.0.0.1:{}\r\n",credentials.0,credentials.1,self.port),&json!({"command":command,"args":args}).to_string());
        if let Some(cookie) = header(&response, "set-cookie") {
            credentials.0 = cookie.split(';').next().unwrap().to_owned();
        }
        if let Some(csrf) = header(&response, "x-csrf-token") {
            credentials.1 = csrf.to_owned();
        }
        let body: Value = serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1)
            .unwrap_or_else(|_| panic!("{command}: {response}"));
        if let Some(error) = body.get("error") {
            Err(error.clone())
        } else {
            Ok(body["result"].clone())
        }
    }
}
fn header<'a>(response: &'a str, name: &str) -> Option<&'a str> {
    response.split("\r\n").find_map(|line| {
        line.split_once(':')
            .filter(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim())
    })
}
fn request(port: u16, method: &str, path: &str, headers: &str, body: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();
    write!(stream,"{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Length: {}\r\n{headers}\r\n{body}",body.len()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}
