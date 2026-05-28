use crate::environment::Environment;
use crate::function::Function;
use crate::interruption::Interruption;
use crate::interruption::runtime_error;
use crate::object::LoxObject;

use std::fmt::format;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

fn fun_clock(args: Vec<LoxObject>, _env: &Environment) -> Result<LoxObject, Interruption> {
    if args.len() != 1 {
        return Err(runtime_error(format!(
            "Function 'clock' takes 0 arguments, but {} provided",
            args.len()
        )));
    }
    let start = SystemTime::now();
    let millis = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_millis() as f64;
    Ok(LoxObject::Number(millis))
}
fn fun_sleep(args: Vec<LoxObject>, _env: &Environment) -> Result<LoxObject, Interruption> {
    if args.len() != 1 {
        return Err(runtime_error(format!(
            "Function 'sleep' takes 1 arguments, but {} provided",
            args.len()
        )));
    }

    let duration = args[0].clone();
    match duration {
        LoxObject::Number(secs) => {
            thread::sleep(Duration::from_secs_f64(secs));
            Ok(LoxObject::Nil)
        }
        _ => Err(runtime_error(format!(
            "Function 'sleep' expects number, but '{}' was provided.",
            duration
        ))),
    }
}
fn fun_concat(args: Vec<LoxObject>, _env: &Environment) -> Result<LoxObject, Interruption> {
    if args.is_empty() {
        return Err(runtime_error(
            "Function 'concat' takes at least 1 arguments, but 0 provided".to_string(),
        ));
    }

    let res = args
        .iter()
        .fold(String::new(), |a, b| format!("{}{}", a, b));
    Ok(LoxObject::String(res))
}

fn debug_env(args: Vec<LoxObject>, env: &Environment) -> Result<LoxObject, Interruption> {
    Ok(LoxObject::String(format!("{}", env)))
}

pub fn build_globals() -> Environment {
    let mut globals = Environment::new_global();

    globals.define(
        "clock".to_string(),
        LoxObject::Function(Rc::new(Function::Native {
            identifier: "clock".to_string(),
            callable: fun_clock,
        })),
    );
    globals.define(
        "sleep".to_string(),
        LoxObject::Function(Rc::new(Function::Native {
            identifier: "sleep".to_string(),
            callable: fun_sleep,
        })),
    );
    globals.define(
        "concat".to_string(),
        LoxObject::Function(Rc::new(Function::Native {
            identifier: "concat".to_string(),
            callable: fun_concat,
        })),
    );
    globals.define(
        "dbgenv".to_string(),
        LoxObject::Function(Rc::new(Function::Native {
            identifier: "dbgenv".to_string(),
            callable: debug_env,
        })),
    );
    globals.define(
        "_version_".to_string(),
        LoxObject::String(format!("Lox.rs v{}", env!("CARGO_PKG_VERSION"))),
    );

    globals
}
