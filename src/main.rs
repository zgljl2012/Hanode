
#[async_std::main]
async fn main() {
    match p2p::p2p::start().await {
        Ok(_ok) => print!("Success"),
        Err(err) => print!("Error: {}", err)
    }
}
