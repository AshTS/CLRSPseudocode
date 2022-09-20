use crate::{error::GenericError, tokenizer::Token};

#[derive(Debug)]
pub enum RuntimeError<'file> {
    FinishedError(GenericError<'file>),
    ArgumentCountError{ expected: usize, got: usize },
    MessageError(String)
}

impl<'file> RuntimeError<'file> {
    pub fn finish(self, token: Token<'file>) -> GenericError<'file> {
        let text = token.extract_text().to_string();
        match self {
            RuntimeError::FinishedError(error) => error,
            RuntimeError::ArgumentCountError{ expected, got } => {
                let msg = format!("expected {} argument{}, got {} argument{}", expected, if expected == 1 { "" } else { "s" }, got, if got == 1 { "" } else { "s" });
                GenericError::error(token, format!("function '{}' {}", text, msg)).arrow(msg)
            },
            RuntimeError::MessageError(msg) => {
                GenericError::error(token, msg.clone()).arrow(msg)
            }
        }
    }

    pub fn finish_no_token(self) -> GenericError<'file> {
        match self {
            RuntimeError::FinishedError(error) => error,
            RuntimeError::ArgumentCountError{ expected, got } => {
                let msg = format!("expected {} argument{}, got {} argument{}", expected, if expected == 1 { "" } else { "s" }, got, if got == 1 { "" } else { "s" });
                GenericError::tokenless_error(msg.to_string()).arrow(msg)
            },
            RuntimeError::MessageError(msg) => {
                GenericError::tokenless_error(msg.clone()).arrow(msg)
            }
        }
    }

    pub fn finish_maybe(self, token: Option<Token<'file>>) -> GenericError<'file> {
        if let Some(token) = token {
            self.finish(token)
        }
        else {
            self.finish_no_token()
        }
    }
}

impl<'file> std::convert::From<GenericError<'file>> for RuntimeError<'file> {
    fn from(v: GenericError<'file>) -> Self {
        Self::FinishedError(v)
    }
}