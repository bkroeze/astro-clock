use astro_clock::cli::App;
use astro_clock::errors::Error;

fn main() -> Result<(), Error> {
    let app = App::new();
    app.run()
}
