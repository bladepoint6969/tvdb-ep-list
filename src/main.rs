mod api;

use std::error::Error;

use api::API;
use clap::{Arg, ArgGroup, Command};

const CHAR_REPLACE: [[&'static str; 2]; 9] = [
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

fn replace_chars(episode: String) -> String {
    let mut episode = episode;
    for pair in CHAR_REPLACE {
        episode = episode.replace(pair[0], pair[1]);
    }
    episode
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
        .group(ArgGroup::new("target").args(&["name", "id"]).required(true))
        .get_matches();

    let api = API::new(include_str!("api_key.txt")).await?;

    let target_series: usize;

    if !matches.is_present("id") {
        let series_results = api.search_series(
            matches.value_of("name"),
            None,
            None,
            None,
            matches.value_of("lang"),
        ).await?;

        if series_results.len() == 1 {
            target_series = series_results[0].id;
        }
        else {
            for series in series_results {
                println!("{}: {}", series.series_name, series.id);
            }
            println!("Multiple Series found, Please use the desired id with --id");
            return Ok(());
        }
    }
    else {
        target_series = matches.value_of("id").unwrap().parse().unwrap();
    }

    let series = api.get_series(target_series, None).await?;

    let mut episodes = api.get_series_episodes(series.id).await?;

    episodes.sort();

    for episode in episodes {
        let mut episode_name = match episode.episode_name {
            Some(name) => name,
            None => "".into(),
        };
        episode_name = replace_chars(episode_name);

        println!("{} - s{:0>2}e{:0>2} - {episode_name}", series.series_name, episode.aired_season, episode.aired_episode_number);
    }

    Ok(())
}
