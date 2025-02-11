use bittorrentclient::decode::decode_bencoded_value;
use std::any::Any;
use std::env;

fn print_bencoded_value(value: &Box<dyn Any>) {
    if let Some(s) = value.downcast_ref::<String>() {
        print!("\"{}\"", s);
    } else if let Some(n) = value.downcast_ref::<i64>() {
        print!("{}", n);
    } else if let Some(vec) = value.downcast_ref::<Vec<Box<dyn Any>>>() {
        print!("[");
        for (i, item) in vec.iter().enumerate() {
            print_bencoded_value(item);
            if i < vec.len() - 1 {
                print!(", ");
            }
        }
        print!("]");
    } else {
        print!("<Unknown Type>");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} decode <bencoded_value>", args[0]);
        return;
    }

    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);

        print_bencoded_value(&decoded_value);
    } else {
        println!("Unknown command: {}", command);
    }
}
