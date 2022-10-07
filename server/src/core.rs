use std::{error::Error, sync::{Mutex, RwLock, Arc}};
use actix_web::{get, web::{self, Data}, App, HttpServer, Responder};
use p2p::message::Message;
use p2p::node::Sender;

struct AppState {
    counter: Mutex<i32>,
    proxy_sender: Arc<RwLock<Sender<Message>>>
}

#[get("/hello/{name}")]
async fn greet(state: Data<AppState>, name: web::Path<String>) -> impl Responder {
    let mut counter = state.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard
    format!("Hello {name} {counter}!")
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub host: Option<String>,
    pub port: u16,
}

pub async fn start_server(proxy_sender: Arc<RwLock<Sender<Message>>>, opts: ServerOptions) -> Result<(), Box<dyn Error>> {
    let host = match opts.host {
        Some(host) => host,
        None => "127.0.0.1".to_string(),
    };
    let port = opts.port.clone();
    let state = Data::new(AppState {
        counter: Mutex::new(0),
        proxy_sender,
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(greet)
    })
    .bind((host, port))?
    .run()
    .await?;
    Ok(())
}
