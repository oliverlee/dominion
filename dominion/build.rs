#![recursion_limit = "128"]
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use regex::Regex;
use scraper::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (types, extended_cards) = parse_cardset("Dominion 2nd Edition")?;

    let declaration_lines: Vec<_> = extended_cards
        .iter()
        .map(|card| Ident::new(&card.ident, Span::call_site()))
        .collect();

    // Declare a reference so we can reuse the Ident's.
    let ident = &declaration_lines;

    let name = extended_cards
        .iter()
        .map(|card| Literal::string(&card.card.name));

    let is_type_methods = CARD_TYPES.iter().map(|s| {
        let method = is_type_method(&extended_cards, &types, s);

        quote! { #method }
    });

    let cost = extended_cards.iter().map(|card| {
        Literal::u8_suffixed(match card.card.cost {
            Cost::Copper(x) => x,
            _ => 0,
        })
    });

    let vp_method = victory_points_method(&extended_cards);

    let description = extended_cards.iter().map(|card| &card.card.description);

    let resources = parse_description(&extended_cards);

    let tokens = quote! {
        use serde::Deserialize;
        use std::str::FromStr;

        #[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
        pub enum CardKind {
            #(#ident,)*
        }

        impl FromStr for CardKind {
            type Err = serde_json::error::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                serde_json::from_str(&format!("\"{}\"", s))
            }
        }

        #[allow(dead_code)]
        impl CardKind {
            pub fn name(self) -> &'static str {
                match self {
                    #(CardKind::#ident => #name,)*
                }
            }

            #(#is_type_methods)*

            // Base set only costs copper so just return an int for now.
            pub fn cost(self) -> u8 {
                match self {
                    #(CardKind::#ident => #cost,)*
                }
            }

            #vp_method

            pub fn description(self) -> &'static str {
                match self {
                    #(CardKind::#ident => #description,)*
                }
            }
        }

        // TODO: prevent object from being constructed?
        #[derive(Debug)]
        pub struct CardResources {
            pub cards: u8,
            pub actions: u8,
            pub buys: u8,
            pub copper: u8,
        }

        #resources
    };

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let out_path = out_dir.join("card.rs");

    let mut file = BufWriter::new(File::create(&out_path)?);
    file.write_all(tokens.to_string().as_bytes())?;
    drop(file); // Don't forget to flush!

    match Command::new("rustfmt").arg(&out_path).status() {
        Ok(status) => assert!(status.success()),
        Err(_) => {} // Can't run, whatever
    }

    Ok(())
}

