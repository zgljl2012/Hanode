
use std::error::Error;

use clap::{arg, Command};
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
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let daemon = sub_matches.get_flag("daemon");
            let bootnode = sub_matches.get_one::<String>("bootnode");
            startup::start(&startup::StartOptions{
                port: 3200, 
                daemon: daemon,
                pid: "./hanode.pid".to_string(),
                host: "0.0.0.0".to_string(),
                bootnode: match bootnode {
                    Some(bootnode) => Some(bootnode.to_string()),
                    None => None,
                },
            }).await?;
        },
        _ => println!("not implemented"),
    }
    Ok(())
}
