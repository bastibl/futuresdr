use once_cell::sync::OnceCell;

pub mod config;
pub mod logging;
pub mod pmt;

pub fn init() {
    static INITIALIZED : OnceCell<()> = OnceCell::new();
    if INITIALIZED.set(()).is_err() {
        return;
    }

    logging::init();
    debug!("initialized");
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn runtime_init() {
        init();
        init();
        debug!("test 123");
    }
}
