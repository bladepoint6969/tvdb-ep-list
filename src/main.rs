mod api;

use std::{error::Error, io::Write};

use api::Api;
use clap::{Arg, ArgGroup, ArgMatches, Command};
use serde::{Deserialize, Serialize};

const CHAR_REPLACE: [[&str; 2]; 9] = [
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

async fn get_id_from_user() -> Option<usize> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        print!("Multiple results found, enter a numeric ID (anything else to quit): ");
        std::io::stdout().flush().unwrap();

        tx.send(
            match std::io::stdin()
                .lines()
                .next()
                .unwrap()
                .unwrap()
                .parse::<usize>()
            {
                Ok(s) => Some(s),
                Err(_) => None,
            },
        )
        .unwrap();
    });

    rx.await.unwrap()
}

async fn do_search(matches: ArgMatches, config: Config) -> Result<(), Box<dyn Error>> {
    let api = Api::new(&config.api_key).await?;

    let target_series: usize = if !matches.is_present("id") {
        let series_results = api
            .search_series(
                matches.value_of("name"),
                None,
                None,
                None,
                matches.value_of("lang"),
            )
            .await?;

        if series_results.len() == 1 {
            series_results[0].id
        } else {
            for series in series_results {
                println!("{}: {}", series.series_name, series.id);
            }
            match get_id_from_user().await {
                Some(id) => id,
                None => return Ok(()), // User decided to abort
            }
        }
    } else {
        matches.value_of("id").unwrap().parse().unwrap()
    };

    let series = api.get_series(target_series, None).await?;

    let mut episodes = api.get_series_episodes(series.id).await?;

    episodes.sort();

    for episode in episodes {
        let mut episode_name = match episode.episode_name {
            Some(name) => name,
            None => "".into(),
        };
        episode_name = replace_chars(episode_name);

        println!(
            "{} - s{:0>2}e{:0>2} - {episode_name}",
            series.series_name, episode.aired_season, episode.aired_episode_number
        );
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("TVDB Episode list")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or("0.0.0"))
        .author(option_env!("CARGO_PKG_AUTHORS").unwrap_or("Unknown"))
        .about("Print an episode listing for the specified series")
        .arg(
            Arg::new("name")
                .takes_value(true)
                .short('n')
                .long("name")
                .help("Name of a series to search for"),
        )
        .arg(
            Arg::new("id")
                .takes_value(true)
                .short('i')
                .long("id")
                .help("Series ID")
                .conflicts_with("name")
                .validator(|v| {
                    let size: usize = v.parse().unwrap_or_default();

                    if size == 0 {
                        Err("The ID must be a whole number greater than 0".to_string())
                    } else {
                        Ok(())
                    }
                }),
        )
        .arg(
            Arg::new("lang")
                .takes_value(true)
                .short('l')
                .long("lang")
                .default_value("en")
                .help("Language code for API Results"),
        )
        .arg(
            Arg::new("key")
                .takes_value(true)
                .short('k')
                .long("key")
                .help("Update configured API key"),
        )
        .group(ArgGroup::new("target").args(&["name", "id"]).required(true))
        .get_matches();

    let mut cfg: Config = confy::load(env!("CARGO_PKG_NAME"))?;

    if matches.is_present("key") {
        cfg.api_key = matches.value_of("key").unwrap().into();
        confy::store(env!("CARGO_PKG_NAME"), cfg.clone()).ok();
    }

    do_search(matches, cfg).await
}
