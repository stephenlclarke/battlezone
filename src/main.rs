mod app;
mod game;
mod kitty;
mod math;
mod render;
mod terminal;

fn main() {
    if let Err(err) = app::run() {
        eprintln!("battlezone-tty: {err:#}");
        std::process::exit(1);
    }
}
