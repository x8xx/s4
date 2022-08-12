use crate::p4c::token::Token;
use crate::p4c::token::TokenKind;

fn extract_number(chars: &[char], pos: usize) -> (Token, usize) {
    let mut end_pos = chars.len();
    for i in (pos+1)..end_pos {
        let c = read(chars, i).unwrap();
        if c < '0' || c > '9' {
            end_pos = i;
            break; 
        }
    }

    let mut number:u64 = 0;
    for i in pos..end_pos {
        number += ((chars[i] as u8 - '0' as u8) as i32 * 10_i32.pow((end_pos - i - 1).try_into().unwrap())) as u64;
    }

    (Token::Token {
        kind: TokenKind::Num(number),
        token: Box::new(Token::None),
    }, end_pos)
}


fn extract_hex_number(chars: &[char], pos: usize) -> (Token, usize) {
    let mut end_pos = chars.len();
    for i in pos..end_pos {
        let c = read(chars, i).unwrap();
        if (c < '0' || c > '9') &&
           (c < 'a' || c > 'f') &&
           (c < 'A' || c > 'F') {
            end_pos = i;
            break; 
        }
    }

    let mut number:u64 = 0;
    for i in pos..end_pos {
        let c = chars[i];
        let c_u8: u8 = if c >= '0' && c <= '9' {
            c  as u8 - '0' as u8
        } else {
            if c >= 'a' && c <= 'f' {
                (c as u8 - 'a' as u8) + 10
            } else {
                (c as u8 - 'A' as u8) + 10
            }
        };
        number += (c_u8 as i32 * 2_i32.pow((4 * (end_pos - i - 1)).try_into().unwrap())) as u64;
    }

    (Token::Token {
        kind: TokenKind::Num(number),
        token: Box::new(Token::None),
    }, end_pos)
}


fn read(src: &[char], pos: usize) -> Option<char> {
    if src.len() > pos {
        return Some(src[pos as usize]);
    }
    None
}


pub fn tokenize(src: &str) -> Token {
    let chars: Vec<char> = src.chars().collect();
    let mut root = Token::Token {
        kind: TokenKind::Eof,
        token: Box::new(Token::None),
    };

    let mut token = &mut root;
    let mut pos: usize = 0;
    loop {
        let c = read(&chars, pos);
        if None == c {
            break;
        }

        if c.unwrap() >= '1' && c.unwrap() <= '9' {
            let (n_token, n_pos) = extract_number(&chars, pos);
            token.set_next_token(Box::new(n_token));
            token =  token.get_next_token();
            pos = n_pos;
        } else if c.unwrap() == '0' {
            let nc = read(&chars, pos + 1);
            if None == nc {
                let n_token = Token::Token {
                    kind: TokenKind::Num(0),
                    token: Box::new(Token::None),
                };
                token.set_next_token(Box::new(n_token));
                token = token.get_next_token();
                break;
            }
            
            if nc.unwrap() != ' ' && nc.unwrap() != 'x' {
                // TODO: occured error
            }

            if nc.unwrap() == 'x' {
                let (n_token, n_pos) = extract_hex_number(&chars, pos + 2);
                token.set_next_token(Box::new(n_token));
                token =  token.get_next_token();
                pos = n_pos;
            } else if nc.unwrap() == ' ' {
                let n_token = Token::Token {
                    kind: TokenKind::Num(0),
                    token: Box::new(Token::None),
                };
                token.set_next_token(Box::new(n_token));
                token = token.get_next_token();
            }

        }

    };

    root
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_number() {
        let chars: Vec<char> = "tes 1055 30998233".chars().collect();
        let (token_1, pos_1) = extract_number(&chars, 4);
        assert_eq!(token_1.get_kind().unwrap().get_num().unwrap(), 1055);
        assert_eq!(pos_1, 8);
        let (token_2, pos_2) = extract_number(&chars, 9);
        assert_eq!(token_2.get_kind().unwrap().get_num().unwrap(), 30998233);
        assert_eq!(pos_2, 17);

    }

    #[test]
    fn test_extract_hex_number() {
        let chars: Vec<char> = "tes 0xff 0x4f9".chars().collect();
        let (token_1, pos_1) = extract_hex_number(&chars, 6);
        assert_eq!(token_1.get_kind().unwrap().get_num().unwrap(), 255);
        assert_eq!(pos_1, 8);
        let (token_2, pos_2) = extract_hex_number(&chars, 11);
        assert_eq!(token_2.get_kind().unwrap().get_num().unwrap(), 1273);
        assert_eq!(pos_2, 14);

    }

    #[test]
    fn test_read() {
        let chars: Vec<char> = "test".chars().collect();
        assert_eq!('t', read(&chars, 0).unwrap());
        assert_eq!('e', read(&chars, 1).unwrap());
        assert_eq!('s', read(&chars, 2).unwrap());
        assert_eq!('t', read(&chars, 3).unwrap());
        assert_eq!(Option::None, read(&chars, 4));
    }
}
