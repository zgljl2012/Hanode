use std::{error::Error, sync::{Mutex, RwLock, Arc}};
use actix_web::{get, web::{self, Data}, App, HttpServer, Responder};
use p2p::message::Message;
use p2p::node::Sender;
use futures::SinkExt;

struct AppState {
    counter: Mutex<i32>,
    proxy_sender: Arc<RwLock<Sender<Message>>>
}

#[get("/hello/{name}")]
async fn greet(state: Data<AppState>, name: web::Path<String>) -> impl Responder {
    let mut counter = state.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard
    let _ = (*state.proxy_sender.write().unwrap()).send(Message::from(name.to_string())).await;
    format!("Hello {name} {counter}!")
}

#[get("/stop")]
async fn stop_p2p_node(state: Data<AppState>) -> impl Responder {
    match (*state.proxy_sender.write().unwrap()).send(Message::stop_message()).await {
        Ok(_) => {
            println!("Stopped p2p node");
        },
        Err(err) => {
            println!("Failed to stop p2p node: {:?}", err);
        }
    }
    format!("stop")
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
            .service(stop_p2p_node)
    })
    .bind((host, port))?
    .run()
    .await?;
    Ok(())
}
