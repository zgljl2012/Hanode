use log::{info, debug, warn, error};
use p2p::lifecycle::NodeLifecycleHooks;
use p2p::node::{Sender, Receiver, NodeBehaviourOptions};
use p2p::{node::NodeBehaviour, message::Message};
use p2p::message;

use signal_hook::consts::SIGINT;
use std::io::Error;
use std::fs::{File};
use std::sync::{Arc, RwLock};
use std::{thread, process};
use signal_hook::{iterator::Signals};
use std::io::ErrorKind;
use async_std::{io};
use futures::{
    prelude::{stream::StreamExt, *},
    select,
};
use futures::channel::mpsc;
use daemonize::Daemonize;

use futures::executor::block_on;

use crate::utils;

pub struct StartOptions {
    pub port: u16,
    pub daemon: bool,
    pub pid: String,
    pub host: String,
    pub bootnode: Option<String>
}

#[derive(Debug, Clone)]
struct NodeLifecycleGlobal {
}

impl NodeLifecycleGlobal {
    pub fn new() -> Self {
        NodeLifecycleGlobal {
        }
    }
}

impl NodeLifecycleHooks for NodeLifecycleGlobal {
    fn on_stopped(&self) {
        debug!("NodeLifecycleHooks on_stopped()");
        // Shutdown when the node is stopped
        process::exit(0);
    }
}

pub async fn start(options: &StartOptions) -> Result<(), Box<dyn std::error::Error>> {
    if options.daemon {
        if utils::exists(&options.pid) {
            let err = Error::new(
                ErrorKind::Other,
                format!("pid file already exists: {}", &options.pid)
            );
            return Err(Box::new(err));
        }
        let stdout = File::create("./daemon.out").unwrap();
        let stderr = File::create("./daemon.err").unwrap();

        let daemonize = Daemonize::new()
            .pid_file(options.pid.clone()) // Every method except `new` and `start`
            .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
            .stderr(stderr)  // Redirect stderr to `/tmp/daemon.err`.
            .exit_action(|| {
                println!("Executed before master process exits");
            })
            .privileged_action(|| "Executed before drop privileges");

        match daemonize.start() {
            Ok(_) => {
                info!("Success, daemonized")
            },
            Err(e) => eprintln!("Error, {}", e),
        }
    }
    // Create the runtime
    let rt = tokio::runtime::Runtime::new()?;

    // Signals handlers
    let mut signals = Signals::new(&[SIGINT])?;
    thread::spawn(move || block_on(async {
        for sig in signals.forever() {
            // Exit program
            if sig == SIGINT {
                process::exit(1);
            }
            warn!("Received signal {:?}", sig);
        }
    }));

    // Spawn the root task
    rt.block_on(async {
        // Create sender and receiver for message processing
        let (sender, mut receiver) = mpsc::unbounded::<Message>();
        // Start node
        async fn start_node (receiver: &mut Receiver<Message>, options: &StartOptions) -> Result<(), Box<dyn std::error::Error>> {
            // Node lifecycle hooks
            let lifecycle = Box::new(NodeLifecycleGlobal::new());

            // Create node
            let mut node = p2p::node::Node::new(receiver, lifecycle, NodeBehaviourOptions{
                bootnode: options.bootnode.clone()
            }).await?;
            // Start node
            futures::join!(async {
                match node.start().await {
                    Ok(_ok) => info!("Success"),
                    Err(err) => error!("Error: {}", err)
                };
            });
            Ok(())
        }
        // Input message
        async fn input(sender: Arc<RwLock<Sender<Message>>>, options: &StartOptions) -> Result<(), Box<dyn std::error::Error>>  {
            // If running in the background, return immediately
            if options.daemon {
                return Ok(());
            }
            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
            loop {
                select! {
                    line = stdin.select_next_some() => {
                        (*sender.write().unwrap()).send(message::Message::from(line.expect("Stdin not to close").to_string()))
                        .await?
                    },
                }
            }
        }
        // Start server
        async fn start_server(proxy_sender: Arc<RwLock<Sender<Message>>>, options: &StartOptions) -> Result<(), Box<dyn std::error::Error>> {
            server::core::start_server(proxy_sender, server::core::ServerOptions {
                port: options.port,
                host: Some(options.host.to_string())
            }).await
        }
        let ps = Arc::new(RwLock::new(sender));
        let _ = futures::join!(start_node(&mut receiver, options), input(Arc::clone(&ps), options), start_server(Arc::clone(&ps), options));
    });
    Ok(())
}


pub struct StopOptions{
    pub host: String,
    pub port: u16,
}

pub async fn stop(opts: StopOptions) -> Result<(), Box<dyn std::error::Error>>  {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on( async {
        let request_url = format!("http://{}:{}/stop", opts.host, opts.port);
        debug!("Send stop command to the node: {}", request_url);
        let _ = reqwest::get(&request_url).await;
    });
    Ok(())
}
