#![deny(clippy::all, clippy::pedantic)]
#![allow(
  clippy::float_cmp,
  clippy::needless_lifetimes,
  clippy::needless_pass_by_value,
  clippy::non_ascii_literal,
  clippy::struct_excessive_bools,
  clippy::too_many_lines,
  clippy::unseparated_literal_suffix,
  clippy::wildcard_imports,
  clippy::large_enum_variant,
  clippy::module_name_repetitions
)]

#[cfg(test)]
#[macro_use]
mod assert_matches;

#[macro_use]
mod errln;

#[macro_use]
mod err;

#[macro_use]
mod out;

#[macro_use]
mod outln;

#[cfg(test)]
#[macro_use]
mod test_env;

#[cfg(test)]
mod test_env_builder;

#[cfg(test)]
mod capture;

mod arguments;
mod bytes;
pub mod common;
pub mod consts;
pub mod env;
mod error;
mod file_error;
mod file_info;
mod file_path;
mod file_status;
mod files;
pub mod hasher;
mod host_port;
mod host_port_parse_error;
pub mod info;
pub mod infohash;
mod input;
mod input_stream;
pub mod input_target;
pub mod into_u64;
pub mod into_usize;
mod invariant;
mod lint;
mod linter;
pub mod magnet_link;
mod magnet_link_parse_error;
mod md5_digest;
pub mod metainfo;
mod metainfo_error;
mod mode;
pub mod options;
mod output_stream;
pub mod output_target;
mod peer;
pub mod piece_length_picker;
mod piece_list;
mod platform;
mod platform_interface;
mod print;
mod reckoner;
mod run;
mod sha1_digest;
mod shell;
mod sort_key;
mod sort_order;
mod sort_spec;
mod status;
mod step;
mod style;
pub mod subcommand;
mod table;
pub mod torrent_summary;
mod tracker;
pub mod use_color;
mod verifier;
pub mod walker;
mod xor_args;

pub use run::run;
