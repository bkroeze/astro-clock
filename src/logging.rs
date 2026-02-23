use tracing_subscriber;

pub fn init_logging() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    subscriber.init();

    tracing::info!("Logging initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_init() {
        init_logging();
        // If no panic occurs, initialization was successful
        assert!(true);
    }
}
