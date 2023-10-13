fn main() {
  if let Err(code) = torrent_helper::run() {
    std::process::exit(code);
  }
}
