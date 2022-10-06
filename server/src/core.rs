use std::error::Error;
use actix_web::{get, web, App, HttpServer, Responder};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

pub struct ServerOptions {
    pub host: Option<String>,
    pub port: u16,
}

pub async fn start_server(opts: ServerOptions) -> Result<(), Box<dyn Error>> {
    let host = match opts.host {
        Some(host) => host,
        None => "127.0.0.1".to_string(),
    };
    HttpServer::new(|| {
        App::new().service(greet)
    })
    .bind((host, opts.port))?
    .run()
    .await?;
    Ok(())
}
