#![deny(clippy::unwrap_used)]

mod api;

use std::{error::Error, fmt::Display, io::Write};

use api::Api;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

const CHAR_REPLACE: &[[&str; 2]] = &[
    ["\\", "-"],
    ["/", "-"],
    [":", " -"],
    ["*", "-"],
    ["?", ""],
    ["\"", ""],
    ["<", "\u{2190}"],
    [">", "\u{2192}"],
    ["|", "-"],
];

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { api_key: "".into() }
    }
}

fn replace_chars(episode: String) -> String {
    let mut episode = episode;
    for pair in CHAR_REPLACE {
        episode = episode.replace(pair[0], pair[1]);
    }
    episode
}

async fn get_id_from_user() -> Option<u64> {
    tokio::task::spawn_blocking(|| {
        print!("Multiple results found, enter a numeric ID (anything else to quit): ");
        std::io::stdout().flush().expect("Flushing stdio buffer");
        let id = std::io::stdin()
            .lines()
            .next()
            .expect("Some line from stdin")
            .expect("Read from stdin")
            .parse::<u64>()
            .ok();
        println!();

        id
    })
    .await
    .expect("Got Input")
}

async fn do_search(matches: Cli, config: Config) -> Result<(), Box<dyn Error>> {
    let api = Api::new(&config.api_key).await?;

    let target_series: u64 = if matches.id.is_none() {
        let name = matches.name.expect("Has name");
        let series_results = api
            .search_series(Some(&name), None, None, None, Some(&matches.lang))
            .await?;

        if series_results.len() == 1 {
            series_results[0].id
        } else {
            for series in series_results {
                println!("{}: {}", series.series_name, series.id);
            }
            println!();
            match get_id_from_user().await {
                Some(id) => id,
                None => return Ok(()), // User decided to abort
            }
        }
    } else {
        matches.id.expect("Should have ID")
    };

    let series = api.get_series(target_series, None).await?;
    let series_name = replace_chars(series.series_name);

    let mut episodes = api.get_series_episodes(series.id).await?;

    episodes.sort();

    for episode in episodes {
        let mut episode_name = match episode.episode_name {
            Some(name) => name,
            None => "".into(),
        };
        episode_name = replace_chars(episode_name);
        let (season, ep) = match matches.ordering {
            Ordering::Aired => (episode.aired_season, episode.aired_episode_number),
            Ordering::Dvd => (
                episode.dvd_season.unwrap_or(episode.aired_season),
                episode
                    .dvd_episode_number
                    .unwrap_or(episode.aired_episode_number),
            ),
        };

        if episode_name.is_empty() {
            println!("{series_name} - s{:0>2}e{:0>2}", season, ep);
        } else {
            println!(
                "{series_name} - s{:0>2}e{:0>2} - {episode_name}",
                season, ep
            );
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Ordering {
    Aired,
    Dvd,
}

impl Display for Ordering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ordering::Aired => write!(f, "aired"),
            Ordering::Dvd => write!(f, "dvd"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "TVDB Episode List",
    version,
    author,
    about = "Print an episode listing for the specified series"
)]
struct Cli {
    #[arg(short, long, default_value_t = Ordering::Aired)]
    /// The Episode ordering to use
    ordering: Ordering,
    #[arg(short, long, help = "Name of a series to search for")]
    name: Option<String>,
    #[arg(short, long, help = "Series ID", conflicts_with = "name", value_parser = clap::value_parser!(u64).range(1..))]
    id: Option<u64>,
    #[arg(
        short,
        long,
        help = "Language code for API Results",
        default_value = "en"
    )]
    lang: String,
    #[arg(short, long, help = "Update configured API key")]
    key: Option<String>,
}

static CONFIG_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Cli::parse();

    let mut cfg: Config = confy::load(CONFIG_NAME, Some(CONFIG_NAME))?;

    if let Some(ref key) = matches.key {
        cfg.api_key.clone_from(key);
        confy::store(CONFIG_NAME, Some(CONFIG_NAME), cfg.clone()).ok();
    }

    do_search(matches, cfg).await
}
