use uds_client::{UdsClientOptions, get};

#[actix_rt::test]
#[ignore]
async fn test_uds_client() {
    let path = "./hanode.sock";
    let result = get(&UdsClientOptions{
        uds_sock_path: path.to_string(),
        url_path: "/boardcast/hello".to_string(),
    }).await.expect("get failed");
    println!("{:?}", result);
}
