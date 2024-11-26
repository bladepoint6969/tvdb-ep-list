# TVDB Episode Listings

[![Latest Version]][crates.io] ![License]

`tvdb-ep-list` is a command line application to generate Plex-compatible episode
names for use in naming files

## Usage

```text
Print an episode listing for the specified series

Usage: tvdb-ep-list [OPTIONS]

Options:
  -o, --ordering <ORDERING>  The Episode ordering to use [default: aired] [possible values: aired, dvd]
  -n, --name <NAME>          Name of a series to search for
  -i, --id <ID>              Series ID
  -l, --lang <LANG>          Language code for API Results [default: en]
  -k, --key <KEY>            Update configured API key
  -h, --help                 Print help
  -V, --version              Print version
```

The first time you run the application, enter your legacy API key with the `-k`
option. This will store the key in a configuration file for future use (the file
is located at `$HOME/.config/tvdb-ep-list/tvdb-ep-list.toml` on Linux and
`%APPDATA%\tvdb-ep-list\tvdb-ep-list.toml` on Windows).

## License

* [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0)
  ([LICENSE-APACHE](https://github.com/bladepoint6969/tvdb-ep-list/blob/HEAD/LICENSE-APACHE))

* [MIT License](https://opensource.org/licenses/MIT)
  ([LICENSE-MIT](https://github.com/bladepoint6969/tvdb-ep-list/blob/HEAD/LICENSE-MIT))

[crates.io]: https://crates.io/crates/tvdb-ep-list
[License]: https://img.shields.io/crates/l/tvdb-ep-list.svg?
[Latest Version]: https://img.shields.io/crates/v/tvdb-ep-list.svg?
