use std::sync::{Once, ONCE_INIT};

use env_logger;

static INIT: Once = ONCE_INIT;

pub fn setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}
