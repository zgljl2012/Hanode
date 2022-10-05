
use std::error::Error;

use clap::{arg, Command};
mod startup;

fn cli() -> Command {
    Command::new("hanode")
        .about("A server for manage node")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
               .about("Start a node")
               .arg(arg!(-d - -daemon "Running in daemon mode"))
        )
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            let daemon = sub_matches.get_flag("daemon");
            println!("start: {:?}", daemon);
            startup::start(&startup::StartOptions{port: 3200, daemon: daemon}).await?;
        },
        _ => println!("not implemented"),
    }
    Ok(())
}
