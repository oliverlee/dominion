use scraper::*;

fn main() {
    let scrape: Scrape = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open("../dominion.json").unwrap(),
    ))
    .unwrap();

    // TODO: Generate code from data.
}
