## Overview
`logbook-integrity` is a small command-line tool that parses plaintext journals
and checks for missing entries and other errors.

## Features
- Parses logbook entries into simple Rust data structures
- Robust error recovery while parsing - one typo won't prevent the entire file
from being parsed

## Planned additions
- Generate some interesting statistics - average length and time, subjects, etc
- Use the content of parsed entries to create a Markov chain

## Formatting
```
Each logbook starts with a preamble page that specifies the range of entries
present, both date and number (although the end date and number may be left as
_ - _ if unknown):

Entries from 1/1/2000 - 1 to 1/3/2000 - 3

This may be preceeded and/or followed by other text (such as this). Page breaks
are represented as a series of 5 dashes, and each new page starts with the date
and entry number range of that page.

-----

1-2
1/1/2000-1/3/2000

Entry 1: 1/1/2000 started 8:00 PM finished 8:10 PM
    Entries/Format
Each entry starts with a header that lists the entry number, date written, and
time started and finished. Following that is a subject line, and then the main
text of the entry.
    Entries/Blocks
An entry can have multiple "blocks" (subject + body), for when you have a lot on
your mind.

Entry 2: 1/3/2000 started 1:00 AM finished 1:10 AM
    Entries/Time
The date an entry was written may not be the date it was written for. Entries
written very early in the morning (before 6 AM) will count for the previous day.
If you run out of space on a page, an entry can be extended onto the next page
using a continuation marker (->)

-----

2-3
1/3/2000-1/3/2000

(->) where there's more space. Note that the continued entry affects new page's
range.

Entry 3: 1/3/2000 started 8:00 PM finished 8:10 PM
    Entries/Format
This format (particularily the concept of pages) was made for writing in a
physical notebook, with this tool being used to maintain a digital backup.

```