const CARD_TYPES: &[&'static str] = &["Action", "Reaction", "Attack", "Victory", "Treasure"];

struct CardExt {
    card: Card,
    ident: String,
}

fn parse_cardset(
    set_name: &str,
) -> Result<(Vec<String>, Vec<CardExt>), Box<dyn std::error::Error>> {
    let mut missing_no: usize = 0;

    let Scrape {
        sets,
        types,
        mut cards,
    } = serde_json::from_reader(BufReader::new(
        std::fs::File::open("../dominion.json").unwrap(),
    ))?;

    let non_ident_regex = Regex::new(r"[^\w\d]+").unwrap();

    let baseset_card_indices: Vec<_> = sets
        .iter()
        .flat_map(|set| {
            if &set.name == set_name {
                set.card_indices.iter()
            } else {
                [].iter()
            }
        })
        .copied()
        .collect();

    // Verify that the range is contiguous.
    for (&a, &b) in baseset_card_indices
        .iter()
        .zip(baseset_card_indices.iter().skip(1))
    {
        assert_eq!(a + 1, b, "card indices are not contiguous");
    }

    let mut extended_cards: Vec<_> = cards
        .drain(std::ops::Range {
            start: baseset_card_indices.first().unwrap(),
            end: baseset_card_indices.last().unwrap(),
        })
        .map(|card| {
            let ident = if card.name.is_empty() {
                missing_no += 1;
                format!("MissingNo{}", missing_no)
            } else {
                non_ident_regex.replace_all(&card.name, "").to_string()
            };

            CardExt { card, ident }
        })
        .collect();

    // Add base cards to the set.
    extended_cards.append(&mut base_cards(&types));

    Ok((types, extended_cards))
}

fn victory_points_method(cards: &Vec<CardExt>) -> TokenStream {
    let victory_regex = Regex::new(r"(-?\d+) Victory Points?").unwrap();

    let victory_point_matches = cards.iter().filter_map(|card| {
        card.card.description.split('\n').find_map(|line| {
            // This assumes that the first match is the only match.
            victory_regex
                .captures(line)
                .map(|captures| captures.get(1).unwrap().as_str().parse::<i32>().unwrap())
                .and_then(|points| {
                    let ident = Ident::new(&card.ident, Span::call_site());
                    Some(quote! { CardKind::#ident => #points })
                })
        })
    });

    quote! {
        pub fn victory_points(self) -> i32 {
            match self {
                #(#victory_point_matches,)*
                _ => 0,
            }
        }
    }
}

fn match_lines_by_type<'a>(
    cards: &'a Vec<CardExt>,
    types: &Vec<String>,
    type_name: &str,
) -> impl Iterator<Item = TokenStream> + 'a {
    let type_index = types.iter().position(|t| t == type_name).unwrap();

    cards.iter().filter_map(move |card| {
        let is_type = card
            .card
            .type_indices
            .iter()
            .any(|&index| index == type_index);

        if is_type {
            let ident = Ident::new(&card.ident, Span::call_site());
            Some(quote! { CardKind::#ident => true })
        } else {
            None
        }
    })
}

fn is_type_method(cards: &Vec<CardExt>, types: &Vec<String>, type_name: &str) -> TokenStream {
    let is_type_match_lines = match_lines_by_type(cards, types, type_name);

    let method_name = format!("is_{}", type_name.to_lowercase());
    let method_name = Ident::new(&method_name, Span::call_site());

    quote! {
        pub fn #method_name(self) -> bool {
            match self {
                #(#is_type_match_lines,)*
                _ => false,
            }
        }
    }
}

fn base_cards(types: &Vec<String>) -> Vec<CardExt> {
    let Scrape {
        sets: _,
        types: _,
        cards: mut base_cards,
    } = serde_json::from_reader(BufReader::new(
        std::fs::File::open("../base_cards.json").unwrap(),
    ))
    .unwrap();

    // Type indices are not set for base cards as they are stored in a separate
    // file from the rest of the cards.
    let treasure_index = types.iter().position(|s| s == "Treasure").unwrap();
    let victory_index = types.iter().position(|s| s == "Victory").unwrap();

    for card in &mut base_cards {
        let type_index = match card.name.as_ref() {
            "Copper" | "Silver" | "Gold" => treasure_index,
            _ => victory_index,
        };

        card.type_indices.push(type_index);
    }

    base_cards
        .into_iter()
        .map(|card| {
            let ident = card.name.clone();

            CardExt { card, ident }
        })
        .collect()
}

fn parse_description(cards: &Vec<CardExt>) -> TokenStream {
    // Build resource regexs
    let other_regex = Regex::new(r"\+(\d+) (Card|Action|Buy)").unwrap();
    let copper_regex = Regex::new(r"\+\$(\d+)").unwrap();

    let desc_type_delim = "———-";

    let card_matches = cards.iter().filter_map(|card| {
        let mut captures = card
            .card
            .description
            .split(desc_type_delim)
            .take(1)
            .next()
            .unwrap()
            .split('\n')
            .filter_map(|line| {
                other_regex
                    .captures(&line)
                    .or_else(|| copper_regex.captures(&line))
            })
            .peekable();

        if let Some(_) = captures.peek() {
            Some((card, captures))
        } else {
            None
        }
    });

    let mut matches = Vec::new();
    let mut const_defs = Vec::new();

    for (card, captures) in card_matches {
        let mut cards = 0;
        let mut actions = 0;
        let mut buys = 0;
        let mut copper = 0;

        for capture in captures {
            let value = capture.get(1).unwrap().as_str().parse::<u8>().unwrap();
            match capture.get(2).map(|s| s.as_str()) {
                Some("Card") => cards += value,
                Some("Action") => actions += value,
                Some("Buy") => buys += value,
                _ => copper += value,
            }
        }

        let ident = Ident::new(&card.ident, Span::call_site());

        let def = Ident::new(
            &format!("{}_RESOURCES", &card.ident.to_uppercase()),
            Span::call_site(),
        );

        matches.push(quote! {
            CardKind::#ident => Some(#def)
        });

        const_defs.push(quote! {
            const #def: &CardResources = &CardResources {
                cards: #cards,
                actions: #actions,
                buys: #buys,
                copper: #copper,
            }
        });
    }

    quote! {
        impl CardKind {
            pub fn resources(self) -> Option<&'static CardResources> {
                match self {
                    #(#matches,)*
                    _ => None,
                }
            }
        }

        #(#const_defs;)*
    }
}
