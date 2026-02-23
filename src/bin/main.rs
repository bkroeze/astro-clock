use astro_clock::cli::App;

fn main() {
    let app = App::new();
    if let Err(err) = app.run() {
        eprintln!("⚠️  Error: {}\n", err);
        std::process::exit(1);
    }
}
