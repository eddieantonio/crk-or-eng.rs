use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;

/**
 * A token can be a character, but it can also be the special Start and End tokens.
 */
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum Token {
  Start,
  End,
  Char(char)
}

/**
 * A bigram is two tokens stuck together.
 */
#[derive(PartialEq, Eq, Hash, Debug)]
struct Bigram(Token, Token);


fn main() {
    let crk_bigrams = count_bigrams_in_file("itwêwina");

    for (bigram, count) in crk_bigrams.iter() {
      println!("{0}\t{1}{2}", count, extract(bigram.0), extract(bigram.1));
    }
}

fn count_bigrams_in_file(filename: &str) -> HashMap<Bigram, u32> {
    let file = File::open(filename).expect("file not found");
    let mut bigrams = HashMap::new();

    for line in BufReader::new(file).lines() {
        let line = line.expect("Couldn't get line");

        count_bigrams(&mut bigrams, &preprocess_line(&line));
    }

    bigrams
}


// TODO: convert into format for Tokens?
fn extract(t: Token) -> char {
  match t {
    Token::Char(c) => c,
    Token::Start => '^',
    Token::End => '$'
  }
}

/// Gets rid of surrounding whitespace,
/// removes circumflexes,
/// and lowercase's everting.
fn preprocess_line(line: &str) -> String {
  let mut buffer = String::new();
  // Remove extraneous spaces and punctuation.
  let word = line.trim_right_matches(|c| "!? \n".contains(c));

  for ch in word.chars() {
    let ch = ch.to_lowercase().nth(0).unwrap();
    buffer.push(match ch {
      'â' => 'a',
      'ê' => 'e',
      'î' => 'i',
      'ô' => 'o',
      _ => ch,
    })
  }

  buffer
}

fn count_bigrams(counter: &mut HashMap<Bigram, u32>, text: &String) {
  if text.len() < 1 {
    return;
  }

  let mut last_char = Token::Start;
  for ch in text.chars() {
    let this_char = Token::Char(ch);
    let bigram = Bigram(last_char, this_char);

    let count = counter.entry(bigram).or_insert(0);
    *count += 1;

    last_char = this_char;
  }

  // Finalize by adding last character in the string.
  let count = counter.entry(Bigram(last_char, Token::End))
    .or_insert(0);
  *count += 1;
}
