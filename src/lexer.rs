#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    String(String),
    LParen,
    RParen,
    Semicolon,
    EOF,
    Illegal(char),
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        ident
    }

    fn read_string(&mut self) -> Result<String, LexError> {
        self.advance(); // Skip opening quote
        let mut string = String::new();
        
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance(); // Skip closing quote
                    return Ok(string);
                }
                '\\' => {
                    self.advance(); // Skip backslash
                    if let Some(escaped) = self.peek() {
                        match escaped {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            'r' => string.push('\r'),
                            '"' => string.push('"'),
                            '\\' => string.push('\\'),
                            _ => return Err(LexError {
                                message: format!("Unknown escape sequence: \\{}", escaped),
                                position: self.position,
                            }),
                        }
                        self.advance();
                    } else {
                        return Err(LexError {
                            message: "Incomplete escape sequence".to_string(),
                            position: self.position,
                        });
                    }
                }
                '\n' => {
                    return Err(LexError {
                        message: "Unclosed string".to_string(),
                        position: self.position,
                    });
                }
                _ => {
                    string.push(ch);
                    self.advance();
                }
            }
        }
        
        Err(LexError {
            message: "Unterminated string constant".to_string(),
            position: self.position,
        })
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Пропускаем пробелы
        self.skip_whitespace();
        
        // Проверяем на комментарии
        while let Some('/') = self.peek() {
            if let Some('/') = self.input.get(self.position + 1) {
                // Это комментарий, пропускаем до конца строки
                self.advance(); // первый /
                self.advance(); // второй /
                
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                
                // После комментария могут быть пробелы
                self.skip_whitespace();
            } else {
                break;
            }
        }

        match self.peek() {
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some('"') => Ok(Token::String(self.read_string()?)),
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                Ok(Token::Ident(self.read_ident()))
            }
            Some(ch) if ch.is_ascii() => {
                self.advance();
                Ok(Token::Illegal(ch))
            }
            Some(ch) => {
                let pos = self.position;
                self.advance();
                Err(LexError {
                    message: format!("Invalid character: '{}'", ch),
                    position: pos,
                })
            }
            None => Ok(Token::EOF),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::EOF);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}