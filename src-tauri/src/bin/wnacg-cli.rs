fn main() {
    if let Err(err) = wnacg_downloader_lib::run_cli() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}
