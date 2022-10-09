
use std::error::Error;

use clap::{arg, Command, ArgMatches};
use env_logger::{Builder, Target};
use log::{error, debug};
mod startup;
mod utils;

fn cli() -> Command {
    let port_arg = arg!(-p - -port <PORT> "Specify a port to connect to").value_parser(clap::value_parser!(u16).range(3000..)).required(false);
    let host_arg = arg!(-H - -host <HOST> "Specify a host to connect to").required(false);
    Command::new("hanode")
        .about("A server for manage node")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
               .about("Start a node")
               .arg(arg!(-d - -daemon "Running in daemon mode"))
               .arg(arg!(--bootnode <BOOTNODE> "Specify a boot node to connect").required(false))
               .arg(&port_arg)
               .arg(&host_arg)
        )
        .subcommand(
            Command::new("stop")
               .about("Stop a node")
               .arg(&port_arg)
               .arg(&host_arg)
        )
        .subcommand(
            Command::new("peers")
               .about("List all peers")
               .arg(&port_arg)
               .arg(&host_arg)
        )
        .subcommand(
            Command::new("boardcast")
               .about("Stop a node")
               .arg(&port_arg)
               .arg(&host_arg)
               .arg(arg!(<MESSAGE> "Specify a message to boardcast"))
        )

}

fn get_server_opts(sub_matches: &ArgMatches) -> startup::ServerOptions {
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
    startup::ServerOptions{
        port: p,
        host: h,
    }
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
            let server_opts = get_server_opts(&sub_matches);
            startup::start(&startup::StartOptions{
                server_opts: server_opts,
                daemon,
                pid: "./hanode.pid".to_string(),
                bootnode: bootnode.map(|bootnode| bootnode.to_string()),
            }).await?;
        },
        Some(("stop", sub_matches)) => {
            startup::stop(get_server_opts(&sub_matches)).await?;
        },
        Some(("peers", sub_matches)) => {
            startup::list_peers(get_server_opts(&sub_matches)).await?;
        },
        Some(("boardcast", sub_matches)) => {
            let message = sub_matches.get_one::<String>("MESSAGE");
            let m = match message {
                Some(host) => host.clone(),
                None => "".to_string(),
            };
            startup::boardcast(startup::BoardcastOptions{
                server_opts: get_server_opts(&sub_matches),
                msg: m,
            }).await?;
        },
        _ => error!("not implemented"),
    }
    Ok(())
}
