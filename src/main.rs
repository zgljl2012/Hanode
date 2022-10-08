
use std::error::Error;

use clap::{arg, Command};
use env_logger::{Builder, Target};
use log::{error, debug};
mod startup;
mod utils;

fn cli() -> Command {
    Command::new("hanode")
        .about("A server for manage node")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
               .about("Start a node")
               .arg(arg!(-d - -daemon "Running in daemon mode"))
               .arg(arg!(--bootnode <BOOTNODE> "Specify a boot node to connect").required(false))
        )
        .subcommand(
            Command::new("stop")
               .about("Stop a node")
               .arg(arg!(-p - -port <PORT> "Specify a port to connect to").value_parser(clap::value_parser!(u16).range(3000..)).required(false))
               .arg(arg!(-H - -host <HOST> "Specify a host to connect to").required(false))
        )
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::new()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();
    debug!("Starting environment logger");
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let daemon = sub_matches.get_flag("daemon");
            let bootnode = sub_matches.get_one::<String>("bootnode");
            startup::start(&startup::StartOptions{
                port: 8080,
                daemon,
                pid: "./hanode.pid".to_string(),
                host: "0.0.0.0".to_string(),
                bootnode: bootnode.map(|bootnode| bootnode.to_string()),
            }).await?;
        },
        Some(("stop", sub_matches)) => {
            let port = sub_matches.get_one::<u16>("port");
            let p: u16 = match port {
                Some(port) => port.clone(),
                None => 8080,
            };
            let host = sub_matches.get_one::<String>("host");
            let h = match host {
                Some(host) => host.clone(),
                None => "127.0.0.1".to_string(),
            };
            startup::stop(startup::StopOptions{
                port: p,
                host: h,
            }).await?;
        },
        _ => error!("not implemented"),
    }
    Ok(())
}
