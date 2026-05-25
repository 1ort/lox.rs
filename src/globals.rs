use crate::environment::Environment;
use crate::function::Function;
use crate::interruption::runtime_error;
use crate::object::{LoxObject, objref};

use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn build_globals() -> Environment {
    let mut globals = Environment::new_global();

    let clock = LoxObject::Function(Rc::new(Function::Native {
        identifier: "clock".to_string(),
        arity: 0,
        callable: |_| {
            let start = SystemTime::now();
            let millis = start
                .duration_since(UNIX_EPOCH)
                .expect("time should go forward")
                .as_millis() as f64;
            Ok(objref(LoxObject::Number(millis)))
        },
    }));
    globals.define("clock".to_string(), objref(clock));

    let sleep = LoxObject::Function(Rc::new(Function::Native {
        identifier: "sleep".to_string(),
        arity: 1,
        callable: |args| {
            let duration = args[0].clone();
            match *duration {
                LoxObject::Number(secs) => {
                    thread::sleep(Duration::from_secs_f64(secs));
                    Ok(LoxObject::Nil)
                }
                _ => Err(runtime_error(format!(
                    "Function 'sleep' expects number, but '{}' was provided.",
                    duration
                ))),
            }
            .map(objref)
        },
    }));

    globals.define("sleep".to_string(), objref(sleep));
    globals.define(
        "_version_".to_string(),
        objref(LoxObject::String(format!(
            "Lox.rs v{}",
            env!("CARGO_PKG_VERSION")
        ))),
    );

    globals
}
