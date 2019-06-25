use proc_macro2::*;
use quote::quote;
use regex::*;
use scraper::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut missing_no: usize = 0;

    let Scrape { sets, types, cards } = serde_json::from_reader(BufReader::new(
        std::fs::File::open("../dominion.json").unwrap(),
    ))?;

    // let out_dir = Path::new(std::env::var("OUT_DIR")?);
    let out_dir = Path::new(".");
    let out_path = out_dir.join("data.rs");
    let mut file = BufWriter::new(File::create(&out_path)?);

    let non_ident_regex = Regex::new(r"[^\w\d]+").unwrap();

    let base_card_indices = sets.iter().flat_map(|set| {
        if &set.name == "Dominion 2nd Edition" {
            set.card_indices.iter()
        } else {
            [].iter()
        }
        .copied()
    });

    let extended_cards: Vec<CardExt> = base_card_indices
        .map(|index| {
            let card = &cards[index as usize];

            let ident = if card.name.is_empty() {
                missing_no += 1;
                format!("MissingNo{}", missing_no)
            } else {
                non_ident_regex.replace_all(&card.name, "").to_string()
            };

            CardExt { card, ident }
        })
        .collect();

    let declaration_lines = extended_cards
        .iter()
        .map(|card| Ident::new(&card.ident, Span::call_site()));

    let name_match_lines = extended_cards.iter().map(|card| {
        let ident = Ident::new(&card.ident, Span::call_site());
        let name = Literal::string(&card.card.name);

        quote! { CardKind::#ident => #name }
    });

    let is_type_methods = ["Action", "Reaction", "Attack", "Victory", "Treasure"]
        .iter()
        .map(|s| {
            let method = is_type_method(&extended_cards, &types, s);

            quote! { #method }
        });

    let tokens = quote! {
        #[derive(Debug)]
        enum CardKind {
            #(#declaration_lines,)*
        }

        impl CardKind {
            fn name(&self) -> &'static str {
                match *self {
                    #(#name_match_lines,)*
                }
            }

            #(#is_type_methods)*
        }

    };

    // TODO: Base cards are not included
    // Copper
    // Silver
    // Gold
    // Estate
    // Duchy
    // Province
    // Curse
    file.write_all(tokens.to_string().as_bytes())?;

    // Don't forget to flush!
    drop(file);

    match Command::new("rustfmt").arg(&out_path).status() {
        Ok(status) => assert!(status.success()),
        Err(_) => {} // Can't run, whatever
    }

    Ok(())
}

struct CardExt<'a> {
    card: &'a Card,
    ident: String,
}

fn match_lines_by_type<'a>(
    cards: &'a Vec<CardExt>,
    types: &Vec<String>,
    type_name: &str,
) -> impl Iterator<Item = TokenStream> + 'a {
    let type_index = types.iter().position(|t| t == type_name).unwrap();

    cards.iter().filter_map(move |card| {
        let ident = Ident::new(&card.ident, Span::call_site());

        let is_type = card
            .card
            .type_indices
            .iter()
            .any(|&index| index == type_index);

        if is_type {
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
        fn #method_name(&self) -> bool {
            match *self {
                #(#is_type_match_lines,)*
                _ => false,
            }
        }
    }
}
