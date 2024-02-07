use actix_web::{get, Responder};
use log::info;
use rand::Rng;

const WORDS: &'static str = include_str!("../10000-english-no-swears.txt");

fn load_words() -> Vec<&'static str> {
    WORDS
        .lines()
        .collect()
}

lazy_static::lazy_static! {
    static ref WORD_LIST: Vec<&'static str> = load_words();
}

fn generate_zoazo() -> String{
    let mut rng = rand::thread_rng();
    let zoazo_short = rng.gen_bool(0.5);
    let zoazo_len = if zoazo_short { 1 } else { rng.gen_range(2..=5) };
    let mut zoazo_words: Vec<&str> = Vec::with_capacity(zoazo_len);

    (0..zoazo_len).into_iter().for_each(|_| {zoazo_words.push(&WORD_LIST[rng.gen_range(0..WORD_LIST.len())])});

    let mut zoazo_emote = String::with_capacity(5 + zoazo_words.iter().map(|w| w.len()).sum::<usize>());
    zoazo_emote += "zoazo";
    for w in zoazo_words.into_iter() {
        w.chars().enumerate().for_each(|(i, c)| {
            if i == 0 {
                zoazo_emote.push(c.to_ascii_uppercase())
            } else {
                zoazo_emote.push(c)
            }});
    };

    return zoazo_emote;
}

#[get("/zoazo")]
async fn zoazo() -> impl Responder {
    let zoazo_emote = generate_zoazo();
    info!("zoazoEmote: {}", zoazo_emote);
    return zoazo_emote;
}
