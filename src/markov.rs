use crate::Logbook;
use std::collections::HashMap;
use std::iter;
use rand;
use rand::Rng;

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

type PrecTokens = [String; PREFIX_TOKENS];

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
            preceding_tokens = Self::advance_previous(token, &preceding_tokens);
        }
        self.add_link("", &preceding_tokens);
    }

    /// Generate a sequence of text from this chain
    pub fn generate(&self, preceding_tokens: [&str; PREFIX_TOKENS]) -> String {
        let mut preceding_tokens = preceding_tokens.map(str::to_owned);
        iter::from_fn(|| {
            let token = self.follow_link(&preceding_tokens);
            preceding_tokens = Self::advance_previous(&token, &preceding_tokens);
            if !token.is_empty() {
                Some(token)
            } else {
                None
            }
        }).reduce(|mut built, next| {
            built.push(' ');
            built.push_str(&next);
            built
        }).unwrap_or_else(String::new)
    }

    /// Add a single relation between a token and the preceding ones
    fn add_link(&mut self, token: &str, preceding_tokens: &PrecTokens) {
        *self
            .links
            .entry(preceding_tokens.clone())
            .or_default()
            .entry(token.to_owned())
            .or_insert(0) += 1;
    }

    /// Follow a link from preceding tokens to generate the following one
    fn follow_link(&self, preceding_tokens: &PrecTokens) -> String {
        if let Some(link) = self.links.get(preceding_tokens) {
            let sum = link.values().sum();
            let mut m = link.iter().collect::<Vec<_>>();
            m.sort_by(|(_, count_a), (_, count_b)| count_b.cmp(count_a));
            let mut i = rand::thread_rng().gen_range(0..=sum);
            m.iter().find(|(_, count)| {
                i = i.saturating_sub(**count);
                i == 0
            }).map(|(token, _)| (*token).to_owned()).unwrap_or_else(String::new)
        } else {
            String::new()
        }
    }

    /// Advance a list of previous tokens
    fn advance_previous(token: &str, preceding_tokens: &PrecTokens) -> PrecTokens {
        preceding_tokens.clone().into_iter()
            .skip(1)
            .chain(iter::once(token.to_owned()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}
