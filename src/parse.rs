
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
