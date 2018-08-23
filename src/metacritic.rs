use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::str;

use reqwest;
use regex::Regex;
use strsim::jaro;

pub fn create_lookup_handler() -> MetacriticLookupHandler {
    let mut database = MetacriticGameDatabase { games: Vec::new(), games_map: HashMap::new() };
    database.update();
    let handler = MetacriticLookupHandler { database: Arc::new(RwLock::new(database))};
    start_database_update_thread(handler.clone());
    handler
}

fn start_database_update_thread(handler: MetacriticLookupHandler) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::new(60*60, 0));
            handler.database.write().unwrap().update();
        }
    });
}

fn retrieve_switch_metacritic_games() -> Vec<MetacriticGame> {
    let mut games = Vec::new();
    let mut page = 0;

    loop {
        let mut results = retrieve_switch_metacritic_games_for_page(page);
        if results.is_empty() { break; }
        games.append(&mut results);
        page += 1;
    }

    games
}

fn retrieve_switch_metacritic_games_for_page(page: u32) -> Vec<MetacriticGame> {
    let client = reqwest::Client::new();
    let url = format!("http://www.metacritic.com/browse/games/release-date/available/switch/name?page={}", page);

    client.get(&url).send().ok().map(|mut resp| parse_metacritic_games(&resp.text().unwrap_or("".to_owned()))).unwrap_or(Vec::new())
}

fn parse_metacritic_games(html: &str) -> Vec<MetacriticGame> {
    lazy_static! {
        static ref GAME_RE: Regex = Regex::new(r####"(?s)<div class="basic_stat product_title">\s*<a href="(.*?)">(.*?)</a>.*?<div class="metascore_w(.*?)">(.*?)</div>.*?<span class="data textscore(.*?)">(.*?)</span>"####).unwrap();
    }
    GAME_RE.captures_iter(html).map(|m| {
        MetacriticGame {
            name: m[2].trim().to_owned(),
            href: format!("http://www.metacritic.com{}", m[1].trim()).to_owned(),
            score: m[4].trim().parse::<u32>().ok(),
            score_detail: parse_score_detail(m[3].trim().to_lowercase()),
            user_score: m[6].trim().parse::<f64>().ok(),
            user_score_detail: parse_user_score_detail(m[5].trim().to_lowercase()),
            stem: create_stem(m[2].trim()),
        }
    }).collect()
}

fn parse_score_detail(value: String) -> String {
    if value.contains("positive") {
        "positive".to_owned()
    } else if value.contains("mixed") {
        "mixed".to_owned()
    } else if value.contains("negative") {
        "negative".to_owned()
    } else {
        "tbd".to_owned()
    }
}

fn parse_user_score_detail(value: String) -> String {
    if value.contains("textscore_favorable") {
        "positive".to_owned()
    } else if value.contains("textscore_mixed") {
        "mixed".to_owned()
    } else if value.contains("textscore_unfavorable") {
        "negative".to_owned()
    } else {
        "tbd".to_owned()
    }
}

fn create_stem(name: &str) -> String {
    lazy_static! {
        static ref STEM_RE: Regex = Regex::new(r"[^a-zA-Z1-9 ]").unwrap();
        static ref SWITCH_EDITION_RE: Regex = Regex::new(r"(nintendo switch edition|switch edition|for nintendo switch|(- | )digital version)").unwrap();
    }
    SWITCH_EDITION_RE.replace_all(&STEM_RE.replace_all(&name.to_lowercase(), ""), "").trim().to_owned()
}

#[derive(Debug, Clone)]
pub struct MetacriticLookupHandler {
    database: Arc<RwLock<MetacriticGameDatabase>>
}

impl MetacriticLookupHandler {
    pub fn lookup_game(&self, game: &str) -> Option<MetacriticGame> {
        let game_stem = create_stem(game);
        let db = self.database.read().unwrap();
        let found_game = db.games_map.get(&game_stem);

        if found_game.is_some() {
            found_game.map(|g| g.clone())
        } else {
            let mut games_with_scores: Vec<(&MetacriticGame,f64)> = 
                db.games
                  .iter()
                  .map(|g| (g, jaro(&g.stem, &game_stem)))
                  .filter(|gs| gs.1 > 0.85)
                  .collect();
            games_with_scores.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            games_with_scores.last().map(|g| g.0.clone())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetacriticGameDatabase {
    games: Vec<MetacriticGame>,
    games_map: HashMap<String,MetacriticGame>,
}

impl MetacriticGameDatabase {
    fn update(&mut self) {
        self.games = retrieve_switch_metacritic_games();
        self.games_map = self.games.iter().flat_map(produce_stem_game_pairs).collect();
        println!("Updated database - {} Games", self.games.len());
    }
}

fn produce_stem_game_pairs(game: &MetacriticGame) -> Vec<(String, MetacriticGame)> {
    if game.name.contains(":") {
        let truncated_name = game.name.split(':').collect::<Vec<&str>>()[0].clone();
        vec![(game.stem.clone(), game.clone()),
             (create_stem(truncated_name), game.clone())]
    } else {
        vec![(game.stem.clone(), game.clone())]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetacriticGame {
    pub name: String,
    pub href: String,
    pub score: Option<u32>,
    pub score_detail: String,
    pub user_score: Option<f64>,
    pub user_score_detail: String,
    pub stem: String,
}
