use lazy_static::lazy_static;
use rand::{prelude::IteratorRandom, seq::SliceRandom, thread_rng};

const GAME_CODE_LEN: u32 = 8; //To be moved in some config file

lazy_static! {
    static ref CODE_CHARS: Vec<char> = ('A'..='Z').chain('0'..='9').collect();
    //One day we should do this in a better way
    static ref ADJECTIVES: Vec<String> = include_str!("../../resources/adjectives.txt")
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    static ref NOUNS: Vec<String> = include_str!("../../resources/nouns.txt")
        .lines()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
}

pub fn get_game_code() -> String {
    let mut rng = thread_rng();
    (0..GAME_CODE_LEN)
        .map(|_| CODE_CHARS.iter().choose(&mut rng).unwrap())
        .collect()
}

pub fn get_agent_name() -> String {
    let mut rng = thread_rng();
    let adjective = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();
    format!("{} {}", adjective, noun)
}
