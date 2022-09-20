use super::Location;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenData<'filedata> {
    Identifier(Cow<'filedata, str>),
    NumericLiteral(Cow<'filedata, str>),
    Symbol(Cow<'filedata, str>),
    Indentation(Cow<'filedata, str>),
    EndOfFile
}

#[derive(Clone, PartialEq, Eq)]
pub struct Token<'file> {
    pub location: Location<'file>,
    pub data: TokenData<'file>
}

impl<'filedata> std::fmt::Display for TokenData<'filedata> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            TokenData::NumericLiteral(command) => write!(f, "number {}", command),
            TokenData::Symbol(symbol) => write!(f, "symbol {}", symbol),
            TokenData::Identifier(identifier) => write!(f, "identifier {}", identifier),
            TokenData::Indentation(indentation) => write!(f, "indentation {}", indentation),
            TokenData::EndOfFile=> write!(f, "eof"),
        }
    }
}

impl<'file> TokenData<'file> {
    pub fn extract_text(&self) -> &Cow<str> {
        match &self {
            TokenData::NumericLiteral(literal) => literal,
            TokenData::Symbol(symbol) => symbol,
            TokenData::Identifier(identifier) => identifier,
            TokenData::Indentation(indentation) => indentation,
            TokenData::EndOfFile=> &Cow::Borrowed(" "),
        }
    }
}

impl<'file> Token<'file> {
    pub fn new(location: Location<'file>, data: TokenData<'file>) -> Self {
        Self {
            location, data
        }
    }

    pub fn extract_text(&self) -> &Cow<str> {
        self.data.extract_text()
    }
}

impl<'file> std::fmt::Display for Token<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.data, self.location)
    }
}

impl<'file> std::fmt::Debug for Token<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token('{}')", self.data)
    }
}