use parity_tokio_ipc::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;
use uds_client::{UdsClientOptions, get};

#[actix_rt::test]
async fn test_uds_client() {
    let path = "./hanode.sock";
    let result = get(&UdsClientOptions{
        uds_sock_path: path,
        url_path: "/peers"
    }).await.expect("get failed");
    println!("{:?}", result);
    // let mut client = Endpoint::connect(&path).await
	// 	.expect("Failed to connect client.");
    // client.write_all(b"\
    //     GET /peers HTTP/1.1\r\n\
    //     Host: localhost\r\n\
    //     User-Agent: client/0.0.1\r\n\
    //     Accept: */*\r\n\
    //     \r\n").await.expect("Unable to write message to client");

    // let mut buf = BytesMut::with_capacity(10);
    // let mut buf_size = 0;
    // loop {
    //     client.read_buf(&mut buf).await.expect("Unable to read message from client");
    //     println!("{:?} {:?}", buf_size, buf.len());
    //     if let Ok(r) = std::str::from_utf8(&buf[..]) {
    //         println!("------\n{}\n------", r);
    //     } else {
    //         assert!(false);
    //     }
    //     if buf.len() == buf_size {
    //         break;
    //     }
    //     buf_size = buf.len();
    // }
}
