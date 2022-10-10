
use std::{error::Error, path::Path, fs};
use clap::{arg, Command, ArgMatches};
use dirs::home_dir;
use env_logger::{Builder, Target};
use log::{error, debug};
mod startup;
mod utils;

fn cli() -> Command {
    let port_arg = arg!(-p - -port <PORT> "Specify a port to listen or connect to").value_parser(clap::value_parser!(u16).range(3000..)).required(false);
    let host_arg = arg!(-H - -host <HOST> "Specify a host to listen or connect to").required(false);
    let uds_path_arg = arg!(--sock <SOCK_FILE> "Specify a socket file to connect to, default is $HOME/.hanode/hanode.sock").required(false);
    let data_dir_arg = arg!(--data_dir <DATA_DIR> "Data directory, default is $USER_HOME/.hanode").required(false);
    Command::new("hanode")
        .about("A server for manage node")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
               .about("Start a node")
               .arg(arg!(-d - -daemon "Running in daemon mode"))
               .arg(arg!(--server "If open the server"))
               .arg(&data_dir_arg)
               .arg(arg!(--bootnode <BOOTNODE> "Specify a boot node to connect").required(false))
               .arg(&port_arg)
               .arg(&host_arg)
               .arg(&uds_path_arg)
        )
        .subcommand(
            Command::new("stop")
               .about("Stop a node")
               .arg(&data_dir_arg)
               .arg(&port_arg)
               .arg(&host_arg)
               .arg(&uds_path_arg)
        )
        .subcommand(
            Command::new("peers")
               .about("List all peers")
               .arg(&data_dir_arg)
               .arg(&port_arg)
               .arg(&host_arg)
               .arg(&uds_path_arg)
        )
        .subcommand(
            Command::new("boardcast")
               .about("Stop a node")
               .arg(&data_dir_arg)
               .arg(&port_arg)
               .arg(&host_arg)
               .arg(arg!(<MESSAGE> "Specify a message to boardcast"))
               .arg(&uds_path_arg)
        )

}

fn get_datadir(sub_matches: &ArgMatches) -> String {
    let data_dir = match sub_matches.get_one::<String>("data_dir") {
        Some(dir) => dir.clone(),
        None => match home_dir() {
            Some(dir) => format!("{}/{}", String::from(dir.clone().to_str().unwrap()), ".hanode"),
            None => ".hanode".to_string()
        },
    };
    // Check if the directory exists
    let data_dir_path = Path::new(&data_dir);
    if !data_dir_path.exists() {
        // Create a new directory
        fs::create_dir_all(&data_dir).unwrap();
    }
    data_dir
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
    let server = match sub_matches.try_contains_id("server") {
        Ok(id) => match id {
            true => sub_matches.get_flag("server"),
            false => false,
        },
        _ => false,
    };
    let uds_path = match sub_matches.get_one::<String>("sock") {
        Some(path) => path.clone(),
        None => Path::new(&get_datadir(&sub_matches)).join("hanode.sock").to_str().unwrap().to_string(),
    };
    startup::ServerOptions{
        server: server,
        port: p,
        host: h,
        uds_path: uds_path,
    }
}

fn get_daemon_options(sub_matches: &ArgMatches) -> startup::DaemonOptions {
    let daemon = sub_matches.get_flag("daemon");
    let data_dir = get_datadir(sub_matches);
    // Check if the directory exists
    let data_dir_path = Path::new(&data_dir);
    let pid_path = data_dir_path.join("hanode.pid");
    let err_path = data_dir_path.join("error.log");
    let info_path = data_dir_path.join("info.log");
    startup::DaemonOptions {
        daemon,
        pid: String::from(pid_path.to_str().unwrap()),
        err_file: String::from(err_path.to_str().unwrap()),
        log_file: String::from(info_path.to_str().unwrap()),
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
            let bootnode = sub_matches.get_one::<String>("bootnode");
            startup::start(&startup::StartOptions{
                server_opts: get_server_opts(&sub_matches),
                daemon_opts: get_daemon_options(&sub_matches),
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
