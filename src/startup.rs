use p2p::node::{Sender, Receiver};
use p2p::{node::NodeBehaviour, message::Message};
use p2p::message;
use server;
use signal_hook::consts::SIGINT;
use std::io::Error;
use std::fs::{File};
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
use tokio;
use futures::executor::block_on;

use crate::utils;

pub struct StartOptions {
    pub port: u16,
    pub daemon: bool,
    pub pid: String
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
                println!("Success, daemonized")
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
            println!("Received signal {:?}", sig);
        }
    }));

    // Spawn the root task
    rt.block_on(async {
        async fn start_node () {
            match run().await {
                Ok(_) => (),
                Err(e) => eprintln!("Error, {}", e),
            }
        }
        futures::join!(start_node())
    });
    Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (mut sender, mut receiver) = mpsc::unbounded::<Message>();
    async fn run<'a>(receiver: &mut Receiver<Message>) -> Result<(), Box<dyn std::error::Error>> {
        let mut node = p2p::node::Node::new(receiver).await?;
        match node.start().await {
            Ok(_ok) => print!("Success"),
            Err(err) => print!("Error: {}", err)
        };
        Ok(())
    }
    async fn input(sender: &mut Sender<Message>) -> Result<(), Box<dyn std::error::Error>>  {
        // Read full lines from stdin
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        loop {
            select! {
                line = stdin.select_next_some() => 
                    sender.send(message::Message::from(line.expect("Stdin not to close").to_string()))
                    .await?,
            }
        }
    }
    let _ = futures::join!(
        run(&mut receiver),
        input(&mut sender),
        server::core::start_server(server::core::ServerOptions {
            port: 8080, 
            host: Some("localhost".to_string())
        }
    ));
    Ok(())
}
