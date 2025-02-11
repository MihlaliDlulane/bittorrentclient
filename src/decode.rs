
use std::any::Any;

#[allow(dead_code)]
pub fn decode_bencoded_value(encoded_value: &str) -> Box<dyn Any> {

    let de_type = decode_type(encoded_value);
    match de_type {
        v if v == "integer" => {
            return Box::new(decode_integer(encoded_value));
        }
        v if v == "string" => {
            return Box::new(decode_string(encoded_value));
        }
        v if v == "list" => {
            return Box::new(decode_lists(encoded_value));
        }
        _ => {
            panic!("Unhandled encoded value: {}", encoded_value)
        }
    }
}

fn decode_type(encoded_value: &str) -> String {
    let string_ecode = std::string::String::from(encoded_value);
    match string_ecode.chars().next().unwrap() {
        'i' => {
            return "integer".to_string();
        }
        ch if ch.is_numeric() => {
            return "string".to_string();
        }
        'l' => {
            return "list".to_string()
        }
        _ => {
            return "unknown".to_string();
        }
    }
}

fn decode_integer(encoded_value: &str) -> i64 {
    let string_ecode = std::string::String::from(encoded_value);
    let uncoded_value = string_ecode.trim_matches(['i','e']);
    // If encoded_value starts with a digit, it's a number
    if uncoded_value.chars().next().unwrap().is_digit(10) || uncoded_value.chars().next().unwrap() == '-' {
        return uncoded_value.parse::<i64>().unwrap();
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

fn decode_string(encoded_value: &str) ->  String {
    let string_ecode = std::string::String::from(encoded_value);
    let decoded_value = string_ecode.trim_matches(|c:char| c.is_numeric()).trim_matches(':');
    return std::string::String::from(decoded_value);
}

fn decode_lists(encoded_value: &str) -> Vec<Box<dyn Any>> {
    let itemslist: String = encoded_value.chars().skip(1).take(encoded_value.len().saturating_sub(2)).collect();
    let mut itemschar: Vec<char> = itemslist.chars().collect();
    let mut resultarray: Vec<Box<dyn Any>> = Vec::new();

    //println!("first items: {:?}", itemschar);

    while !itemschar.is_empty() {
        match itemschar[0] {
            'i' => {
                //println!("items before drain int: {:?} len: {}", itemschar, itemschar.len());
                if let Some(endindex) = itemschar.iter().position(|&c| c == 'e') {
                    let inter_value: String = itemschar.drain(..=endindex).collect();
                    resultarray.push(Box::new(decode_integer(&inter_value)));
                }
                //println!("items after drain int: {:?}", itemschar);
            }
            v if v.is_numeric() => {
               // println!("items before drain string: {:?} len: {}", itemschar, itemschar.len());
                if let Some(colon_index) = itemschar.iter().position(|&c| c == ':') {
                    let len_string: String = itemschar.drain(..colon_index).collect();
                    let numlen: usize = len_string.parse().unwrap();
                    itemschar.remove(0); // Remove `:`
                    let stringvalue: String = itemschar.drain(..numlen).collect();
                    resultarray.push(Box::new(decode_string(&stringvalue)));
                }
               // println!("items after drain string: {:?}", itemschar);
            }
            'l' => {
                //println!("items before drain list: {:?}", itemschar);
                let mut depth = 1;
                let mut j = 1;

                while j < itemschar.len() {
                    match itemschar[j] {
                        'l' => depth += 1,
                        'e' => {
                            if is_valid_end_of_list(j, &itemschar) {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }

                if depth == 0 {
                    let sublist_str: String = itemschar.drain(..=j).collect();
                    resultarray.push(Box::new(decode_lists(&sublist_str)));
                }
               // println!("items after drain list: {:?}", itemschar);
            }
            _ => {
                itemschar.remove(0); // Skip unknown chars
            }
        }
    }

    resultarray
}


// Helper function to check if 'e' is a valid list terminator
fn is_valid_end_of_list(index: usize, itemschar: &[char]) -> bool {
    if index == itemschar.len() - 1 {
        return true; // 'e' at the end
    }
    let next_char = itemschar.get(index + 1);
    match next_char {
        Some('i') => {
            if let Some(next_digit) = itemschar.get(index + 2) {
                return next_digit.is_numeric(); // 'e' before 'i' with a digit after it
            }
        }
        Some(c) if c.is_numeric() => {
            // Check if there's a ':' after a valid number
            let mut j = index + 1;
            while j < itemschar.len() && itemschar[j].is_numeric() {
                j += 1;
            }
            return itemschar.get(j) == Some(&':'); // 'e' before a valid string encoding
        }
        _ => {}
    }
    false
}
