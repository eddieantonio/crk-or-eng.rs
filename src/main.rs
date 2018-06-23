use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufReader, BufRead};

/**
 * Since we're interested in counting what are common starts of words, and common ends of words, a
 * "token" is more than simply a character---we encode the start and end of words explicitly.
 */
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum Token {
  Start,
  End,
  Char(char)
}

/**
 * A digraph is two tokens stuck together.
 */
#[derive(PartialEq, Eq, Hash, Debug)]
struct Digraph(Token, Token);


fn main() {
    let crk_digraphs = count_digraphs_in_file("itwêwina");

    for (digraph, count) in crk_digraphs.iter() {
      println!("{0}\t{1}{2}", count, digraph.0, digraph.1);
    }
}

/**
 * Given a filename of a word list, counts all of the digraphs
 * present, and returns it as a HashMap.
 */
fn count_digraphs_in_file(filename: &str) -> HashMap<Digraph, u32> {
    let file = File::open(filename).expect("file not found");
    let mut digraphs = HashMap::new();

    for line in BufReader::new(file).lines() {
        let line = line.expect("Couldn't get line");
        count_digraphs(&mut digraphs, &preprocess_line(&line));
    }

    digraphs
}


/// Gets rid of surrounding whitespace,
/// removes circumflexes,
/// and lowercase's everting.
fn preprocess_line(line: &str) -> String {
  let mut buffer = String::new();
  // Remove extraneous spaces and punctuation.
  let word = line.trim_right_matches(|c| "!? \n".contains(c));

  for ch in word.chars() {
    // TODO: use a crate the provides NFD normalization,
    // and simply remove \u{03xx} code points.
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

/**
 * Counts digraphs in a word. Assumes the word has already been preprocessed.
 */
fn count_digraphs(counter: &mut HashMap<Digraph, u32>, text: &String) {
  if text.is_empty() {
    return;
  }
  assert!(!text.ends_with('\n'));

  // The first digraph always has includes the Start token.
  let mut last_char = Token::Start;
  for ch in text.chars() {
    let this_char = Token::Char(ch);
    let digraph = Digraph(last_char, this_char);

    let count = counter.entry(digraph).or_insert(0);
    *count += 1;

    last_char = this_char;
  }

  // Finalize by adding last character in the string.
  let count = counter.entry(Digraph(last_char, Token::End))
    .or_insert(0);
  *count += 1;
}

/**
 * Displays a character. Note that this will panic if the characters are either
 * '^' or '$' as those are used to indicate the Start and End tokens, respectively.
 */
impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", match *self {
      Token::Start => '^',
      Token::End => '$',
      Token::Char(c) => {
        /* Make sure the meaning of '^' and '$' is unambiguous---
         * we wouldn't want our shorthand to be an actual character! */
        assert!(c != '^' || c != '$');
        c
      },
    })
  }
}
