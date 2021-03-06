/*
 * Copyright (C) 2018 Eddie Antonio Santos <easantos@ualberta.ca>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
struct Digraph(Token, Token);

/**
 * Which language?
 */
#[derive(Debug)]
enum Language {
  Crk, // nêhiyawêwin/Plains Cree
  Eng, // English
}

/**
 * How many times a digraph appears in nêhiyawêwin vs. English.
 */
#[derive(Debug)]
struct Occurance {
  crk: u32,
  eng: u32
}

struct Classifier {
  features: HashMap<Digraph, Occurance>
}


fn main() -> io::Result<()> {
  let mut model = Classifier::new();
  model.count_digraphs_in_file("itwêwina", Language::Crk);
  model.count_digraphs_in_file("words", Language::Eng);

  model.prune_features();

  use std::io;
  use std::io::prelude::*;

  let stdin = io::stdin();
  for line in stdin.lock().lines() {
    let word = line_to_word(&line.unwrap());
    let guessed_lang = model.classify(&word);

    println!("{}: {:?}", word, guessed_lang);
  }

  Ok(())
}

/// Gets rid of surrounding whitespace,
/// removes circumflexes,
/// and lowercase's everting.
fn line_to_word(line: &str) -> String {
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
fn digraphs_of(text: &str) -> HashSet<Digraph> {
  if text.is_empty() {
    return HashSet::new();
  }
  assert!(!text.ends_with('\n'));

  let mut digraphs = HashSet::new();

  // The first digraph always has includes the Start token.
  let mut last_char = Token::Start;
  for ch in text.chars() {
    let this_char = Token::Char(ch);
    digraphs.insert(Digraph(last_char, this_char));
    last_char = this_char;
  }

  // Finalize by adding last character in the string.
  digraphs.insert(Digraph(last_char, Token::End));

  digraphs
}


impl Classifier {
  fn new() -> Classifier {
    Classifier { features: HashMap::new() }
  }

  /**
   * Given a filename, gets a set of all of the digraphs present in each word.
   * Use the "on_digraph" closure to increment the correct counter.
   */
  fn count_digraphs_in_file(&mut self, filename: &str, lang: Language) {
    let file = File::open(filename).expect("file not found");

    for line in BufReader::new(file).lines() {
      let line = line.expect("Couldn't get line");
      let word = line_to_word(&line);
      for digraph in digraphs_of(&word).iter() {
        let occ = self.features.entry(*digraph)
          .or_insert(Occurance { crk: 0, eng: 0});
        match lang {
          Language::Crk => occ.crk += 1,
          Language::Eng => occ.eng += 1,
        };
      }
    }
  }

  /**
   * Removes unhelpful features.
   */
  fn prune_features(&mut self) {
    // "Unhelpful" features are digraphs that have only been witnessed once, ever.
    // Remove them, since they don't add much when classifying.
    self.features.retain(|_digraph, occ| occ.total() > 1);
  }

  fn classify(&self, word: &str) -> Language {
    let mut log_prob_crk: f64 = 0.0;
    let mut log_prob_eng: f64 = 0.0;

    for digraph in digraphs_of(word) {
      // Skip digraphs we've never seen.
      if !self.features.contains_key(&digraph) {
        continue;
      }

      log_prob_crk += self.log_prob(digraph, Language::Crk).expect("digraph does not exist");
      log_prob_eng += self.log_prob(digraph, Language::Eng).expect("digraph does not exist");
    }

    println!("  P(crk|{}) = {}", word, log_prob_crk.exp());
    println!("  P(eng|{}) = {}", word, log_prob_eng.exp());

    if log_prob_crk > log_prob_eng {
      Language::Crk
    } else {
      Language::Eng
    }
  }

  fn log_prob(&self, digraph: Digraph, language: Language) -> Option<f64> {
    if let Some(occurance) = self.features.get(&digraph) {
      let numerator: f64 = (occurance.of(language) + 1).into();
      let denominator: f64 = (occurance.total() + self.num_features()).into();

      Some(numerator.ln() - denominator.ln())
    } else {
      None
    }
  }

  fn num_features(&self) -> u32 {
    self.features.len() as u32
  }
}


impl Occurance {
  fn total(&self) -> u32 {
    self.crk + self.eng
  }

  fn of(&self, language: Language) -> u32 {
    match language {
      Language::Crk => self.crk,
      Language::Eng => self.eng,
    }
  }
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
