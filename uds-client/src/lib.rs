use parity_tokio_ipc::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;

pub struct UdsClientOptions {
    pub uds_sock_path: &'static str,
    pub url_path: &'static str,
}

/// Only support GET requests now
pub async fn get(opts: &UdsClientOptions) -> Result<String, Box<dyn std::error::Error>> {
    let mut client = Endpoint::connect(&opts.uds_sock_path).await
		.expect("Failed to connect client.");
    let message = format!("\
        GET {} HTTP/1.1\r\n\
        Host: localhost\r\n\
        User-Agent: client/0.0.1\r\n\
        Accept: */*\r\n\
        \r\n", opts.url_path);
    client.write_all(message.as_bytes()).await.expect("Unable to write message to client");

    let mut buf = BytesMut::with_capacity(256);
    let mut buf_size = 0;
    loop {
        client.read_buf(&mut buf).await.expect("Unable to read message from client");
        if buf.len() == buf_size {
            break;
        }
        buf_size = buf.len();
    }
    if let Ok(r) = std::str::from_utf8(&buf[..]) {
        return Ok(String::from(r));
    }
    let err = std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Read result from buffer failed")
    );
    return Err(Box::new(err));
}
