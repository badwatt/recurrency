//! Local HTTP servers for API client tests.

use std::io::{Read, Write};
use std::net::TcpListener;

pub(crate) fn serve_once(
    status: &str,
    body: impl Into<String>,
) -> (String, std::thread::JoinHandle<()>) {
    let body = body.into();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let status = status.to_string();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept test request");
        let mut request = [0u8; 4096];
        let _ = stream.read(&mut request);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    });
    (format!("http://{address}"), handle)
}

pub(crate) async fn with_server<T, F>(body: impl Into<String>, call: F) -> T
where
    F: std::future::Future<Output = T>,
{
    with_response("200 OK", body, call).await
}

pub(crate) async fn with_response<T, F>(status: &str, body: impl Into<String>, call: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let (base, server) = serve_once(status, body);
    crate::api::client::set_base_url(&base);
    let result = call.await;
    crate::api::client::clear_base_url();
    server.join().expect("test server");
    result
}
