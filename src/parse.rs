
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
