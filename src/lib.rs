pub mod cli;
pub mod config;
pub mod errors;
pub mod logging;

pub use cli::App;
pub use errors::Error;
pub use logging::init_logging;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

fn main() {
    if let Err(err) = astro_clock() {
        eprintln!("⚠️  Error: {}\n", err);
        std::process::exit(1);
    }
}

fn astro_clock() -> Result<(), Error> {
    init_logging();

    println!("Astro Clock v0.1.0");
    println!("Configuration: Not yet implemented");

    Ok(())
}
