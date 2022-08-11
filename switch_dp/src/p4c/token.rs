#[derive(Debug)]
pub enum Reserved {
    Const,
    Typedef,
    Bit,
    Header,
    Parser,
}

#[derive(Debug)]
pub enum TokenKind {
    Reserved(Reserved),
    Str(String),
    Num(u64),
    Eof,
}


impl TokenKind {
    pub fn get_num(&self) -> Option<u64> {
        match self {
            TokenKind::Num(num) => Some(*num),
            _ => None
        }
    }
}


#[derive(Debug)]
pub enum Token {
    Token {
        kind: TokenKind,
        token: Box<Token>,
    },
    None,
}

impl Token {
    pub fn set_next_token(&mut self, new_token: Box<Token>) {
        match self {
            Token::Token { kind: _, ref mut token } => {
                *token = new_token;
            },
            _ => {}
        }
    }

    pub fn get_next_token(&mut self) -> &mut Token {
        match self {
            Token::Token {kind: _, ref mut token} => {
                &mut *token
            },
            Token::None => self,
        }
    }

    pub fn get_kind(&self) -> Option<&TokenKind> {
        match self {
            Token::Token {kind, token: _} => {
                Some(kind)
            },
            Token::None => None,
        }
    }
}
