use std::env;
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> i32 {

    let string_ecode = std::string::String::from(encoded_value);
    let uncoded_value = string_ecode.trim_matches(['i','e']);
    // If encoded_value starts with a digit, it's a number
    if uncoded_value.chars().next().unwrap().is_digit(10) {
        return uncoded_value.parse::<i32>().unwrap();
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
