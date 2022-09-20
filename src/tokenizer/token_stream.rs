use super::LocationTrack;
use super::LocationTrackOwned;
use super::LocationTracker;
use super::Token;
use super::TokenData;

pub struct TokenStream<'file, I: LocationTracker<'file>> {
    pub location_stream: I,
    cached_next_token: Option<Token<'file>>,
    sent_eof: bool
}

impl TokenStream<'static, LocationTrackOwned> {
    pub fn from_source_owned<Data: Into<String>>(data: Data, name: &'static str) -> TokenStream<'static, LocationTrackOwned> {
        Self {
            location_stream: LocationTrackOwned::new(data, name),
            cached_next_token: None,
            sent_eof: false
        }
    }
}

impl<'file> TokenStream<'file, LocationTrack<'file>> {
    pub fn from_source<Data: Into<&'file str>>(data: Data, name: &'file str) -> Self {
        Self::new(LocationTrack::new(data, name))
    }
}

impl<'file, I: LocationTracker<'file>> TokenStream<'file, I> {
    pub fn new(location_stream: I) -> Self {
        Self {
            location_stream,
            cached_next_token: None,
            sent_eof: false
        }
    }

    fn consume_while_identifier(&mut self) {
        self.location_stream.consume_if(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_'));
        self.location_stream.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'));
    }

    fn consume_while_number(&mut self) {
        self.location_stream.consume_while(|c| matches!(c, '0'..='9'));
        self.location_stream.consume_if(|c| c == '.');
        self.location_stream.consume_while(|c| matches!(c, '0'..='9'));
    }

    fn consume_compound_token(&mut self, c: char) {
        match c {
            '=' => {
                self.location_stream.consume_if(|c| c == '=');
            }
            '<' => {
                self.location_stream.consume_if(|c| c == '=');
            }
            '>' => {
                self.location_stream.consume_if(|c| c == '=');
            }
            '!' => {
                self.location_stream.consume_if(|c| c == '=');
            }
            _ => {eprintln!("{}", c); todo!() }
        }
    }

    pub fn peek(&mut self) -> Option<&Token<'file>> {
        if self.cached_next_token.is_none() {
            self.cached_next_token = 
            if let Some((index, location, c)) = self.location_stream.next() {
                match c {
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.consume_while_identifier();
                        Some(Token::new(location, TokenData::Identifier(self.location_stream.to_last_reported(index))))
                    },
                    '0'..='9' => {
                        self.consume_while_number();
                        Some(Token::new(location, TokenData::NumericLiteral(self.location_stream.to_last_reported(index))))
                    }
                    // Comments
                    '/' => {
                        if self.location_stream.consume_if(|c| c == '/') {
                            self.location_stream.consume_while(|c| c != '\n');
                            self.next()
                        }
                        else {
                            Some(Token::new(location, TokenData::Symbol(self.location_stream.to_last_reported(index))))
                        }
                    }
                    // Compound Symbols
                    '!' | '<' | '>' | '=' => {
                        self.consume_compound_token(c);
                        Some(Token::new(location, TokenData::Symbol(self.location_stream.to_last_reported(index))))
                    }
                    '(' | ')' | '[' | ']' | '.' | ',' | '+' | '-' | '*' => Some(Token::new(location, TokenData::Symbol(self.location_stream.to_last_reported(index)))),
                    ' ' | '\r' => self.next(),
                    '\n' => {
                        let mut location = location;
                        let mut start_index = index + 1;
                        loop {
                            if let Some((_, ' ')) = self.location_stream.peek() {
                                location = self.location_stream.next().unwrap().1;
                            }
                            self.location_stream.consume_while(|c| c == ' ' || c == '\r');
                            if let Some((index, '\n')) = self.location_stream.peek() {
                                start_index = index + 1;
                                self.location_stream.next();
                            }
                            else {
                                break;
                            }
                        }

                        let s = self.location_stream.to_last_reported(start_index);

                        if s.len() == 0 {
                            self.next()
                        }
                        else {
                            Some(Token::new(location, TokenData::Indentation(self.location_stream.to_last_reported(start_index))))
                        }
                    }
                    _ => {
                        println!("Character: {}", c);
                        todo!()
                    }
                }
            }
            else if !self.sent_eof {
                self.sent_eof = true;

                let location = self.location_stream.next_location();

                Some(Token::new(location, TokenData::EndOfFile))
            }
            else {
                None
            };
        }

        self.cached_next_token.as_ref()
    }
}

impl<'file, I: LocationTracker<'file>> std::iter::Iterator for TokenStream<'file, I> {
    type Item = Token<'file>;

    fn next(&mut self) -> Option<Self::Item> {
        self.peek();
        self.cached_next_token.take()
    }
}