use std::process;

pub fn encode(decoded_token: &str) -> String {
    if decoded_token.len() == 0 {
        process::exit(1);
    }

    // parse the string
    let bytes = decoded_token.as_bytes();
    let mut index = 0;
    let mut encoded_token = String::new();

    while index < bytes.len() {
        let b = bytes[index];

        // ascii graphics are unprintable, we dont accept whitespace, we dont accept unsafe characters
        if !b.is_ascii_graphic()
            || b.is_ascii_whitespace()
            || b == b':' || b == b'\\' || b == b'x'
            || !((b'!'..=b'9').contains(&b)
            || (b';'..=b'[').contains(&b)
            || (b']'..=b'w').contains(&b)
            || (b'y'..=b'~').contains(&b)) {
            // encode in hex
            encoded_token.push_str(&format!("x{:02x}", b));
        } else {
            // keep literal
            encoded_token.push(b as char);
        }

        index += 1;
    }
    encoded_token
}

pub fn decode(encoded_token: &str) -> String {
    let bytes = encoded_token.as_bytes();
    let mut index = 0;

    let mut decoded_token = String::new();
    while index < bytes.len() {
        if bytes[index] == b'x' {
            if index + 2 >= bytes.len() {
                process::exit(1); // malformed token
            }

            let hex_str = &encoded_token[index + 1..index + 3];
            let byte_val = match u8::from_str_radix(hex_str, 16) {
                Ok(b) => b,
                Err(_) => process::exit(1),
            };

            decoded_token.push(byte_val as char);

            index += 3;
        } else {
            let c = bytes[index];
            if (c.is_ascii_graphic() || c == b' ') && c != b':' && c != b'\\' && c != b'x' {
                decoded_token.push(c as char); // what is this syntax?
            } else {
                process::exit(1); // invalid literal
            }
            index += 1;
        }
    }
    decoded_token
}