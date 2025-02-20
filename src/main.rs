#![allow(unused)]
mod utils;
use clap::{Parser, Subcommand};
use utils::commands::{print_decode,print_info,print_peers};
use utils::torrent_info::{handletorret};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command : Command
}

#[derive(Subcommand)]
enum Command {
    Decode {value:String},
    Info {path:String},
    Peers {path:String}
}


#[tokio::main]
async fn main() -> () {
    let args = Args::parse();
    
    match args.command {
        Command::Decode { value } => {
            print_decode(value);
        }
        Command::Info { path } => {
            handletorret(path);

            //print_info(path);
        }    
        Command::Peers { path } => {
            print_peers(path).await;
        }
    }

    ()
}
