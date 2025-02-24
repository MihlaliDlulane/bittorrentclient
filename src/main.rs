#[allow(unused)]
mod utils;
use clap::{Parser, Subcommand};
use utils::commands::{print_decode,print_peers,return_peers_and_infohash};
use utils::torrent_info::handletorret;
use utils::tcp::peerhandshake;

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
    Peers {path:String},
    Handshake {torrentfile:String}
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
            match print_peers(path).await {
                Ok(_e) =>{}
                Err(_x) => {}
            }
        }
        Command::Handshake { torrentfile } => {
            match return_peers_and_infohash(torrentfile).await {
                Ok(e) =>{
                    let peerlist = e.0;
                    let infohash = e.1;
                    
                    for x in peerlist {
                        match peerhandshake(x.ip, x.port,&infohash).await{
                            Ok(_s) => { todo!()
                            }
                            Err(e) => {
                                println!("Error:{}",e)
                            }
                        }
                        //println!("Peer: {:?}",x)
                    }

                }
                Err(x) => {
                    println!("{}",x)
                }
            }
        }

    }

    ()
}


// async fn main() -> () {
//     let args = Args::parse();
    
//     match args.command {
//         Command::Decode { value } => {
//             print_decode(value);
//         }
//         Command::Info { path } => {
//             handletorret(path);
//         }    
//         Command::Peers { path } => {
//             match print_peers(path).await {
//                 Ok(_e) => {},
//                 Err(_x) => {}
//             }
//         }
//         Command::Handshake { torrentfile } => {
//             match return_peers_and_infohash(torrentfile).await {
//                 Ok(e) => {
//                     let peerlist = e.0;
//                     let infohash = e.1;

//                     // Create a semaphore with a max of 5 concurrent tasks
//                     let semaphore = Arc::new(Semaphore::new(5));

//                     // Collect all handshake tasks with concurrency limit
//                     let mut tasks = Vec::new();

//                     for x in peerlist {
//                         let semaphore = semaphore.clone();
//                         let task = tokio::spawn(async move {
//                             // Try to acquire a permit
//                             let _permit = semaphore.acquire().await.unwrap();

//                             // Perform the handshake
//                             match peerhandshake(x.ip, x.port, &infohash).await {
//                                 Ok(_) => println!("Handshake with peer {}:{}. Success!", x.ip, x.port),
//                                 Err(e) => println!("Error with peer {}:{} - {}", x.ip, x.port, e),
//                             }

//                             // Permit is released automatically when it goes out of scope
//                         });

//                         tasks.push(task);
//                     }

//                     // Wait for all tasks to complete
//                     for task in tasks {
//                         task.await.unwrap();
//                     }

//                 }
//                 Err(x) => {
//                     println!("{}", x);
//                 }
//             }
//         }
//     }

//     ()
// }
