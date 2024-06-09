use std::path::PathBuf;
use time::{Date, Time, PrimitiveDateTime, Duration};
use time::macros::{format_description, time};
use crate::{Logbook, Mark, Entry, Block};

/// Parse a series of files
pub fn parse_files(files: Vec<PathBuf>) -> Result<Vec<Logbook>, String> {
    files.into_iter()
        .map(|file| std::fs::read_to_string(&file).map_err(
            |error| format!("Could not read file {}: {}", file.display(), error),
        ).map(|text| (file, text)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|(file, text)| parse(&text).map_err(
            |error| format!("{}\nwhile parsing file {}", error, file.display())
        ))
        .collect()
}

/// Parse a logbook
pub fn parse(logbook: &str) -> Result<Logbook, String> {
    // Extract the first page of the logbook
    let (preamble, entries) = logbook
        .split_once(&(PAGE_MARKER.to_owned() + COMPONENT_SEPARATOR))
        .ok_or("Could not parse preamble - no page boundry markers")?;
    // Parse the preamble
    let (start, end) = parse_preamble(preamble)
        .map_err(|error| format!("Could not parse preamble - {error}"))?;
    // Parse the entries
    let (entries, errors) = entries
        .split(COMPONENT_SEPARATOR)
        .fold(EntryParser::new(start.clone()), |parser, chunk| parser.advance(chunk))
        .finish(end.clone());
    // Print parsing errors
    for error in errors {
        eprintln!("Parsing error: {error}\n");
    }
    Ok(Logbook::new(start, end, entries))
}

/// Parse the preable of a logbook, extracting the start and end marks
fn parse_preamble(preable: &str) -> Result<(Mark, Option<Mark>), String> {
    // Extract components
    let mut words = preable
        .split_once(ENTRY_RANGE_START)
        .ok_or("no entry range found")?.1
        .split_ascii_whitespace();
    let start_date = expect_value(words.next(), "start date", "entry range")?;
    expect_literal(words.next(), ENTRY_RANGE_MARK_SEPARATOR, "entry range start separator")?;
    let start_number = expect_value(words.next(), "start entry number", "entry range")?;
    expect_literal(words.next(), ENTRY_RANGE_SEPARATOR, "entry range separator")?;
    let end_date = expect_value(words.next(), "end date", "entry range")?;
    expect_literal(words.next(), ENTRY_RANGE_MARK_SEPARATOR, "entry range end separator")?;
    let end_number = expect_value(words.next(), "end entry number", "entry range")?;
    // Parse and structure marks
    let start = Mark::new(
        parse_date(start_date, "start date")?,
        parse_number(start_number, "start entry number")?
    );
    let end = if end_number == ENTRY_RANGE_PLACEHOLDER {
        None
    } else {
        Some(Mark::new(
            parse_date(end_date, "end date")?,
            parse_number(end_number, "end entry number")?
        ))
    };
    Ok((start, end))
}

/// A state-based parser for entries (and page headers)
#[derive(Debug)]
struct EntryParser {
    /// The next expected entry position
    next_entry_position: Mark,
    // The entry currently being read
    current_entry: Entry,
    /// State flag for multi-page entries
    multi_page_flag: bool,
    /// Expectations imposed by the most recent page header
    page_header_expectations: PageHeaderExpectations,
    /// Previously-read entries
    read_entries: Vec<Entry>,
    /// Errors encountered during parsing
    errors: Vec<String>,
}

impl EntryParser {
    /// Create a new parser
    pub fn new(expected_start: Mark) -> Self {
        Self {
            next_entry_position: expected_start,
            current_entry: Entry::new(
                Mark::new(Date::MIN, 0),
                PrimitiveDateTime::MIN,
                PrimitiveDateTime::MIN,
                Vec::new()
            ),
            multi_page_flag: false,
            page_header_expectations: PageHeaderExpectations::NewHeader,
            read_entries: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Advance the parser over a chunk of input
    pub fn advance(mut self, chunk: &str) -> Self {
        let result = self.try_advance(chunk);
        self.record(result);
        self
    }

    /// Extract the parsed entries and any encountered errors
    pub fn finish(mut self, end: Option<Mark>) -> (Vec<Entry>, Vec<String>) {
        if let Some(end) = end {
            if let Some(last) = self.read_entries.last() {
                if *last.position() != end {
                    self.error(format!(
                        "logbook does not end with specified end date and/or entry number, expected {} {} but got {} {}",
                        end.effective_date(),
                        end.entry_number(),
                        last.position.effective_date(),
                        last.position.entry_number(),
                    ));
                }
            } else {
                self.error(
                    "an end entry was specified but the logbook contains no entries".to_string()
                );
            }
        }
        (self.read_entries, self.errors)
    }

    /// Attempt to advance the parser
    fn try_advance(&mut self, chunk: &str) -> Result<(), String> {
        if matches!(self.page_header_expectations, PageHeaderExpectations::NewHeader) {
            // The previous page just ended, read in the new header
            // We haven't looked at the new header yet, just assume
            // it's broken for now
            self.page_header_expectations = PageHeaderExpectations::Error;
            // Extract components
            let (number_range, date_range) = chunk
                .split_once(PAGE_RANGE_SPLIT)
                .ok_or("no separator between number and date ranges of page header")?;
            let (number_start, number_end) = number_range
                .split_once(PAGE_RANGE_SEPARATOR)
                .ok_or("no separator between start and end of entry number range of page header")?;
            let (date_start, date_end) = date_range
                .split_once(PAGE_RANGE_SEPARATOR)
                .ok_or("no separator between start and end of date range of page header")?;
            // Parse and structure values
            self.page_header_expectations = PageHeaderExpectations::StartAndEnd {
                start_recorded_date: parse_date(date_start, "page header start date")?,
                start_number: parse_number(number_start, "page header start number")?,
                end_recorded_date: parse_date(date_end, "page header end date")?,
                end_number: parse_number(number_end, "page header end number")?,
            }
        } else if chunk == PAGE_MARKER {
            // The current page is ending, check the page header's end against
            // the previous entry
            match self.page_header_expectations {
                PageHeaderExpectations::NewHeader =>
                    Err("expected page header, got page break"),
                PageHeaderExpectations::StartAndEnd { .. } =>
                    Err("expected at least one entry after page header, got page break"),
                PageHeaderExpectations::End {
                    end_recorded_date,
                    end_number,
                } => {
                    // Check the ending entry
                    if self.current_entry.recorded_date() != end_recorded_date {
                        self.error(format!(
                            "page header end date mismatch: header ends on {}, last entry is {}",
                            end_recorded_date,
                            self.current_entry.recorded_date(),
                        ));
                    }
                    if self.current_entry.position().entry_number() != end_number {
                        self.error(format!(
                            "page header end number mismatch: header ends on {}, last entry is {}",
                            end_number,
                            self.current_entry.position().entry_number(),
                        ));
                    }
                    // The next chunk should be the new page header
                    self.page_header_expectations = PageHeaderExpectations::NewHeader;
                    Ok(())
                }
                PageHeaderExpectations::Error => Ok(()),
            }?;
        } else {
            // This is some part of an entry
            // If this is the first entry of the page, get the expected entry
            // number and date from the page header
            let page_header_expectations = match self.page_header_expectations {
                PageHeaderExpectations::NewHeader =>
                    Err("expected page header, got entry"),
                PageHeaderExpectations::StartAndEnd {
                    start_recorded_date,
                    start_number,
                    end_recorded_date,
                    end_number,
                } => {
                    // We're checking the start entry, check the end entry next
                    self.page_header_expectations = PageHeaderExpectations::End {
                        end_recorded_date,
                        end_number,
                    };
                    Ok(Some((start_recorded_date, start_number)))
                }
                PageHeaderExpectations::End { .. } | PageHeaderExpectations::Error => Ok(None),
            }?;
            // Split the entry into lines
            let mut lines = chunk.lines();
            // Don't look for a header if we are continuing a mult-page entry
            if !self.multi_page_flag {
                // Extrapolate the next expected entry mark
                let expected_here = self.next_entry_position.clone();
                self.next_entry_position = Mark::new(
                    expected_here.effective_date() + Duration::DAY,
                    expected_here.entry_number() + 1
                );
                // Be generous and assume that this entry had the right position
                // when checking page headers if parsing fails
                self.current_entry.set_position(expected_here.clone());
                self.current_entry.set_started(PrimitiveDateTime::new(
                    expected_here.effective_date(),
                    time!(9 pm),
                ));
                // Get the header
                let mut header = lines.next().ok_or("entry has no header")?.split(" ");
                // Extract components
                self.record(expect_literal(header.next(), HEADER_START, "entry header start"));
                let number = expect_value(header.next(), "entry number", "entry header")?;
                let number = number
                    .strip_suffix(HEADER_POST_NUMBER)
                    .ok_or("missing separator between entry number and date in entry header")?;
                let start_date = expect_value(header.next(), "entry date", "entry header")?;
                self.record(expect_literal(header.next(), HEADER_STARTED, "entry header started"));
                let start_time = expect_value(header.next(), "entry started time", "entry header")?;
                let start_period = expect_value(header.next(), "entry started period", "entry header")?;
                self.record(expect_literal(header.next(), HEADER_FINISHED, "entry header finished"));
                let end_time = expect_value(header.next(), "entry finished time", "entry header")?;
                let end_period = expect_value(header.next(), "entry finished period", "entry header")?;
                if let Some(extra) = header.next() {
                    self.error(format!("Unexpected text after entry header: {extra}"));
                }
                // Parse and structure values
                let entry_number = parse_number(number, "entry number")?;
                let start_date = parse_date(start_date, "entry date")?;
                let start_time = parse_time(&format!("{} {}", start_time, start_period), "entry start time")?;
                let end_time = parse_time(&format!("{} {}", end_time, end_period), "entry start time")?;
                // Calculate end timestamp and effective entry date
                let start = PrimitiveDateTime::new(start_date, start_time);
                let end_date = if start_time < end_time {
                    // This entry was finished the same day it was started
                    start_date
                } else {
                    // This entry was finished the day after it was started
                    start_date + Duration::DAY
                };
                let end = PrimitiveDateTime::new(end_date, end_time);
                let date = if start.hour() > NEXT_DAY {
                    // Past the previous-day threshold, the effective and recorded
                    // dates are the same
                    start_date
                } else {
                    // Before the previous-day threshold, this is a past-midnight
                    // entry and has an effective date of the previous day
                    start_date - Duration::DAY
                };
                // Calculate entry position
                let position = Mark::new(date, entry_number);
                // Calculate the next expected entry position
                let expected_next = Mark::new(date + Duration::DAY, entry_number + 1);
                // Check that this entry has the expected position
                if expected_here.effective_date() != position.effective_date() {
                    if expected_here.effective_date() + Duration::DAY == position.effective_date() {
                        self.error(format!(
                            "Missing entry for {}",
                            expected_here.effective_date(),
                        ));
                    } else {
                        self.error(format!(
                            "Expected {} for effective date, got {}",
                            expected_here.effective_date(),
                            position.effective_date(),
                        ));
                    }
                }
                if expected_here.entry_number() != position.entry_number() {
                    self.error(format!(
                        "Expected {} for entry number, got {}",
                        expected_here.entry_number(),
                        position.entry_number(),
                    ));
                }
                // Update the next expected entry position. This should be the
                // same as the extrapolated version, but may be different in the
                // case of a missed entry
                self.next_entry_position = expected_next;
                // Start recording the new entry
                self.current_entry = Entry::new(position, start, end, Vec::new());
            }
            // Check the header constraints if present
            if let Some((date, number)) = page_header_expectations {
                if date != self.current_entry.recorded_date() {
                    self.error(format!(
                        "page header start date mismatch: header starts on {}, first entry is {}",
                        date,
                        self.current_entry.recorded_date()
                    ));
                }
                if number != self.current_entry.position().entry_number() {
                    self.error(format!(
                        "page header start number mismatch: header starts on {}, first entry is {}",
                        number,
                        self.current_entry.position().entry_number(),
                    ));
                }
            }
            // Parse the remaining lines of the entry
            for line in lines {
                if let Some(subject) = line.strip_prefix(SUBJECT_PREFIX) {
                    // This is the start of a new block
                    if self.multi_page_flag {
                        self.error("continuation of multi-page entry started with a new subject line instead of a continuation marker".to_owned());
                        self.multi_page_flag = false;
                    }
                    self.current_entry.contents_mut().push(Block::new(
                        subject.to_owned(), String::new()
                    ));
                } else {
                    // Check for a continuation marker
                    let line = if self.multi_page_flag {
                        if let Some(line) = line.strip_prefix(&format!(
                            "{} ", MULTI_PAGE
                        )) { line } else {
                            self.error("no continuation marker after multi-page split".to_owned());
                            line
                        }
                    } else { line };
                    self.multi_page_flag = false;
                    // Check for a new multi-page marker
                    let line = if let Some(line) = line.strip_suffix(&format!(
                        " {}", MULTI_PAGE
                    )) {
                        // This is a multi-page entry
                        self.multi_page_flag = true;
                        line
                    } else { line };
                    // Add the line to the current block
                    self.current_entry.contents_mut()
                        .last_mut().ok_or("no subject line")?
                        .text_mut().push_str(line);
                }
            }
            if !self.multi_page_flag {
                // Finish recording this entry
                self.read_entries.push(self.current_entry.clone());
            }
        }
        Ok(())
    }

    /// Record a potential error into the parser's memory
    fn record<T>(&mut self, result: Result<T, String>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(message) => {
                self.error(message);
                None
            }
        }
    }

    /// Record an error into the parser's memory
    fn error(&mut self, error: String) {
        self.errors.push(format!(
            "{error}\nnear entry {}",
            self.next_entry_position.entry_number()
        ));
    }
}

/// Expectations imposed by the most recent page header
#[derive(Debug)]
enum PageHeaderExpectations {
    /// The previous page was just finished, expect a new header
    NewHeader,
    /// The page header was just read, check the first entry next
    StartAndEnd {
        start_recorded_date: Date,
        start_number: u32,
        end_recorded_date: Date,
        end_number: u32,
    },
    /// The first entry on the page has been confirmed, check the last entry on
    /// page end
    End {
        end_recorded_date: Date,
        end_number: u32,
    },
    /// There was an error parsing the previous page header, there are no
    /// constraints to apply but a new page header should not
    /// be immidiately expected
    Error,
}

/// Separator between components (page boundaries, entries, page headers)
const COMPONENT_SEPARATOR: &str = "\n\n";

/// Page boundary marker
const PAGE_MARKER: &str = "-----";

/// Page header split between date and number range
const PAGE_RANGE_SPLIT: &str = "\n";

/// Page header date and number separator
const PAGE_RANGE_SEPARATOR: &str = "-";

/// Entry header start
const HEADER_START: &str = "Entry";

/// Separator between entry number and date in entry header
const HEADER_POST_NUMBER: &str = ":";

/// Separator between entry date and start time in entry header
const HEADER_STARTED: &str = "started";

/// Separator between entry start and finished time in entry header
const HEADER_FINISHED: &str = "finished";

/// Prefix for the subject lines of an entry
const SUBJECT_PREFIX: &str = "    ";

/// Marker for multi-page entries
const MULTI_PAGE: &str = "(->)";

/// Start of the preamble entry rage
const ENTRY_RANGE_START: &str = "Entries from";

/// Separator between the date and entry number of the preamble entry range
const ENTRY_RANGE_MARK_SEPARATOR: &str = "-";

/// Separator between the start and end marks of the preamble entry range
const ENTRY_RANGE_SEPARATOR: &str = "to";

/// Placeholder for the ending entry number of the preamble entry range
const ENTRY_RANGE_PLACEHOLDER: &str = "_";

/// Hour before which an entry will be considered as being written on the
/// previous day
const NEXT_DAY: u8 = 6;

/// Parse a string into a date
fn parse_date(string: &str, name: &str) -> Result<Date, String> {
    Date::parse(
        string,
        format_description!("[month padding:none]/[day padding:none]/[year]")
    ).map_err(|error| format!("bad {name} - {error}"))
}

/// Parse a string into a time
fn parse_time(string: &str, name: &str) -> Result<Time, String> {
    Time::parse(
        string,
        format_description!("[hour padding:none repr:12]:[minute padding:none] [period]")
    ).map_err(|error| format!("bad {name} - {error}"))
}

/// Parse a string into a number
fn parse_number(string: &str, name: &str) -> Result<u32, String> {
    u32::from_str_radix(string, 10)
        .map_err(|error| format!("bad {name} - {error}"))
}

/// Return the next item of an iterator, or an error if there is none
fn expect_value<'a>(
    source: Option<&'a str>,
    name: &str,
    location: &str
) -> Result<&'a str, String> {
    source.ok_or(format!("no {name} in {location}"))
}

/// Expect a value as the next item of an iterator, returning an error if it
/// doesn't match
fn expect_literal<'a>(
    source: Option<&'a str>,
    value: &str,
    name: &str,
) -> Result<(), String> {
    match source {
        Some(next) if next == value => Ok(()),
        Some(other) => Err(format!("got {other} for {name}, expected {value}")),
        None => Err(format!("no {name}, expected {value}")),
    }
}
