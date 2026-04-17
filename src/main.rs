fn main() {
    if let Err(err) = battlezone::app::run() {
        eprintln!("battlezone: {err:#}");
        std::process::exit(1);
    }
}
