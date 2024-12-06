use crate::Logbook;
use std::collections::HashMap;

const PREFIX_TOKENS: usize = 2;

/// Generate a chain from a collection of logbooks
pub fn chain_from_logs(logs: &[Logbook]) -> Chain {
    logs.iter()
        .flat_map(|logbook| logbook.entries.iter())
        .flat_map(|entry| entry.contents.iter())
        .fold(Chain::new(), |mut chain, block| {
            chain.feed(block, &["", ""]);
            chain
        })
}

/// A semi-randomized Markov chain text generator
#[derive(Default, Debug)]
pub struct Chain {
    /// Map<Preceding tokens -> Map<Potential next token -> number of occurences>>
    links: HashMap<[String; PREFIX_TOKENS], HashMap<String, usize>>,
}

impl Chain {
    /// Create a new, untrained chain
    pub fn new() -> Self {
        Self::default()
    }
}
