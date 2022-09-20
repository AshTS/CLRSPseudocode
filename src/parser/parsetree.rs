use crate::tokenizer::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionType {
    Assignment,
    Add,
    Subtract,
    Multiply,
    Divide,
    MemberAccess,
    Indexing,
    LogicalOr,
    LogicalAnd,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Equality,
    Inequality,
    FunctionCall
}

#[derive(Debug, Clone)]
pub enum ParseTreeNode<'file> {
    Function{name: Token<'file>, arguments: Vec<Token<'file>>, block: Box<ParseTreeNode<'file>>},
    Block{statements: Vec<ParseTreeNode<'file>>},
    ReturnStatement{token: Token<'file>, expression: Option<Box<ParseTreeNode<'file>>>},
    IdentifierValue{token: Token<'file>},
    NumericValue{token: Token<'file>, value: f64},
    IfStatement{ifs: Vec<(Token<'file>, ParseTreeNode<'file>, ParseTreeNode<'file>)>, else_block: Option<Box<ParseTreeNode<'file>>> },
    ForLoop{token: Token<'file>, loop_variable: Token<'file>, bound0: Box<ParseTreeNode<'file>>, bound1: Box<ParseTreeNode<'file>>, reverse: bool, block: Box<ParseTreeNode<'file>> },
    WhileLoop{token: Token<'file>, condition: Box<ParseTreeNode<'file>>, block: Box<ParseTreeNode<'file>>},
    Expression{expression_type: ExpressionType, symbols: Vec<Token<'file>>, children: Vec<ParseTreeNode<'file>>}
}

impl<'file> ParseTreeNode<'file> {
    pub fn get_token(&self) -> &Token<'file> {
        match self {
            ParseTreeNode::Function { name, .. } => name,
            ParseTreeNode::Block { statements } => statements[0].get_token(),
            ParseTreeNode::ReturnStatement { token, .. } => token,
            ParseTreeNode::IdentifierValue { token } => token,
            ParseTreeNode::NumericValue { token, .. } => token,
            ParseTreeNode::IfStatement { ifs, .. } => &ifs[0].0,
            ParseTreeNode::ForLoop { token, .. } => token,
            ParseTreeNode::WhileLoop { token, .. } => token,
            ParseTreeNode::Expression { symbols, .. } => &symbols[0],
        }
    }
}