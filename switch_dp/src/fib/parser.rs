pub struct Header<'a> {
    pub fields: &'a [Field],
    pub use_fields_index: &'a [usize],
}


pub struct Field {
    pub start_byte_pos: usize,
    pub start_bit_mask: u8,
    pub end_byte_pos: usize,
    pub end_bit_mask: u8,
}


pub struct  ParseMatchPattern<'a> {
    pub value: &'a [&'a [u8]],
    pub next_parser: &'a Parser<'a>,
}


pub enum Parser<'a> {
    Parser {
        extract_headers: &'a [&'a Header<'a>],
        select_keys: &'a [(&'a Header<'a>, usize)],  // (header, field index)
        match_patterns: &'a [ParseMatchPattern<'a>],
    },
    Accept,
}


impl <'a> Parser<'a> {
    pub fn parse(&self, packet: &[u8]) -> Option<&Parser> {
        match self {
            Parser::Parser { extract_headers: _, select_keys, match_patterns } => {
                for match_pattern in match_patterns.iter() {
                    let mut match_result = true;
                    for i in 0..select_keys.len() {
                        let field = &select_keys[i].0.fields[select_keys[i].1];

                        if match_pattern.value[i][0] != packet[field.start_byte_pos] & field.start_bit_mask {
                            match_result = false;
                            break;
                        }

                        if field.start_byte_pos == field.end_byte_pos {
                            continue;
                        } else {
                            for j in field.start_byte_pos+1..field.end_byte_pos {
                                if match_pattern.value[i][j] != packet[j] {
                                    match_result = false;
                                    break;
                                }
                            }
                            if match_pattern.value[i][field.end_byte_pos] != packet[field.end_byte_pos] & field.end_bit_mask {
                                match_result = false;
                                break;
                            }
                        }
                    }

                    if match_result {
                        return Some(match_pattern.next_parser);
                    }
                }
                None
            },
            Parser::Accept => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parser_parse() {
    }
}
