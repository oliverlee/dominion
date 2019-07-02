use regex::Regex;
use scraper::*;
use select::document::Document;
use select::predicate::{Class, Name};
use std::collections::HashMap;

fn main() {
    let scrape = scrape_dominion_strategy();
    std::fs::write(
        "dominion.json",
        serde_json::to_string_pretty(&scrape).unwrap(),
    )
    .unwrap();
}

#[allow(clippy::non_ascii_literal)]
fn scrape_dominion_strategy() -> Scrape {
    let mut sets: Vec<Set> = Vec::new();
    let mut cards: Vec<Card> = Vec::new();
    let mut types: Vec<String> = Vec::new();
    let mut type_name_to_index: HashMap<String, usize> = HashMap::new();

    // Build regular expressions.
    let type_regex = Regex::new(r"\w+").unwrap();
    let cost_regex = Regex::new(r"(?:\$(\d+))?(◉|\*)?").unwrap();

    let document =
        Document::from_read(reqwest::get("https://dominionstrategy.com/all-cards/").unwrap())
            .unwrap();
    let node = document.find(Class("entry-content")).next().unwrap();

    let titles = node.find(Name("h2"));
    let tables = node.find(Name("table"));
    for (set_name, table) in titles.zip(tables) {
        let mut set = Set {
            name: set_name.text(),
            card_indices: Vec::new(),
        };
        for row in table.find(Name("tr")) {
            let mut cols = row.find(Name("td"));
            let card = Card {
                name: cols
                    .next()
                    .map(|node| node.text().trim().to_string())
                    .unwrap_or_default(),

                type_indices: cols
                    .next()
                    .map(|node| {
                        let text = node.text();
                        type_regex
                            .find_iter(&text)
                            .map(|m| {
                                let ty = m.as_str().trim().to_string();
                                // Unfortunately we can't look up by reference. This is
                                // a known problem with the entry API.
                                *type_name_to_index.entry(ty.clone()).or_insert_with(|| {
                                    let i = types.len();
                                    types.push(ty);
                                    i
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                cost: cols
                    .next()
                    .map(|node| {
                        let text = node.text().trim().to_string();
                        let captures = cost_regex.captures(&text).unwrap_or_else(|| {
                            panic!("Failed to parse card cost from {:?}.", &text)
                        });
                        match captures.get(1) {
                            Some(value_match) => {
                                let value: u8 = value_match.as_str().parse().unwrap();
                                match captures.get(2) {
                                    Some(postfix_match) => match postfix_match.as_str() {
                                        "◉" => Cost::Potion(value),
                                        _ => Cost::Special(value),
                                    },
                                    None => Cost::Copper(value),
                                }
                            }
                            None => Cost::None,
                        }
                    })
                    .unwrap(),
                description: cols
                    .next()
                    .map(|node| node.text().trim().to_string())
                    .unwrap_or_default(),
            };
            let card_index = cards.len();
            cards.push(card);
            set.card_indices.push(card_index);
        }
        sets.push(set);
    }

    Scrape { sets, types, cards }
}
