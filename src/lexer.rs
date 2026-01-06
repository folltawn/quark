#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Базовые токены
    Ident(String),
    StringLiteral(String),
    NumberLiteral(String),
    LParen,
    RParen,
    Semicolon,
    Equals,
    EOF,
    Illegal(char),
    
    // Ключевые слова типов
    StringType,
    IntegerType,
    FloatType,
    BooleanType,
    
    // Булевы литералы
    True,
    False,
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

    fn skip_comments(&mut self) {
        while let Some('/') = self.peek() {
            if let Some('/') = self.input.get(self.position + 1) {
                // Пропускаем комментарий
                self.advance(); // /
                self.advance(); // /
                
                while let Some(ch) = self.peek() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                
                // Пропускаем пробелы после комментария
                self.skip_whitespace();
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

    fn read_number(&mut self) -> String {
        let mut number = String::new();
        let mut has_dot = false;
        
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !has_dot {
                if let Some(next) = self.input.get(self.position + 1) {
                    if next.is_ascii_digit() {
                        number.push(ch);
                        self.advance();
                        has_dot = true;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        number
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
        // Пропускаем пробелы и комментарии
        self.skip_whitespace();
        self.skip_comments();

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
            Some('=') => {
                self.advance();
                Ok(Token::Equals)
            }
            Some('"') => Ok(Token::StringLiteral(self.read_string()?)),
            
            // Числа
            Some(ch) if ch.is_ascii_digit() => {
                Ok(Token::NumberLiteral(self.read_number()))
            }
            
            // Идентификаторы и ключевые слова
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_ident();
                Ok(match ident.as_str() {
                    "String" => Token::StringType,
                    "Integer" => Token::IntegerType,
                    "Float" => Token::FloatType,
                    "Boolean" => Token::BooleanType,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Ident(ident),
                })
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