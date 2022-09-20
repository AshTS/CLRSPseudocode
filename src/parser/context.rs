use crate::{error::{GenericError, ErrorType}, tokenizer::{TokenStream, LocationTracker, Token, TokenData}};

use super::{ParseTreeNode};

pub struct ParserContext<'file, 'this, I: LocationTracker<'file>> {
    errors: Vec<GenericError<'file>>,
    failed: bool,
    token_stream: &'this mut TokenStream<'file, I>,
    current_indent: usize,
    indentation_stack: Vec<usize>,
}

impl<'file, 'this, I: LocationTracker<'file>> ParserContext<'file, 'this, I> {
    pub fn new(token_stream: &'this mut TokenStream<'file, I>) -> Self {
        Self {
            errors: Vec::new(),
            failed: false,
            token_stream,
            current_indent: 0,
            indentation_stack: vec![]
        }
    }

    pub fn add_error(&mut self, error: GenericError<'file>) {
        if error.error_type == ErrorType::Error {
            self.failed = true;
        }
        self.errors.push(error);
    }

    pub fn expect_token(&mut self) -> Option<Token<'file>> {
        if let Some(next_token) = self.token_stream.next() {
            if next_token.data == TokenData::EndOfFile {
                self.add_error(GenericError::error(next_token, "unexpected end of file while parsing".to_string()));
                None
            }
            else {
                Some(next_token)
            }
        }
        else {
            None
        }
    }

    pub fn consume_if(&mut self, predicate: impl Fn(&Token<'file>) -> bool) -> Option<Token<'file>> {
        if let Some(next_token) = self.token_stream.peek() {
            if predicate(next_token) {
                self.token_stream.next()
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    pub fn optional_consume_number(&mut self) -> Option<Token<'file>> {
        self.consume_if(|t| matches!(t.data, TokenData::NumericLiteral(_)))
    }

    pub fn optional_consume_identifier(&mut self) -> Option<Token<'file>> {
        self.consume_if(|t| matches!(t.data, TokenData::Identifier(_)))
    }

    pub fn enforce_consume_identifier(&mut self) -> Option<Token<'file>> {
        self.optional_consume_identifier().or_else(|| {
            let token = self.token_stream.peek()?.clone();
            let text = token.extract_text().to_string();
            self.add_error(GenericError::error(token, format!("expected identifier, got '{}'", text))
                                    .arrow("expected identifier".to_string()));
            None
        })
    }

    pub fn optional_consume_identifier_value(&mut self, identifier: &str) -> Option<Token<'file>> {
        self.consume_if(|t| if let TokenData::Identifier(s) = &t.data { s == identifier } else { false })
    }

    pub fn enforce_consume_identifier_value(&mut self, identifier: &str) -> Option<Token<'file>> {
        self.optional_consume_identifier().or_else(|| {
            let token = self.token_stream.peek()?.clone();
            let text = token.extract_text().to_string();
            self.add_error(GenericError::error(token, format!("expected keyword '{}', got '{}'", identifier, text))
                                    .arrow(format!("expected keyword '{}'", identifier)));
            None
        })
    }

    pub fn optional_consume_symbol(&mut self, symbol: &str) -> Option<Token<'file>> {
        self.consume_if(|t| if let TokenData::Symbol(s) = &t.data { s == symbol } else { false })
    }

    pub fn enforce_consume_symbol(&mut self, symbol: &str) -> Option<Token<'file>> {
        self.optional_consume_symbol(symbol).or_else(|| {
            let token = self.token_stream.peek()?.clone();
            let text = token.extract_text().to_string();
            self.add_error(GenericError::error(token, format!("expected symbol '{}', got '{}'", symbol, text))
                                .arrow(format!("expected symbol '{}'", symbol)));
            None
        })
    }

    pub fn optional_consume_consistent_indentation(&mut self) -> Option<Token<'file>> {
        let indent = self.current_indent;
        self.consume_if(|t| if let TokenData::Indentation(s) = &t.data { s.len() == indent } else { false })
    }

    pub fn enforce_consume_more_indentation(&mut self) -> Option<Token<'file>> {
        let indent = self.current_indent;
        self.consume_if(|t| if let TokenData::Indentation(s) = &t.data { s.len() > indent } else { false }).or_else( || {
            let token = self.expect_token()?;
            self.add_error(GenericError::error(token, "expected indented block".to_string())
                                .help("make sure blocks are denoted with further levels of indentation".to_string()));
            None
        }).map( |v| {
            self.indentation_stack.push(self.current_indent);
            self.current_indent = v.extract_text().len();
            v
        })
    }

    pub fn enforce_indent_or_less(&mut self) -> Option<Token<'file>> {
        self.optional_consume_consistent_indentation().or_else( || {
            let indent = self.current_indent;
            if let Some(token) = self.consume_if(|t| if let TokenData::Indentation(s) = &t.data { s.len() > indent } else { false }) {
                self.add_error(GenericError::error(token, "unexpected indentation level".to_string()));
            }
            None
        })
    }

    pub fn label_token(&mut self, token: Option<Token<'file>>) -> Option<Token<'file>> {
        let t = token?.clone();
        self.add_error(GenericError::info(t.clone(), "labeled token".to_string()));
        Some(t)
    }

    pub fn parse_expression(&mut self) -> Option<ParseTreeNode<'file>> {
        self.parse_assignment_expressions()
    }

    pub fn parse_statement(&mut self) -> Option<ParseTreeNode<'file>> {
        let token = self.token_stream.peek()?;
        {
            if token.extract_text() == "return" {
                let token = self.expect_token()?;
                let expression = Some(Box::new(self.parse_expression()?));

                Some(ParseTreeNode::ReturnStatement { token, expression })
            }
            else if token.extract_text() == "while" {
                let token = self.expect_token()?;
                let condition = Box::new(self.parse_expression()?);
                let block = Box::new(self.parse_block()?);

                Some(ParseTreeNode::WhileLoop { token, condition, block })
            }
            else if token.extract_text() == "for" {
                let token = self.expect_token()?;
                
                let loop_variable = self.enforce_consume_identifier()?;
                self.enforce_consume_symbol("=");
                let bound0 = Box::new(self.parse_expression()?);
                let reverse = self.optional_consume_identifier_value("down").is_some();
                self.enforce_consume_identifier_value("to")?;
                let bound1 = Box::new(self.parse_expression()?);

                let block = Box::new(self.parse_block()?);

                Some(ParseTreeNode::ForLoop { token, loop_variable, bound0, bound1, reverse, block })
            }
            else {
                self.parse_expression()
            }
        }
    }

    pub fn parse_block(&mut self) -> Option<ParseTreeNode<'file>> {
        self.enforce_consume_more_indentation()?;

        let mut statements = vec![];

        'outer: loop {
            let s = self.token_stream.peek().map(|v| v.extract_text().to_string());
            if s == Some("if".to_string()) {
                let token = self.expect_token()?;
                let condition = self.parse_expression()?;
                let block = self.parse_block()?;

                let mut else_ifs: Vec<(Token<'file>, ParseTreeNode, ParseTreeNode)> = vec![(token, condition, block)];

                loop {
                    // Consume the next indentation
                    if self.enforce_indent_or_less().is_none() {
                        self.current_indent = self.indentation_stack.pop().unwrap_or(0);
                        statements.push(ParseTreeNode::IfStatement { ifs: else_ifs, else_block: None });
                        break 'outer;
                    }

                    let t = self.token_stream.peek()?.extract_text().to_string();

                    if t == "elseif" {
                        let token = self.token_stream.next()?;
                        let condition = self.parse_expression()?;
                        let block = self.parse_block()?;
                        else_ifs.push((token, condition, block))
                    }
                    else if t == "else" {
                        let _ = self.token_stream.next()?;
                        let block = Some(Box::new(self.parse_block()?));

                        statements.push(ParseTreeNode::IfStatement { ifs: else_ifs, else_block: block });

                        // Consume the next indentation
                        if self.enforce_indent_or_less().is_none() {
                            self.current_indent = self.indentation_stack.pop().unwrap_or(0);
                            break 'outer;
                        }
                        break;
                    }
                    else {
                        statements.push(ParseTreeNode::IfStatement { ifs: else_ifs, else_block: None });
                        break;
                    }
                }
            }
            else {
                statements.push(self.parse_statement()?);
                if self.enforce_indent_or_less().is_none() {
                    self.current_indent = self.indentation_stack.pop().unwrap_or(0);
                    break 'outer;
                }
            }
        }

        Some(ParseTreeNode::Block { statements })
    }

    pub fn parse_function(&mut self) -> Option<ParseTreeNode<'file>> {
        let name = self.enforce_consume_identifier()?;
        self.enforce_consume_symbol("(");

        let mut arguments = vec![];

        if self.optional_consume_symbol(")").is_none() {
            loop {
                arguments.push(self.enforce_consume_identifier()?);
                if self.optional_consume_symbol(",").is_none() {
                    self.enforce_consume_symbol(")");
                    break;
                }
            }
        }

        let block = Box::new(self.parse_block()?);

        Some(ParseTreeNode::Function { name, arguments, block })
    }

    pub fn parse_document(&mut self) -> Result<(Vec<ParseTreeNode<'file>>, Vec<GenericError<'file>>), Vec<GenericError<'file>>> {
        let mut result = Vec::new();

        loop {
            if let Some(next) = self.token_stream.peek()
            {
                if next.data == TokenData::EndOfFile {
                    break;
                }
            }
            else {
                break;
            }

            if let Some(function) = self.parse_function() {
                result.push(function);
            }
            else {
                break;
            }
        }

        if self.failed {
            Err(std::mem::take(&mut self.errors))
        }
        else {
            Ok((result, std::mem::take(&mut self.errors)))
        }
    }
}