use serde::Deserialize;
use std::collections::HashMap;
use crate::dpdk::*;


pub struct Header<'a> {
    pub fields: &'a [Field],
    pub used_fields: &'a [Field],
}


pub struct Field {
    pub start_byte_pos: usize,
    pub start_bit_mask: u8,
    pub end_byte_pos: usize,
    pub end_bit_mask: u8,
}


pub enum Parser<'a> {
    Parser {
        extract_headers: &'a [&'a Header<'a>],
        select_keys: &'a [(&'a Field, usize)],  // (header, base_pos, field index)
        match_patterns: &'a [ParseMatchPattern<'a>],
    },
    Accept,
}


pub struct  ParseMatchPattern<'a> {
    pub value: &'a [&'a [u8]],
    pub next_parser: &'a Parser<'a>,
}


impl <'a> Parser<'a> {
    pub fn parse(&self, packet: &[u8]) -> Option<&Parser> {
        match self {
            Parser::Parser { extract_headers: _, select_keys, match_patterns } => {
                for match_pattern in match_patterns.iter() {
                    let mut match_result = true;
                    for i in 0..select_keys.len() {
                        let field = &select_keys[i].0;
                        let start_byte_pos = field.start_byte_pos + select_keys[i].1;
                        let end_byte_pos = field.end_byte_pos + select_keys[i].1;

                        if match_pattern.value[i][0] != packet[start_byte_pos] & field.start_bit_mask {
                            match_result = false;
                            break;
                        }

                        if start_byte_pos == end_byte_pos {
                            continue;
                        } else {
                            for j in start_byte_pos+1..end_byte_pos {
                                if match_pattern.value[i][j] != packet[j] {
                                    match_result = false;
                                    break;
                                }
                            }
                            if match_pattern.value[i][end_byte_pos] != packet[end_byte_pos] & field.end_bit_mask {
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


#[derive(Deserialize, Debug)]
pub struct DataPlaneConfig {
    pub headers: HashMap<String, Vec<u16>>,
    pub parsers: HashMap<String, ParserConfig>,
}


#[derive(Deserialize, Debug)]
pub struct ParserConfig {
    pub extract_headers: Vec<String>,
    pub select_keys: HashMap<String, u16>,
    pub match_patterns: Vec<ParserMatchPatternConfig>,
}


#[derive(Deserialize, Debug)]
pub struct ParserMatchPatternConfig {
    pub value: Vec<String>,
    pub next_parser: String,
}


impl DataPlaneConfig {
    pub fn new(json_str: &str) -> Self {
        let dp_conf: DataPlaneConfig = serde_json::from_str(json_str).unwrap();
        dp_conf
    }

    pub fn sum_header_field_len(&self) -> usize {
        let mut sum: usize = 0;
        for hdr in self.headers.iter() {
            sum += hdr.1.len();
        }
        sum
    }

    pub fn sum_extract_headers_len(&self) -> usize {
        let mut sum: usize = 0;
        for parser in self.parsers.iter() {
            sum += parser.1.extract_headers.len();
        }
        sum
    }

    pub fn sum_select_keys_len(&self) -> usize {
        let mut sum: usize = 0;
        for parser in self.parsers.iter() {
            sum += parser.1.select_keys.len();
        }
        sum
    }

    // pub fn sum_select_keys_len(&self) -> usize {
    //     let mut sum: usize = 0;
    //     for parser in self.parsers.iter() {
    //         sum += parser.1.select_keys.len();
    //     }
    //     sum
    // }
}



// pub fn load_dataplane_config_from_json(json_str: &str) -> Parser {
pub fn load_dataplane_config_from_json(json_str: &str) -> DataPlaneConfig {
    let dp_conf: DataPlaneConfig = serde_json::from_str(json_str).unwrap();
    dp_conf
}


pub fn get_sample_dp_config() -> String {
    "
        {
            \"headers\": {
                \"ethernet\": [48, 48, 16],
                \"ip\": [4, 4, 8, 16, 16, 3, 13, 8, 8, 16, 32, 32]
            },
            \"parsers\": {
                \"start\": {
                    \"extract_headers\": [\"ethernet\"],
                    \"select_keys\": {
                        \"ethernet\": 2
                    },
                    \"match_patterns\": [
                        {
                            \"value\": [\"0x800\"],
                            \"next_parser\": \"accept\"
                        }
                    ]
                }
            },
            \"tables\": {
                
            }
        }
    ".to_string()
}


pub fn create_parser<'a>(dp_config: &'a DataPlaneConfig, hdr_hashmap: &'a HashMap<String, &Header>, parser_name: &'a str, base_pos: usize) -> Parser<'a> {
    let parser_conf = &dp_config.parsers[parser_name];

    // create header
    for hdr_name in parser_conf.extract_headers.iter() {
        if !hdr_hashmap.contains_key(hdr_name) {
            let hdr_conf = &dp_config.headers[hdr_name];
            let header = dpdk_memory::malloc::<Header>(hdr_name, 1);

            let fields_mempool_name = format!("{}{}", hdr_name, "_fields");
            let fields = dpdk_memory::malloc::<Field>(&fields_mempool_name, hdr_conf.len() as u32);

            let used_fields_mempool_name = format!("{}{}", hdr_name, "_used_fields");
            let used_fields = dpdk_memory::malloc::<Field>(&used_fields_mempool_name, hdr_conf.len() as u32);
            
            {
                let mut end_byte_pos = 0;
                let mut end_bit_mask = 0;
                for (i, field_bit_size) in hdr_conf.iter().enumerate() {
                    let (start_byte_pos, start_bit_mask): (usize, u8) = if end_bit_mask == 0xff {
                        if *field_bit_size >= 8 {
                            (end_byte_pos + 1, 0xff)
                        } else {
                            let mask_list = [128, 192, 224, 240, 248, 252, 254];
                            (end_byte_pos + 1, mask_list[*field_bit_size as usize - 1])
                        }
                    } else {
                        (end_byte_pos, 0)
                    };


                    // let start_byte_pos = if i > 0 { 
                    //     fields[i - 1].end_byte_pos + if fields[i - 1].end_bit_mask == 0xff { 1 } else { 0 }
                    // } else { 
                    //     0
                    // };
                    // end_byte_pos = start_byte_pos + field_bit_size;
                    
                    fields[i].start_byte_pos = start_byte_pos;
                    fields[i].start_bit_mask = start_bit_mask;
                    fields[i].end_byte_pos = end_byte_pos;
                    fields[i].end_bit_mask = end_bit_mask;
                }
            }

            hdr_hashmap[hdr_name] = &header[0];
        }
    }

    
    Parser::Accept
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parser_parse() {
        let json = get_sample_dp_config();
        load_dataplane_config_from_json(&json);
    }
}
