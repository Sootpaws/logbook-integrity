use crate::Logbook;
use std::collections::HashMap;
use std::iter;

const PREFIX_TOKENS: usize = 2;

/// Generate a chain from a collection of logbooks
pub fn chain_from_logs(logs: &[Logbook]) -> Chain {
    logs.iter()
        .flat_map(|logbook| logbook.entries.iter())
        .flat_map(|entry| entry.contents.iter())
        .fold(Chain::new(), |mut chain, block| {
            chain.feed(&block.text, ["", ""]);
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

    /// "Feed" a piece of text to the chain to train it
    pub fn feed(&mut self, text: &str, preceding_tokens: [&str; PREFIX_TOKENS]) {
        let mut preceding_tokens = preceding_tokens.map(str::to_owned);
        for token in text.split(" ") {
            self.add_link(token, &preceding_tokens);
            preceding_tokens = preceding_tokens
                .into_iter()
                .skip(1)
                .chain(iter::once(token.to_owned()))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            dbg!(&preceding_tokens);
        }
    }

    /// Add a single relation between a token and the preceding ones
    fn add_link(&mut self, token: &str, preceding_tokens: &[String; PREFIX_TOKENS]) {
        *self
            .links
            .entry(preceding_tokens.clone())
            .or_default()
            .entry(token.to_owned())
            .or_insert(0) += 1;
    }
}
