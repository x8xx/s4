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
    pub fn parse(&'a self, packet: &'a [u8], extract_headers: &'a mut [&'a Header<'a>]) {
        let mut parser = Some(self);
        let mut hdr_count = 0;
        for hdr in parser.unwrap().get_extarct_headers().unwrap().iter() {
            extract_headers[hdr_count] = hdr;
            hdr_count += 1;
        }
        while match parser { None => false, _ => true } {
            for hdr in parser.unwrap().get_extarct_headers().unwrap().iter() {
                extract_headers[hdr_count] = hdr;
                hdr_count += 1;
            }
            parser = parser.unwrap().matching(packet);
        }
    }

    fn matching(&self, packet: &[u8]) -> Option<&Parser> {
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

    fn get_extarct_headers(&'a self) -> Option<&'a [&'a Header]> {
        match self {
            Parser::Parser { extract_headers, select_keys: _, match_patterns: _ } => {
                Some(extract_headers)
            },
            _ => None
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
}


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


pub fn create_parser<'a>(dp_config: &'a DataPlaneConfig,
                         parser_hashmap: &'a mut HashMap<String, &Parser<'a>>,
                         hdr_hashmap: &'a mut HashMap<String, &Header<'a>>,
                         parser_name: &'a str,
                         base_pos: usize) -> &'a Parser<'a> {
    let parser = &dpdk_memory::malloc::<Parser>(parser_name, 1)[0];
    parser_hashmap.insert(parser_name.to_string(), parser);

    let parser_conf = &dp_config.parsers[parser_name];

    let extract_headers_mempool_name = format!("{}{}", parser_name, "_extract_headers");
    let extract_headers = dpdk_memory::malloc::<&Header>(&extract_headers_mempool_name, parser_conf.extract_headers.len() as u32);

    // create header
    for (i, hdr_name) in parser_conf.extract_headers.iter().enumerate() {
        if !hdr_hashmap.contains_key(hdr_name) {
            hdr_hashmap.insert(hdr_name.to_string(), create_header(hdr_name, &dp_config.headers[hdr_name]));
        }
        extract_headers[i] = hdr_hashmap[hdr_name];
    }

    // select keys
    

    // match pattern
    &parser
}


// fn create_header<'a, 'b>(hdr_name: &'a str, hdr_conf: &'b [u16]) -> &'b Header<'b> {
fn create_header<'a>(hdr_name: &'a str, hdr_conf: &'a [u16]) -> &'a Header<'a> {
    // let hdr_name_string = hdr_name.to_string();
    // let header = &dpdk_memory::malloc::<Header>(&hdr_name_string, 1)[0];
    let header = &dpdk_memory::malloc::<Header>(&hdr_name, 1)[0];

    let fields_mempool_name = format!("{}{}", hdr_name, "_fields");
    let fields = dpdk_memory::malloc::<Field>(&fields_mempool_name, hdr_conf.len() as u32);

    let used_fields_mempool_name = format!("{}{}", hdr_name, "_used_fields");
    let used_fields = dpdk_memory::malloc::<Field>(&used_fields_mempool_name, hdr_conf.len() as u32);
    
    let mut end_byte_pos = 0;
    let mut end_bit_mask = 0;
    for (i, field_bit_size) in hdr_conf.iter().enumerate() {
        let (start_byte_pos, start_bit_mask, read_bit): (usize, u8, u16) = if end_bit_mask == 0xff {
            if *field_bit_size >= 8 {
                (end_byte_pos + 1, 0xff, 8)
            } else {
                let mask_list = [128, 192, 224, 240, 248, 252, 254];
                (end_byte_pos + 1, mask_list[*field_bit_size as usize - 1], *field_bit_size)
            }
        } else {
            let bit_sapce = 8 - (end_bit_mask as u64).count_ones();
            if bit_sapce > *field_bit_size as u32 {
                let mask_list = [128, 192, 224, 240, 248, 252, 254];
                (end_byte_pos, mask_list[bit_sapce as usize - 1] ^ mask_list[8 - *field_bit_size as usize] , *field_bit_size)
            } else {
                (end_byte_pos, end_bit_mask ^ 0xff, bit_sapce as u16)
            }
        };

        let field_bit_size = *field_bit_size - read_bit;
        if field_bit_size == 0 {
            fields[i].start_byte_pos = start_byte_pos;
            fields[i].start_bit_mask = start_bit_mask;
            fields[i].end_byte_pos = start_byte_pos;
            fields[i].end_bit_mask = start_bit_mask;
        } else {
            let residue_bit = field_bit_size % 8;
            if residue_bit == 0 {
                fields[i].end_byte_pos = (field_bit_size / 8) as usize;
                fields[i].end_bit_mask = 0xff;
            } else {
                fields[i].end_byte_pos = (field_bit_size / 8) as usize + 1;
                fields[i].end_bit_mask = 0xff;
            }
        }

        end_byte_pos = fields[i].end_byte_pos;
        end_bit_mask = fields[i].end_bit_mask;
    }
    header
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
