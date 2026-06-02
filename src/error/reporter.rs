use crate::error::error::LoxError;

pub trait ErrorReporter {
    fn report(&self, err: &LoxError) {
        eprintln!("{err}")
    }
}
