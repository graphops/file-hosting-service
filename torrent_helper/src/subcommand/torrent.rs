use crate::common::*;

pub mod announce;
pub mod create;
pub mod from_link;
pub mod link;
pub mod piece_length;
pub mod show;
pub mod stats;
pub mod verify;

#[derive(StructOpt)]
#[structopt(
  help_message(consts::HELP_MESSAGE),
  version_message(consts::VERSION_MESSAGE),
  about("Subcommands related to the BitTorrent protocol.")
)]
pub(crate) enum Torrent {
  Announce(announce::Announce),
  Create(create::Create),
  FromLink(from_link::FromLink),
  Link(link::Link),
  #[structopt(alias = "piece-size")]
  PieceLength(piece_length::PieceLength),
  Show(show::Show),
  Stats(stats::Stats),
  Verify(verify::Verify),
}

impl Torrent {
  pub(crate) fn run(self, env: &mut Env, options: &Options) -> Result<(), Error> {
    match self {
      Self::Announce(announce) => announce.run(env),
      Self::Create(create) => create.run(env, options),
      Self::FromLink(from_link) => from_link.run(env, options),
      Self::Link(link) => link.run(env),
      Self::PieceLength(piece_length) => piece_length.run(env),
      Self::Show(show) => show.run(env),
      Self::Stats(stats) => stats.run(env, options),
      Self::Verify(verify) => verify.run(env, options),
    }
  }
}
