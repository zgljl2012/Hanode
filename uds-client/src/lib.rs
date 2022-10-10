pub mod utils;
use parity_tokio_ipc::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;

pub struct UdsClientOptions {
    pub uds_sock_path: String,
    pub url_path: String,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub content_length: usize,
    pub body: String,
    pub date: String,
}

/// Only support GET requests now
pub async fn get(opts: &UdsClientOptions) -> Result<Response, Box<dyn std::error::Error>> {
    let mut client = Endpoint::connect(&opts.uds_sock_path).await
		.expect("Failed to connect client.");
    let message = format!("\
        GET {} HTTP/1.1\r\n\
        Host: localhost\r\n\
        User-Agent: client/0.0.1\r\n\
        Accept: */*\r\n\
        \r\n", opts.url_path);
    client.write_all(message.as_bytes()).await.expect("Unable to write message to client");

    let chunk_size = 256;
    let mut buf = BytesMut::with_capacity(chunk_size);
    loop {
        let n = client.read_buf(&mut buf).await.expect("Unable to read message from client");
        if n == 0 || buf.len() < buf.capacity() {
            break;
        }
    }
    if let Ok(r) = std::str::from_utf8(&buf[..]) {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut response = httparse::Response::new(&mut headers);
        let res = response.parse(r.as_bytes());
        if res.is_err() {
            return Err(format!("Failed to parse response: {:?}", res).into());
        }
        let status = response.code;
        if status != Some(200) {
            return Err(format!("Invalid status code: {:?}", status).into());
        }
        // parse body
        let body_offset = res.unwrap().unwrap();
        let body = std::str::from_utf8(&r.as_bytes()[body_offset..]);
        // parse headers
        let mut content_length = 0;
        let mut content_type = String::new();
        let mut date = String::new();
        for i in 0..response.headers.len() {
            let name = response.headers[i].name.to_lowercase();
            let value = std::str::from_utf8(response.headers[i].value).unwrap();
            if name == "content-length" {
                content_length = value.parse::<i32>().unwrap();
            } else if name == "content-type" {
                content_type = value.to_string();
            } else if name == "date" {
                date = value.to_string();
            }
        }
        let result = Response {
            status: response.code.unwrap(),
            content_type: content_type,
            content_length: content_length as usize,
            body: body.unwrap().to_string(),
            date: date,
        };
        return Ok(result);
    }
    let err = std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Read result from buffer failed")
    );
    return Err(Box::new(err));
}
