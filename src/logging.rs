use tracing_subscriber::EnvFilter;

pub fn init_logging(level: tracing::Level) {
    let filter = EnvFilter::from_default_env().add_directive(level.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init()
        .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        init_logging(tracing::Level::INFO);
        assert!(true);
    }
}
