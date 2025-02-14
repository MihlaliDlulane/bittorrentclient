use std::any::Any;
use std::collections::HashMap;

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
        v if v == "dict" => {
            return Box::new(decode_dicts(encoded_value));
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
        'd' => {
            return "dict".to_string()
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

fn decode_dicts(encoded_value: &str) -> HashMap<String,Box<dyn Any>> {

    let mut resultdict: HashMap<String,Box<dyn Any>> = HashMap::new();
    let itemslist: String = encoded_value.chars().skip(1).take(encoded_value.len().saturating_sub(2)).collect();
    let mut itemschar: Vec<char> = itemslist.chars().collect();

    while !itemschar.is_empty(){

        //Get key
        match itemschar[0]{

            v if v.is_numeric() =>  
                {
                //println!("items char:{:?}",itemschar);
                let colon_index = itemschar.iter().position(|&c| c == ':');
                let len_string: String = itemschar.drain(..colon_index.unwrap()).collect();
                //println!("colon_index: {:?} len_string: {:?}",colon_index,len_string);
                let numlen: usize = len_string.parse().unwrap();
                itemschar.remove(0); // Remove `:`
                let stringvalue: String = itemschar.drain(..numlen).collect();
                let keyvalue = decode_string(&stringvalue);

                let handler_return = handledictmatch(itemschar, resultdict, keyvalue);
                itemschar = handler_return.0;
                resultdict = handler_return.1;
                }
            'd'  => {
                    //println!("dict found, items char: {:?}",itemschar);
                    let mut depth = 1;
                    let mut j = 1;
                    while j < itemschar.len() {
                        match itemschar[j] {
                            'd' => depth += 1,
                            'e' => {
                                //println!("found e:{:?} and depth is: {:?}",j,depth);
                                if is_valid_end_of_dict(j, &itemschar) {
                                    depth -= 1;
                                    //println!("e is valid:{:?} and depth is:{:?}",j,depth);
                                    if depth == 0 {
                                        //println!("got depth to zero");
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                
                    if depth == 0 {
                       // println!("depth is zero, itemschar:{:?}",itemschar);
                        let sublist_str: String = itemschar.drain(..=j).collect();
                        //println!("sublist_str: {:?}\n items char:{:?}",sublist_str,itemschar);

                        let mut sublist_char:Vec<char> = sublist_str.chars().collect();
                        sublist_char.remove(0); // Remove `d`
                        let colon_index = sublist_char.iter().position(|&c| c == ':');
                        let len_string: String = sublist_char.drain(..colon_index.unwrap()).collect();
                        let numlen: usize = len_string.parse().unwrap();
                        sublist_char.remove(0); // Remove `:`
                        let stringvalue: String = sublist_char.drain(..numlen).collect();
                        let keyvalue = decode_string(&stringvalue);

                        resultdict.insert(keyvalue,Box::new(decode_dicts(&sublist_str)) );
                    }
            }
            _ => {}
        }

    }



    return resultdict;
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

fn is_valid_end_of_dict(index: usize, itemschar: &[char]) -> bool {
    if index == itemschar.len() - 1 {
        return true; // 'e' at the end
    }
    
    let next_char = itemschar.get(index + 1);
    match next_char {
        Some(c) if c.is_numeric() => {
            // Check if there's a ':' after a valid number
            let mut j = index + 1;
            while j < itemschar.len() && itemschar[j].is_numeric() {
                j += 1;
            }
            return itemschar.get(j) == Some(&':'); // 'e' before a valid string key
        }
        Some('d') => {
            // Before 'd' must be a valid string key (numeric prefix followed by ':')
            let mut j = index;
            while j > 0 && itemschar[j - 1].is_numeric() {
                j -= 1;
            }
            return j > 0 && itemschar[j - 1] == ':'; // Check if it's a valid key
        }
        _ => {}
    }
    false
}


fn handledictmatch(mut itemschar: Vec<char>,mut resultdict: HashMap<String,Box<dyn Any>>,keyvalue:String) -> (Vec<char>,HashMap<String,Box<dyn Any>>){

    match itemschar[0] {
        'i' => {
            if let Some(endindex) = itemschar.iter().position(|&c| c == 'e') {
                let inter_value: String = itemschar.drain(..=endindex).collect();
                resultdict.insert(keyvalue,Box::new(decode_integer(&inter_value)) );
                return (itemschar,resultdict);
            }
        }
        v if v.is_numeric() => {
            if let Some(colon_index) = itemschar.iter().position(|&c| c == ':') {
                let len_string: String = itemschar.drain(..colon_index).collect();
                let numlen: usize = len_string.parse().unwrap();
                itemschar.remove(0); // Remove `:`
                let stringvalue: String = itemschar.drain(..numlen).collect();
                resultdict.insert(keyvalue,Box::new(decode_string(&stringvalue)) );
                return (itemschar,resultdict);
            }
        }
        
        'l' => {
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
                resultdict.insert(keyvalue,Box::new(decode_lists(&sublist_str)) );
                return (itemschar,resultdict);
            }
        }

        'd' => {
            let mut depth = 1;
            let mut j = 1;

            while j < itemschar.len() {
                match itemschar[j] {
                    'd' => depth += 1,
                    'e' => {
                        if is_valid_end_of_dict(j, &itemschar) {
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
                resultdict.insert(keyvalue,Box::new(decode_dicts(&sublist_str)) );
                return (itemschar,resultdict);
            }
        }

        _ => {
                return (itemschar,resultdict);
        }

    }

    return (itemschar,resultdict);

}