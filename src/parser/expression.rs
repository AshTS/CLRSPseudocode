use crate::{tokenizer::LocationTracker, error::GenericError};

use super::{ParserContext, ParseTreeNode, ExpressionType};

impl<'file, 'this, I: LocationTracker<'file>> ParserContext<'file, 'this, I> {
    
    pub fn parse_value(&mut self) -> Option<ParseTreeNode<'file>> {
        if let Some(identifier_token) = self.optional_consume_identifier() {
            Some(ParseTreeNode::IdentifierValue { token: identifier_token })
        }
        else if let Some(numeric_token) = self.optional_consume_number() {
            if let Ok(value) = numeric_token.extract_text().parse::<f64>() {
                Some(ParseTreeNode::NumericValue { token: numeric_token, value })
            }
            else {
                let text = numeric_token.extract_text().to_string();
                self.add_error(GenericError::error(numeric_token.clone(), format!("unable to parse number from '{}'", text)));

                Some(ParseTreeNode::NumericValue { token: numeric_token, value: 0.0 })
            }
        }
        else if self.optional_consume_symbol("(").is_some() {
            let value = self.parse_expression();

            self.enforce_consume_symbol(")");

            value
        }
        else if let Some(token) = self.expect_token() {
            let text = token.extract_text().to_string();
            self.add_error(GenericError::error(token, format!("expected value, got '{}'", text)).arrow("expected value".to_string()).help("a value can be any of the following:\n  a numeric literal\n  an identifier".to_string()));

            None
        }
        else {
            None
        }
    }

    pub fn parse_postfix_expression(&mut self) -> Option<ParseTreeNode<'file>> {
        let mut inner = self.parse_value()?;

        loop {
            if let Some(symbol) = self.optional_consume_symbol(".") {
                inner = ParseTreeNode::Expression { expression_type: ExpressionType::MemberAccess, symbols: vec![symbol], children: vec![inner, self.parse_value()?] };
            }
            else if let Some(symbol) = self.optional_consume_symbol("[") {
                let index = self.parse_expression()?;

                let symbol1 = self.enforce_consume_symbol("]")?;

                inner = ParseTreeNode::Expression { expression_type: ExpressionType::Indexing, symbols: vec![symbol, symbol1], children: vec![inner, index] }
            }
            else if let Some(symbol) = self.optional_consume_symbol("(") {
                let mut children = vec![inner];

                loop {
                    children.push(self.parse_expression()?);

                    if self.optional_consume_symbol(",").is_none() {
                        break;
                    }
                }

                let symbol1 = self.enforce_consume_symbol(")")?;

                inner = ParseTreeNode::Expression { expression_type: ExpressionType::FunctionCall, symbols: vec![symbol, symbol1], children }
            }
            else {
                break;
            }
        }
        
        Some(inner)
    }

    pub fn parse_multiplicative_expressions(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_postfix_expression()?;

        if let Some(symbol) = self.optional_consume_symbol("*") {
            let right = self.parse_multiplicative_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Multiply, symbols: vec![symbol], children: vec![left, right] })
        }
        else if let Some(symbol) = self.optional_consume_symbol("/") {
            let right = self.parse_multiplicative_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Divide, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }

    pub fn parse_additive_expressions(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_multiplicative_expressions()?;

        if let Some(symbol) = self.optional_consume_symbol("+") {
            let right = self.parse_additive_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Add, symbols: vec![symbol], children: vec![left, right] })
        }
        else if let Some(symbol) = self.optional_consume_symbol("-") {
            let right = self.parse_additive_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Subtract, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }

    pub fn parse_comparison_expressions(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_additive_expressions()?;

        if let Some(symbol) = self.optional_consume_symbol("<") {
            let right = self.parse_comparison_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::LessThan, symbols: vec![symbol], children: vec![left, right] })
        }
        else if let Some(symbol) = self.optional_consume_symbol(">") {
            let right = self.parse_comparison_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::GreaterThan, symbols: vec![symbol], children: vec![left, right] })
        }
        else if let Some(symbol) = self.optional_consume_symbol("<=") {
            let right = self.parse_comparison_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::LessThanEqual, symbols: vec![symbol], children: vec![left, right] })
        }
        else if let Some(symbol) = self.optional_consume_symbol(">=") {
            let right = self.parse_comparison_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::GreaterThanEqual, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }

    pub fn parse_equality_expressions(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_comparison_expressions()?;

        if let Some(symbol) = self.optional_consume_symbol("==") {
            let right = self.parse_equality_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Equality, symbols: vec![symbol], children: vec![left, right] })
        }
        else if let Some(symbol) = self.optional_consume_symbol("!=") {
            let right = self.parse_equality_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Inequality, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }

    pub fn parse_logical_and_expression(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_equality_expressions()?;

        if let Some(symbol) = self.optional_consume_identifier_value("and") {
            let right = self.parse_logical_and_expression()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::LogicalAnd, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }

    pub fn parse_logical_or_expression(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_logical_and_expression()?;

        if let Some(symbol) = self.optional_consume_identifier_value("or") {
            let right = self.parse_logical_or_expression()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::LogicalOr, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }

    pub fn parse_assignment_expressions(&mut self) -> Option<ParseTreeNode<'file>> {
        let left = self.parse_logical_or_expression()?;

        if let Some(symbol) = self.optional_consume_symbol("=") {
            let right = self.parse_assignment_expressions()?;

            Some(ParseTreeNode::Expression { expression_type: ExpressionType::Assignment, symbols: vec![symbol], children: vec![left, right] })
        }
        else {
            Some(left)
        }
    }
}