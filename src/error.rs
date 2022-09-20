use crate::tokenizer::Token;

const CLEAR: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    Error,
    Warning,
    Info
}

impl ErrorType {
    pub fn to_str(&self) -> &'static str {
        match self {
            ErrorType::Error => "error",
            ErrorType::Warning => "warning",
            ErrorType::Info => "info",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ErrorType::Error => RED,
            ErrorType::Warning => YELLOW,
            ErrorType::Info => CYAN,
        }
    }
}

#[derive(Debug)]
pub struct GenericError<'file> {
    token: Option<Token<'file>>,
    pub error_type: ErrorType,
    message: String,
    help: Option<String>,
    arrow_note: Option<String>
}

impl<'file> GenericError<'file> {
    pub fn error(token: Token<'file>, message: String) -> Self {
        Self {
            error_type: ErrorType::Error,
            token: Some(token), message, help: None, arrow_note: None
        }
    }

    pub fn warning(token: Token<'file>, message: String) -> Self {
        Self {
            error_type: ErrorType::Warning,
            token: Some(token), message, help: None, arrow_note: None
        }
    }

    pub fn info(token: Token<'file>, message: String) -> Self {
        Self {
            error_type: ErrorType::Info,
            token: Some(token), message, help: None, arrow_note: None
        }
    }

    pub fn tokenless_error(message: String) -> Self {
        Self {
            error_type: ErrorType::Error,
            token: None, message, help: None, arrow_note: None
        }
    }

    pub fn tokenless_warning(message: String) -> Self {
        Self {
            error_type: ErrorType::Warning,
            token: None, message, help: None, arrow_note: None
        }
    }

    pub fn tokenless_info(message: String) -> Self {
        Self {
            error_type: ErrorType::Info,
            token: None, message, help: None, arrow_note: None
        }
    }

    pub fn tokenable_error(token: Option<Token<'file>>, message: String) -> Self {
        Self {
            error_type: ErrorType::Error,
            token, message, help: None, arrow_note: None
        }
    }

    pub fn tokenable_warning(token: Option<Token<'file>>, message: String) -> Self {
        Self {
            error_type: ErrorType::Warning,
            token, message, help: None, arrow_note: None
        }
    }

    pub fn tokenable_info(token: Option<Token<'file>>, message: String) -> Self {
        Self {
            error_type: ErrorType::Info,
            token, message, help: None, arrow_note: None
        }
    }

    pub fn help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }

    pub fn arrow(mut self, arrow_note: String) -> Self {
        self.arrow_note = Some(arrow_note);
        self
    }
}

impl<'file> std::fmt::Display for GenericError<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}{}{}: {}{}", self.error_type.color(), self.error_type.to_str(), WHITE, self.message, CLEAR)?;
        if let Some(token) = &self.token {
            let location = token.location.clone();
            
            writeln!(f, "  {}-->{} {}:{}:{}", CYAN, CLEAR, location.filename, location.line + 1, location.column + 1)?;
            if let Some(raw) = location.file_text {
                let error_line = location.line;
                let mut index_offset = 0;
                writeln!(f, "    {}|", CYAN)?;
                for (i, line) in raw.split('\n').enumerate() {
                    if i == error_line {
                        if i == error_line {
                            writeln!(f, "{:<4}|{} {}{}", i + 1, CLEAR, line, CYAN)?;
                            write!(f, "    | {}", self.error_type.color())?;
                            for _ in 0..(location.index - index_offset) {
                                write!(f, " ")?;
                            }
                            for _ in token.extract_text().chars() {
                                write!(f, "^")?;
                            }
                            if let Some(arrow_note) = &self.arrow_note {
                                write!(f, " {}", arrow_note)?;
                            }
                            writeln!(f, "{}", CYAN)?;
                        }
                        else {
                            writeln!(f, "    |{} {}{}", CLEAR, line, CYAN)?;
                        }
                    }
                    index_offset += line.len() + 1;
                }
            }
        }
        

        if let Some(help) = &self.help {
            for line in help.lines() {
                writeln!(f, "    {}= {}help: {}{}", CYAN, WHITE, CLEAR, line)?;
            }
        }
        write!(f, "{}", CLEAR)?;

        Ok(())
    }
}