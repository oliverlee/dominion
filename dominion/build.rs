use proc_macro2::*;
use quote::quote;
use regex::*;
use scraper::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::process::Command;

enum CardKind {
    CardName,
}

fn main() -> Result<(), Box<std::error::Error>> {
    let mut missing_no: usize = 0;

    let Scrape { sets, types, cards } = serde_json::from_reader(BufReader::new(
        std::fs::File::open("../dominion.json").unwrap(),
    ))?;

    // let out_dir = Path::new(std::env::var("OUT_DIR")?);
    let out_dir = Path::new(".");
    let out_path = out_dir.join("data.rs");
    let mut file = BufWriter::new(File::create(&out_path)?);

    let non_ident_regex = Regex::new(r"[^\w\d]+").unwrap();

    struct CardExt {
        card: Card,
        ident: String,
    };

    let extended_cards: Vec<CardExt> = cards
        .into_iter()
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

    let declaration_lines = extended_cards
        .iter()
        .map(|card| Ident::new(&card.ident, Span::call_site()));

    let name_match_lines = extended_cards.iter().map(|card| {
        let ident = Ident::new(&card.ident, Span::call_site());
        let name = Literal::string(&card.card.name);

        quote! { CardKind::#ident => #name }
    });

    let action_type_index = types.iter().position(|ty| ty == "Action").unwrap();

    let is_action_match_lines = extended_cards.iter().filter_map(|card| {
        let ident = Ident::new(&card.ident, Span::call_site());
        let is_action = card.card.type_indices.iter().any(|&index| index == action_type_index);

        if is_action {
            Some(quote! { CardKind::#ident => true })
        } else {
            None
        }
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

            fn is_action(&self) -> bool {
                match *self {
                    #(#is_action_match_lines,)*
                    _ => false
                }
            }
        }

    };

    file.write_all(tokens.to_string().as_bytes())?;

    // Don't forget to flush!
    drop(file);

    match Command::new("rustfmt").arg(&out_path).status() {
        Ok(status) => assert!(status.success()),
        Err(_) => {} // Can't run, whatever
    }

    Ok(())
}
