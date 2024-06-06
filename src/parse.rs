use time::Date;
use time::macros::format_description;

/// Separator between components (page boundaries, entries, page headers)
const COMPONENT_SEPARATOR: &str = "\n\n";

/// Page boundary marker
const PAGE_MARKER: &str = "-----";

/// Start of the preamble entry rage
const ENTRY_RANGE_START: &str = "Entries from ";

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
    source: &mut impl Iterator<Item = &'a str>,
    name: &str,
    location: &str
) -> Result<&'a str, String> {
    source.next().ok_or(format!("no {name} in {location}"))
}

/// Expect a value as the next item of an iterator, returning an error if it
/// doesn't match
fn expect_literal<'a>(
    source: &mut impl Iterator<Item = &'a str>,
    value: &str,
    name: &str,
) -> Result<(), String> {
    match source.next() {
        Some(next) if next == value => Ok(()),
        Some(other) => Err(format!("got {other} for {other}, expected {value}")),
        None => Err(format!("no {name}, expected {value}")),
    }
}
