use std::{str::CharIndices, borrow::Cow, collections::VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location<'filename> {
    pub filename: &'filename str,
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub file_text: Option<&'filename str>
}

pub struct LocationTrack<'file> {
    pub raw: &'file str,
    characters: CharIndices<'file>,
    last_reported_index: usize,
    name: &'file str,
    line: usize,
    column: usize,
    cached_next: Option<(usize, char)>,
}

pub struct LocationTrackOwned {
    pub raw: String,
    characters: VecDeque<(usize, char)>,
    last_reported_index: usize,
    name: &'static str,
    line: usize,
    column: usize,
    cached_next: Option<(usize, char)>,
}

pub trait LocationTracker<'file>: std::iter::Iterator<Item = (usize, Location<'file>, char)> {
    fn get_slice(&self, index: usize, length: usize) -> Cow<'file, str>;
    fn to_last_reported(&self, index: usize) -> Cow<'file, str>;
    fn consume_while(&mut self, predicate: impl Fn(char) -> bool);
    fn consume_if(&mut self, predicate: impl Fn(char) -> bool) -> bool;
    fn peek(&mut self) -> Option<(usize, char)>;
    fn consume(&mut self, data: (usize, char));

    fn raw(&self) -> &str;

    fn next_location(&self) -> Location<'file>;
}

impl<'filename> std::fmt::Display for Location<'filename> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {} in file {}", self.line + 1, self.column + 1, self.filename)
    }
}

impl<'file> LocationTrack<'file> {
    pub fn new<Data: Into<&'file str>>(data: Data, name: &'file str) -> Self {
        let s: &str = data.into();
        Self {
            cached_next: None,
            characters: s.char_indices(),
            last_reported_index: 0,
            raw: s,
            name,
            line: 0,
            column: 0
        }
    }
}

impl<'file> LocationTracker<'file> for LocationTrack<'file> {
    fn get_slice(&self, index: usize, length: usize) -> Cow<'file, str> {
        Cow::from(&self.raw[index..length])
    }

    fn to_last_reported(&self, index: usize) -> Cow<'file, str> {
        Cow::from(&self.raw[index..=self.last_reported_index])
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(data) = self.peek() {
            if !predicate(data.1) {
                return;
            }

            self.consume(data);
            self.cached_next.take();
        }
    }

    fn consume_if(&mut self, predicate: impl Fn(char) -> bool) -> bool {
        if let Some(data) = self.peek() {
            if !predicate(data.1) {
                return false;
            }

            self.consume(data);
            self.cached_next.take();
            true
        }
        else {
            false
        }
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        if self.cached_next.is_none() {
            self.cached_next = self.characters.next();
        }

        self.cached_next
    }

    fn consume(&mut self, data: (usize, char)) {
        match data.1 {
            '\n' => {
                self.line += 1;
                self.column = 0;
            },
            _ => {
                self.column += 1;
            }
        };

        self.last_reported_index = data.0;
    }

    fn raw(&self) -> &str {
        self.raw
    }

    fn next_location(&self) -> Location<'file> {
        Location { filename: self.name, line: self.line, column: self.column, index: self.last_reported_index + 1, file_text: Some(self.raw) }
    }
}

impl<'file> std::iter::Iterator for LocationTrack<'file> {
    type Item = (usize, Location<'file>, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.peek();

        self.cached_next.take().map( |(index, character)| {
            let location = Location { filename: self.name, line: self.line, column: self.column, index, file_text: Some(self.raw) };

            self.consume((index, character));

            (index, location, character)
        })
    }
}


impl LocationTrackOwned {
    pub fn new<Data: Into<String>>(data: Data, name: &'static str) -> Self {
        let s = data.into();
        Self {
            cached_next: None,
            raw: s.clone(),
            characters: s.char_indices().collect(),
            last_reported_index: 0,
            name,
            line: 0,
            column: 0
        }
    }
}
impl LocationTracker<'static> for LocationTrackOwned {
    fn get_slice(&self, index: usize, length: usize) -> Cow<'static, str> {
        Cow::from(self.raw[index..length].to_string())
    }

    fn to_last_reported(&self, index: usize) -> Cow<'static, str> {
        Cow::from(self.raw[index..=self.last_reported_index].to_string())
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(data) = self.peek() {
            if !predicate(data.1) {
                return;
            }

            self.consume(data);
            self.cached_next.take();
        }
    }

    fn consume_if(&mut self, predicate: impl Fn(char) -> bool) -> bool {
        if let Some(data) = self.peek() {
            if !predicate(data.1) {
                return false;
            }

            self.consume(data);
            self.cached_next.take();
            true
        }
        else {
            false
        }
    }

    fn peek(&mut self) -> Option<(usize, char)> {
        if self.cached_next.is_none() {
            self.cached_next = self.characters.pop_front();
        }

        self.cached_next
    }

    fn consume(&mut self, data: (usize, char)) {
        match data.1 {
            '\n' => {
                self.line += 1;
                self.column = 0;
            },
            _ => {
                self.column += 1;
            }
        };

        self.last_reported_index = data.0;
    }

    fn raw(&self) -> &str {
        &self.raw
    }

    fn next_location(&self) -> Location<'static> {
        Location { filename: self.name, line: self.line, column: self.column, index: self.last_reported_index + 1, file_text: None }
    }
}

impl std::iter::Iterator for LocationTrackOwned {
    type Item = (usize, Location<'static>, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.peek();

        self.cached_next.take().map( |(index, character)| {
            let location = Location { filename: self.name, line: self.line, column: self.column, index, file_text: None };

            self.consume((index, character));

            (index, location, character)
        })
    }
}