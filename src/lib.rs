use time::{Date, PrimitiveDateTime};

/// A logbook of journal entries
pub struct Logbook {
    // The start position of this logbook
    start: Mark,
    // The end position (position of the last entry) of this logbook. None if
    // this logbook is still being written
    end: Option<Mark>,
    // The entries in this logbook
    entries: Vec<Entry>,
}

/// A position within a sequence of entries
pub struct Mark {
    /// The date for which the entry was written
    effective_date: Date,
    // The entry number of the entry
    entry_number: u32,
}

/// An entry in a logbook
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
pub struct Block {
    /// The subject of this block
    subject: String,
    /// The main text of this block
    text: String,
}
