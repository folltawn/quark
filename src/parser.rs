use crate::lexer::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Variable(String),
    Literal(Value),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaration {
        var_type: VarType,
        name: String,
        value: Value,
    },
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum VarType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}:{}", self.message, self.line, self.column)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    current_line: usize,
    current_column: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            current_line: 1,
            current_column: 1,
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
                line: self.current_line,
                column: self.current_column,
            }),
            None => Err(ParseError {
                message: format!("Expected {:?}, but no more tokens", expected),
                line: self.current_line,
                column: self.current_column,
            }),
        }
    }

    fn parse_type(&mut self) -> Result<VarType, ParseError> {
        match self.advance() {
            Some(Token::StringType) => Ok(VarType::String),
            Some(Token::IntegerType) => Ok(VarType::Integer),
            Some(Token::FloatType) => Ok(VarType::Float),
            Some(Token::BooleanType) => Ok(VarType::Boolean),
            Some(token) => Err(ParseError {
                message: format!("Expected type, got {:?}", token),
                line: self.current_line,
                column: self.current_column,
            }),
            None => Err(ParseError {
                message: "Expected type".to_string(),
                line: self.current_line,
                column: self.current_column,
            }),
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.advance() {
            Some(Token::StringLiteral(s)) => Ok(Value::String(s.clone())),
            Some(Token::NumberLiteral(num)) => {
                if num.contains('.') {
                    match num.parse::<f64>() {
                        Ok(f) => Ok(Value::Float(f)),
                        Err(_) => Err(ParseError {
                            message: format!("Invalid float literal: {}", num),
                            line: self.current_line,
                            column: self.current_column,
                        }),
                    }
                } else {
                    match num.parse::<i64>() {
                        Ok(i) => Ok(Value::Integer(i)),
                        Err(_) => Err(ParseError {
                            message: format!("Invalid integer literal: {}", num),
                            line: self.current_line,
                            column: self.current_column,
                        }),
                    }
                }
            }
            Some(Token::True) => Ok(Value::Boolean(true)),
            Some(Token::False) => Ok(Value::Boolean(false)),
            Some(token) => Err(ParseError {
                message: format!("Expected value, got {:?}", token),
                line: self.current_line,
                column: self.current_column,
            }),
            None => Err(ParseError {
                message: "Expected value".to_string(),
                line: self.current_line,
                column: self.current_column,
            }),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(Token::StringLiteral(_)) | Some(Token::NumberLiteral(_)) | 
            Some(Token::True) | Some(Token::False) => {
                let value = self.parse_value()?;
                Ok(Expr::Literal(value))
            }
            Some(Token::Ident(_)) => {
                let name = match self.advance() {
                    Some(Token::Ident(name)) => name.clone(),
                    _ => unreachable!(),
                };
                Ok(Expr::Variable(name))
            }
            Some(Token::LParen) => {
                self.advance(); // пропускаем (
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: "Expected expression".to_string(),
                line: self.current_line,
                column: self.current_column,
            }),
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary_expression()?;
        
        while let Some(Token::Plus) = self.peek() {
            self.advance(); // пропускаем +
            let right = self.parse_primary_expression()?;
            
            left = Expr::BinaryOp {
                left: Box::new(left),
                op: BinOp::Add,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_declaration(&mut self) -> Result<Stmt, ParseError> {
        let var_type = self.parse_type()?;
        
        let name = match self.advance() {
            Some(Token::Ident(name)) => name.clone(),
            Some(token) => {
                return Err(ParseError {
                    message: format!("Expected variable name, got {:?}", token),
                    line: self.current_line,
                    column: self.current_column,
                })
            }
            None => {
                return Err(ParseError {
                    message: "Expected variable name".to_string(),
                    line: self.current_line,
                    column: self.current_column,
                })
            }
        };
        
        self.expect(Token::Equals)?;
        
        let value = self.parse_value()?;
        
        match (&var_type, &value) {
            (VarType::String, Value::String(_)) => {}
            (VarType::Integer, Value::Integer(_)) => {}
            (VarType::Float, Value::Float(_)) => {}
            (VarType::Boolean, Value::Boolean(_)) => {}
            _ => {
                return Err(ParseError {
                    message: format!("Type mismatch: cannot assign {:?} to {:?}", value, var_type),
                    line: self.current_line,
                    column: self.current_column,
                })
            }
        }
        
        self.expect(Token::Semicolon)?;
        
        Ok(Stmt::Declaration {
            var_type,
            name,
            value,
        })
    }

    fn parse_call(&mut self, name: String) -> Result<Expr, ParseError> {
        self.expect(Token::LParen)?;
        
        let mut args = Vec::new();
        
        if let Some(Token::RParen) = self.peek() {
            // Нет аргументов
        } else {
            args.push(self.parse_expression()?);
        }
        
        self.expect(Token::RParen)?;
        self.expect(Token::Semicolon)?;
        
        Ok(Expr::Call {
            name,
            args,
        })
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut statements = Vec::new();
        
        while let Some(token) = self.peek() {
            match token {
                Token::StringType | Token::IntegerType | Token::FloatType | Token::BooleanType => {
                    statements.push(self.parse_declaration()?);
                }
                
                Token::Ident(name) => {
                    let name = name.clone();
                    self.advance();
                    
                    if name == "echo" {
                        statements.push(Stmt::Expression(self.parse_call(name)?));
                    } else {
                        return Err(ParseError {
                            message: format!("Unknown function or variable: {}", name),
                            line: self.current_line,
                            column: self.current_column,
                        });
                    }
                }
                
                Token::EOF => break,
                Token::Illegal(ch) => {
                    return Err(ParseError {
                        message: format!("Invalid character: '{}'", ch),
                        line: self.current_line,
                        column: self.current_column,
                    });
                }
                _ => {
                    let token = self.advance().unwrap();
                    return Err(ParseError {
                        message: format!("Unexpected token: {:?}", token),
                        line: self.current_line,
                        column: self.current_column,
                    });
                }
            }
        }
        
        Ok(Program { statements })
    }
}