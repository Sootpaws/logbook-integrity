use time::Date;
use time::macros::format_description;
use crate::{Logbook, Mark};

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

/// Separator between components (page boundaries, entries, page headers)
const COMPONENT_SEPARATOR: &str = "\n\n";

/// Page boundary marker
const PAGE_MARKER: &str = "-----";

/// Start of the preamble entry rage
const ENTRY_RANGE_START: &str = "Entries from";

/// Separator between the date and entry number of the preamble entry range
const ENTRY_RANGE_MARK_SEPARATOR: &str = "-";

/// Separator between the start and end marks of the preamble entry range
const ENTRY_RANGE_SEPARATOR: &str = "to";

/// Placeholder for the ending entry number of the preamble entry range
const ENTRY_RANGE_PLACEHOLDER: &str = "_";

/// Parse a string into a date
fn parse_date(string: &str, name: &str) -> Result<Date, String> {
    Date::parse(
        string,
        format_description!("[month padding:none]/[day padding:none]/[year]")
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
