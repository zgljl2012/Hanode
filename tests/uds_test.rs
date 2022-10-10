use uds_client::{UdsClientOptions, get};

#[actix_rt::test]
#[ignore]
async fn test_uds_client() {
    let path = "./hanode.sock";
    let result = get(&UdsClientOptions{
        uds_sock_path: path,
        url_path: "/boardcast/hello"
    }).await.expect("get failed");
    println!("{:?}", result);
}
