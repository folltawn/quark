use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Call {
        name: String,
        args: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.advance() {
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(&expected) => {
                Ok(())
            }
            Some(token) => Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, token),
            }),
            None => Err(ParseError {
                message: format!("Expected {:?}, but no more tokens", expected),
            }),
        }
    }

    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError> {
        self.expect(Token::LParen)?;
        
        let arg = match self.advance() {
            Some(Token::String(s)) => s.clone(),
            Some(token) => {
                return Err(ParseError {
                    message: format!("Expected string, got {:?}", token),
                })
            }
            None => {
                return Err(ParseError {
                    message: "Expected argument".to_string(),
                })
            }
        };
        
        self.expect(Token::RParen)?;
        self.expect(Token::Semicolon)?;
        
        Ok(Expr::Call {
            name,
            args: vec![arg],
        })
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        
        while let Some(token) = self.peek() {
            match token {
                Token::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    
                    if name == "echo" {
                        statements.push(self.parse_call(name)?);
                    } else {
                        return Err(ParseError {
                            message: format!("Unknown function: {}", name),
                        });
                    }
                }
                Token::EOF => break,
                Token::Illegal(ch) => {
                    return Err(ParseError {
                        message: format!("Invalid character: '{}'", ch),
                    });
                }
                _ => {
                    let token = self.advance().unwrap();
                    return Err(ParseError {
                        message: format!("Unexpected token: {:?}", token),
                    });
                }
            }
        }
        
        Ok(Program { statements })
    }
}