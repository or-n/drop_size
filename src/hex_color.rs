fn is_valid(text: &str) -> bool {
    if !text.starts_with('#') {
        return false;
    }
    match text.len() {
        7 | 4 => text[1..].chars().all(|c| c.is_ascii_hexdigit()),
        _ => false,
    }
}

pub fn decode(text: &str) -> Option<[u8; 3]> {
    if is_valid(text) {
        let hex_digits = &text[1..];
        if hex_digits.len() == 6 {
            let r = u8::from_str_radix(&hex_digits[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex_digits[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex_digits[4..6], 16).ok()?;
            Some([r, g, b])
        } else if hex_digits.len() == 3 {
            let r = u8::from_str_radix(&hex_digits[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex_digits[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex_digits[2..3].repeat(2), 16).ok()?;
            Some([r, g, b])
        } else {
            unreachable!()
        }
    } else {
        None
    }
}
