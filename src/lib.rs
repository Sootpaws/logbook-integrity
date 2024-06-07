use time::{Date, PrimitiveDateTime};

pub mod parse;

/// A logbook of journal entries
#[derive(Debug)]
pub struct Logbook {
    /// The start position of this logbook
    start: Mark,
    /// The end position (position of the last entry) of this logbook. None if
    /// this logbook is still being written
    end: Option<Mark>,
    /// The entries in this logbook
    entries: Vec<Entry>,
}

/// A position within a sequence of entries
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    /// The date for which the entry was written
    effective_date: Date,
    /// The entry number of the entry
    entry_number: u32,
}

/// An entry in a logbook
#[derive(Debug)]
pub struct Entry {
    /// The position of this entry
    position: Mark,
    /// The starting timestamp for this entry
    started: PrimitiveDateTime,
    /// The ending timestamp for this entry
    finished: PrimitiveDateTime,
    /// The body of this entry
    contents: Vec<Block>,
}

/// One block of an entry's body
#[derive(Debug)]
pub struct Block {
    /// The subject of this block
    subject: String,
    /// The main text of this block
    text: String,
}

impl Logbook {
    pub fn new(start: Mark, end: Option<Mark>, entries: Vec<Entry>) -> Self {
        Self { start, end, entries }
    }
}

impl Mark {
    pub fn new(effective_date: Date, entry_number: u32) -> Self {
        Self { effective_date, entry_number }
    }
    pub fn effective_date(&self) -> Date {
        self.effective_date
    }
    pub fn entry_number(&self) -> u32 {
        self.entry_number
    }
}

impl Entry {
    pub fn new(
        position: Mark,
        started: PrimitiveDateTime,
        finished: PrimitiveDateTime,
        contents: Vec<Block>
    ) -> Self {
        Self { position, started, finished, contents }
    }
    pub fn position(&self) -> &Mark {
        &self.position
    }
    pub fn started(&self) -> PrimitiveDateTime {
        self.started
    }
    pub fn finished(&self) -> PrimitiveDateTime {
        self.finished
    }
    pub fn contents(&self) -> &Vec<Block> {
        &self.contents
    }
    pub fn contents_mut(&mut self) -> &mut Vec<Block> {
        &mut self.contents
    }
    /// Get the recorded date of this entry
    pub fn recorded_date(&self) -> Date {
        self.started.date()
    }
}

impl Block {
    pub fn new(subject: String, text: String) -> Self {
        Self { subject, text }
    }
    pub fn subject(&self) -> &String {
        &self.subject
    }
    pub fn text(&self) -> &String {
        &self.text
    }
    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }
}
