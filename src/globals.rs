use crate::environment::Environment;
use crate::function::Function;
use crate::interruption::runtime_error;
use crate::object::LoxObject;

use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn build_globals() -> Environment {
    let mut globals = Environment::new();

    let clock = LoxObject::Function(Function::Native {
        identifier: "clock".to_string(),
        arity: 0,
        callable: |_| {
            let start = SystemTime::now();
            let millis = start
                .duration_since(UNIX_EPOCH)
                .expect("time should go forward")
                .as_millis() as f64;
            Ok(LoxObject::Number(millis))
        },
    });
    globals.define("clock".to_string(), clock);

    let sleep = LoxObject::Function(Function::Native {
        identifier: "sleep".to_string(),
        arity: 1,
        callable: |args| {
            let duration = args[0].clone();
            match duration {
                LoxObject::Number(secs) => {
                    thread::sleep(Duration::from_secs_f64(secs));
                    Ok(LoxObject::Nil)
                }
                _ => Err(runtime_error(format!(
                    "Function 'sleep' expects number, but '{}' was provided.",
                    duration.format()
                ))),
            }
        },
    });

    globals.define("sleep".to_string(), sleep);
    globals.define(
        "_version_".to_string(),
        LoxObject::String(format!("Lox.rs v{}", env!("CARGO_PKG_VERSION"))),
    );

    globals
}
