use std::fs::File;
use std::io::{BufReader, BufRead, Result};
use std::collections::HashMap;

const START: char = '\u{0002}'; // START OF TEXT
const END: char = '\u{0003}'; // END OF TEXT

#[derive(PartialEq, Eq, Hash, Debug)]
struct Bigram(char, char);


fn main() -> Result<()> {
    println!("Hello, world!");
    let crk_file = File::open("itwêwina").expect("file not found");
    let mut crk_bigrams = HashMap::new();

    for line in BufReader::new(crk_file).lines() {
        let line = line.expect("Couldn't get line");
        count_bigrams(&mut crk_bigrams, line.trim())
    }

    println!("{:?}", crk_bigrams);
    Ok(())
}

fn count_bigrams(counter: &mut HashMap<Bigram, i32>, text: &str) {
  if text.len() < 1 {
    return;
  }

  let mut last_char = START;
  for this_char in text.chars() {
    let bigram = Bigram(last_char, this_char);

    let count = counter.entry(bigram).or_insert(0);
    *count += 1;

    last_char = this_char;
  }
}
