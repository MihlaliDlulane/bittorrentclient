use std::env;
use bittorrentclient::decode::decode_bencoded_value;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);

        if let Some(s) = decoded_value.downcast_ref::<String>() {
            println!("\"{}\"",s)
        } else if let Some(n) = decoded_value.downcast_ref::<i64>()  {
            println!("{}",n)
        }

    } else {
        println!("unknown command: {}", args[1])
    }
}
