use std::sync::Once;

static INIT_LOGGER: Once = Once::new();

pub(crate) fn init_logger() {
    INIT_LOGGER.call_once(|| {
        let _ = env_logger::Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .is_test(true)
            .try_init();
    });
}
