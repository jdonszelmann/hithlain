use crate::parse::lexer::{TokenStream, Token, TokenIterator};
use crate::parse::span::Span;
use thiserror::Error;
use crate::parse::ast::{Program, Circuit, Variable, Statement, Expr, Constant, Atom, BinaryAction, Process, StatementOrTime, TimeSpec, NaryAction, Test, UnaryAction, Assignment};
use miette::{Diagnostic, SourceSpan, NamedSource};
use crate::time::{Duration, Instant};

#[derive(Error, Debug, Diagnostic)]
pub enum ParseError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    UnexpectedToken(#[from] UnexpectedToken),

    #[error(transparent)]
    #[diagnostic(transparent)]
    UnexpectedEnd(#[from] UnexpectedEnd),

    #[error(transparent)]
    #[diagnostic(transparent)]
    RightSideOfExpr(#[from] RightSideOfExpr),
}

#[derive(Error, Debug, Diagnostic)]
#[error("unexpected token, expected {}, found {}", expected, found)]
#[diagnostic()]
pub struct UnexpectedToken {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,

    expected: String,
    found: Token,
}


#[derive(Error, Debug, Diagnostic)]
#[error("invalid right side of expression")]
#[diagnostic()]
pub struct RightSideOfExpr {
    #[source_code]
    src: NamedSource,

    #[related]
    inner: Vec<ParseError>,

    #[label("after here")]
    span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("unexpected end of input, expected {}", expected)]
#[diagnostic()]
pub struct UnexpectedEnd {
    #[source_code]
    src: NamedSource,

    #[label("here")]
    span: SourceSpan,

    expected: String,
}

pub struct Parser {
    tokens: TokenIterator,
    previous_span: Option<Span>,
    current_span: Option<Span>,
    current_token: Option<Token>,
}

fn or_unexpected_end<T>(i: Option<T>, description: impl AsRef<str>, span: impl FnOnce() -> Span) -> Result<T, UnexpectedEnd> {
    if let Some(i) = i {
        Ok(i)
    } else {
        let span = span();

        return Err(UnexpectedEnd {
            expected: description.as_ref().to_string(),
            span: span.clone().into(),
            src: span.source().clone().into()
        })
    }
}


impl Parser {
    pub fn new(tokens: TokenStream) -> Self {
        Self {
            tokens: tokens.into_iter(),
            previous_span: None,
            current_span: None,
            current_token: None
        }
    }

    fn next(&mut self) -> Option<(Token, Span)> {
        self.previous_span = self.current_span.take();

        let n = self.tokens.next();

        if let Some((ref tok, ref spn)) = n {
            if self.previous_span.is_none() {
                self.previous_span = Some(spn.clone());
            }


            self.current_span = Some(spn.clone());
            self.current_token = Some(tok.clone());
        }

        n
    }

    fn peek(&mut self) -> Option<&(Token, Span)> {
        self.tokens.peek()
    }
    fn peek_2(&mut self) -> Option<[(Token, Span); 2]> {
        self.tokens.peek_2()
    }

    fn previous_span(&self) -> Span {
        self.previous_span.clone().expect("there to be a previous span")
    }

    fn current_span(&self) -> Span {
        self.current_span.clone().expect("there to be a current span")
    }

    fn current_token(&self) -> Token {
        self.current_token.clone().expect("there to be a current token")
    }


    fn allow_single_token(&mut self, t: &Token) {
        if let Some((tkn, _)) = self.peek() {
            if tkn == t {
                self.next();
            }
        }
    }

    fn expect_single_token(&mut self, t: &Token, description: Option<String>) -> Result<(), ParseError> {
        let mut spn = self.current_span().clone();
        let (tkn, t_spn) = or_unexpected_end(self.peek(), "circuit name", || spn.clone())?;
        spn = t_spn.clone();

        if tkn == t {
            self.next();
            return Ok(())
        }

        return Err(UnexpectedToken {
            expected: description.unwrap_or(format!("`{}`", t)),
            found: tkn.clone(),
            span: spn.clone().into(),
            src: spn.source().clone().into()
        }.into())
    }

    pub fn parse_variable(&mut self, description: Option<String>) -> Result<Variable, ParseError>  {
        let (tok, spn) = or_unexpected_end(self.peek().cloned(), "circuit name", || self.previous_span())?;

        if let Token::Name(name) = tok {
            let name = name.clone();
            self.next();
            Ok(Variable(name, Some(spn)))
        } else {
            return Err(UnexpectedToken {
                expected: description.unwrap_or("variable".to_string()),
                found: tok,
                span: spn.clone().into(),
                src: spn.source().clone().into()
            }.into())
        }
    }

    pub fn parse_constant(&mut self, description: Option<String>) -> Result<Constant, ParseError> {
        let (tok, spn) = or_unexpected_end(self.peek().cloned(), "circuit name", || self.previous_span())?;

        if let Token::Bit(value) = tok {
            let value = value.clone();
            self.next();
            Ok(Constant::Bit(value))
        } else {
            return Err(UnexpectedToken {
                expected: description.unwrap_or("variable".to_string()),
                found: tok,
                span: spn.clone().into(),
                src: spn.source().clone().into()
            }.into())
        }
    }

    pub fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        if let Ok(i) = self.parse_variable(None) {
            Ok(Expr::Atom(Atom::Variable(i)))
        } else {
            if let Ok(i) = self.parse_constant(None) {
                Ok(Expr::Atom(Atom::Constant(i)))
            } else {
                if let Some(i) = self.peek() {
                    if let (Token::LParen, _) = i {
                        self.next(); // opening paren
                        let expr = self.parse_expr()?;
                        let res = Atom::Expr(Box::new(expr));

                        self.expect_single_token(&Token::RParen, Some("closing parenthesis".to_string()))?;

                        return Ok(Expr::Atom(res))
                    }
                }
                return Err(UnexpectedToken {
                    expected: "variable, constant value or parenthesized expression".to_string(),
                    found: self.current_token().clone(),
                    span: self.current_span().into(),
                    src: self.current_span().source().clone().into()
                }.into())
            }
        }
    }

    pub fn parse_binary(&mut self) -> Result<Expr, ParseError> {
        let mut root = self.parse_atom()?;

        while let Some((tok, spn)) = self.peek().cloned() {
            let op = match tok {
                Token::And => BinaryAction::And,
                Token::Or => BinaryAction::Or,
                Token::Nand => BinaryAction::Nand,
                Token::Nor => BinaryAction::Nor,
                Token::Xor => BinaryAction::Xor,
                Token::Xnor => BinaryAction::Xnor,
                Token::Eq => BinaryAction::Xnor,
                _ => break,
            };

            // consume the operator
            self.next();

            let right_side = match self.parse_atom() {
                Ok(i) => i,
                Err(e) => return Err(RightSideOfExpr {
                    src: spn.source().clone().into(),
                    inner: vec![e],
                    span: spn.clone().into(),
                }.into()),
            };

            root = Expr::BinaryOp {
                a: Box::new(root),
                b: Box::new(right_side),
                action: op
            };
        }

        Ok(root)
    }

    pub fn parse_call(&mut self) -> Result<Expr, ParseError> {
        let circuit = self.parse_variable(Some("a circuit name to use".to_string()))?;
        self.expect_single_token(&Token::LParen, None)?;

        let mut params = vec![self.parse_atom()?];

        loop {
            let (tok, _) = or_unexpected_end(self.peek().cloned(), "statement or time specification", || self.previous_span())?;
            if tok == Token::Comma {
                self.next().unwrap();

                params.push(self.parse_atom()?)
            } else {
                break;
            }
        }

        self.expect_single_token(&Token::RParen, Some("closing parenthesis".to_string()))?;


        return Ok(Expr::NaryOp {
            params,
            action: NaryAction::Custom(circuit)
        })
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        if let Some([(tok1, _), (tok2, _)]) = self.peek_2() {
            if matches!(tok1, Token::Name(_)) && tok2 == Token::LParen {
                return self.parse_call()
            }
        }

        if let Some((tok, _)) = self.peek() {
            if tok == &Token::Not {
                self.next();
                self.expect_single_token(&Token::LParen, None)?;
                let param = self.parse_atom()?;
                self.expect_single_token(&Token::RParen, None)?;


                return Ok(Expr::NaryOp {
                    params: vec![param],
                    action: NaryAction::UnaryAction(
                        UnaryAction::Not
                    )
                })
            }
        }

        self.parse_binary()
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if let Some((tok, spn)) = self.peek() {
            let spn = spn.clone();
            if tok == &Token::Assert {
                self.next();
                let expr = self.parse_expr()?;
                self.expect_single_token(&Token::SemiColon, None)?;

                return Ok(Statement::Assert {
                    expr,
                    span: spn.merge_with(&self.current_span())
                })
            }
        }


        let mut vars = vec![self.parse_variable(Some("a variable to assign the expression outcome to".to_string()))?];

        loop {
            let (tok, _) = or_unexpected_end(self.peek().cloned(), "statement or time specification", || self.previous_span())?;
            if tok == Token::Comma {
                self.next().unwrap();

                vars.push(self.parse_variable(Some("another variable to assign the expression outcome to".to_string()))?)
            } else {
                break;
            }
        }


        self.expect_single_token(&Token::Assignment, None)?;

        let expr = self.parse_expr()?;

        self.expect_single_token(&Token::SemiColon, Some("`;` or binary operator".to_string()))?;


        Ok(Statement::Assignment(Assignment {
            into: vars,
            expr,
        }))
    }

    pub fn parse_statement_or_time(&mut self) -> Result<StatementOrTime, ParseError> {
        let (tok, spn) = or_unexpected_end(self.peek().cloned(), "statement or time specification", || self.previous_span())?;

        let tkn = match tok {
            Token::After => {
                self.next().unwrap();

                let spn = self.current_span().clone();
                let (tkn, _) = or_unexpected_end(self.peek(), "circuit name", || spn.clone())?;

                if let &Token::Time(a) = tkn {
                    self.next();

                    self.expect_single_token(&Token::Colon, None)?;

                    return Ok(StatementOrTime::Time(TimeSpec::After(Duration::from_nanos(a))))
                } else {
                    tkn
                }
            },
            Token::At => {
                self.next().unwrap();

                let spn = self.current_span().clone();
                let (tkn, _) = or_unexpected_end(self.peek(), "circuit name", || spn.clone())?;

                if let &Token::Time(a) = tkn {
                    self.next();

                    self.expect_single_token(&Token::Colon, None)?;

                    return Ok(StatementOrTime::Time(TimeSpec::At(Instant::nanos_from_start(a))))
                } else {
                    tkn
                }
            },
            _ => return Ok(StatementOrTime::Statement(self.parse_statement()?))
        };

        Err(UnexpectedToken {
            expected: "time".to_string(),
            found: tkn.clone(),
            span: spn.clone().into(),
            src: spn.source().clone().into()
        }.into())
    }

    pub fn parse_circuit(&mut self) -> Result<Circuit, ParseError> {
        let (_circuit, _) = self.next().unwrap();
        let name = self.parse_variable(Some("circuit name".to_string()))?;
        self.expect_single_token(&Token::Colon, Some(format!("`:` in definition of circuit {}", name.0)))?;

        let mut inputs = Vec::new();

        loop {
            let span = self.current_span();
            match or_unexpected_end(self.peek(), "circuit name", || span)? {
                (Token::Arrow, _) => {
                    self.next();
                    break;
                }
                (_, _) => inputs.push(self.parse_variable(Some("input name or ->".to_string()))?)
            }

            self.allow_single_token(&Token::Comma);
        }

        let mut outputs = Vec::new();

        loop {
            let span = self.current_span();
            match or_unexpected_end(self.peek(), "circuit name", || span)? {
                (Token::LBrace, _) => {
                    self.next();
                    break;
                }
                (_, _) => outputs.push(self.parse_variable(Some("output name or {".to_string()))?)
            }

            self.allow_single_token(&Token::Comma);
        }

        let mut body = Vec::new();

        loop {
            let span = self.current_span();
            match or_unexpected_end(self.peek(), "circuit name", || span)? {
                (Token::RBrace, _) => {
                    self.next();
                    break;
                }
                (_, _) => body.push(self.parse_statement()?),
            }
        }


        Ok(Circuit {
            name,
            inputs,
            outputs,
            body,
        })
    }


    pub fn parse_process(&mut self) -> Result<Process, ParseError> {
        let (_test, _) = self.next().unwrap();
        let name = self.parse_variable(Some("process name".to_string()))?;

        let mut body = Vec::new();

        self.expect_single_token(&Token::LBrace, None)?;

        loop {
            let span = self.current_span();
            match or_unexpected_end(self.peek(), "circuit name", || span)? {
                (Token::RBrace, _) => {
                    self.next();
                    break;
                }
                (_, _) => body.push(self.parse_statement_or_time()?),
            }
        }

        Ok(Process {
            name,
            inputs: vec![],
            outputs: vec![],
            body,
        })
    }

    pub fn parse_test(&mut self) -> Result<Test, ParseError> {
        let (_test, _) = self.next().unwrap();
        let name = self.parse_variable(Some("test name".to_string()))?;

        let mut body = Vec::new();

        self.expect_single_token(&Token::LBrace, None)?;

        loop {
            let span = self.current_span();
            match or_unexpected_end(self.peek(), "circuit name", || span)? {
                (Token::RBrace, _) => {
                    self.next();
                    break;
                }
                (_, _) => body.push(self.parse_statement_or_time()?),
            }
        }

        Ok(Test {
            name,
            body,
        })
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut circuits = Vec::new();
        let mut tests = Vec::new();
        let mut processes = Vec::new();
        while let Some((tok, spn)) = self.peek() {
            println!("{:?}", tok);
            match tok {
                Token::Circuit => {
                    circuits.push(self.parse_circuit()?)
                },
                Token::Test => {
                    tests.push(self.parse_test()?)
                },
                Token::Process => {
                    processes.push(self.parse_process()?)
                },
                Token::Error => unreachable!(),
                i => {
                    return Err(UnexpectedToken {
                        expected: "circuit definition".to_string(),
                        found: i.clone(),
                        span: spn.clone().into(),
                        src: spn.source().clone().into()
                    }.into())
                }
            }
        }

        Ok(Program {
            circuits,
            processes,
            tests,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::lexer::lex;
    use crate::error::NiceUnwrap;
    use crate::parse::source::Source;
    use crate::parse::parser::Parser;

    #[test]
    fn test_smoke() {
        let src = "
        circuit something: a b c -> d e {
            d = a and b;
            e = b or c;
        }

        test main {
            x, y = something(a, b, c);

            at 0ns:
                a = 1;
                b = 1;

                // assert x == 1

            after 5ns:
                a = 1;
                b = 0;

                // assert x == 0
        }
        ";

        let lexed = lex(Source::test(src)).nice_unwrap_panic();
        let mut parser = Parser::new(lexed);

        parser.parse_program().nice_unwrap_panic();
    }
}

