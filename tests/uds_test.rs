use uds_client::{UdsClientOptions, get};

#[actix_rt::test]
async fn test_uds_client() {
    let path = "./hanode.sock";
    let result = get(&UdsClientOptions{
        uds_sock_path: path,
        url_path: "/peers"
    }).await.expect("get failed");
    println!("{:?}", result);
}
