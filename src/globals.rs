use crate::environment::Environment;
use crate::function::NativeFunction;
use crate::interruption::LoxError;
use crate::interruption::new_runtime_error;
use crate::object::LoxObject;

use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

// Prefix function names with "fun_" redundant
fn fun_clock(args: Vec<LoxObject>, _env: &Environment) -> Result<LoxObject, LoxError> {
    if !args.is_empty() {
        return Err(new_runtime_error(
            format!(
                "Function 'clock' takes 0 arguments, but {} provided",
                args.len()
            ),
            None,
        ));
    }
    let start = SystemTime::now();
    let millis = start
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward")
        .as_millis() as f64;
    Ok(LoxObject::Number(millis))
}
fn fun_sleep(args: Vec<LoxObject>, _env: &Environment) -> Result<LoxObject, LoxError> {
    if args.len() != 1 {
        return Err(new_runtime_error(
            format!(
                "Function 'sleep' takes 1 arguments, but {} provided",
                args.len()
            ),
            None,
        ));
    }

    let duration = args[0].clone();
    match duration {
        LoxObject::Number(secs) => {
            thread::sleep(Duration::from_secs_f64(secs));
            Ok(LoxObject::Nil)
        }
        _ => Err(new_runtime_error(
            format!(
                "Function 'sleep' expects number, but '{}' was provided.",
                duration
            ),
            None,
        )),
    }
}
fn fun_concat(args: Vec<LoxObject>, _env: &Environment) -> Result<LoxObject, LoxError> {
    if args.is_empty() {
        return Err(new_runtime_error(
            "Function 'concat' takes at least 1 arguments, but 0 provided".to_string(),
            None,
        ));
    }

    let res = args
        .iter()
        .fold(String::new(), |a, b| format!("{}{}", a, b));
    Ok(LoxObject::String(res))
}

fn debug_env(_args: Vec<LoxObject>, env: &Environment) -> Result<LoxObject, LoxError> {
    Ok(LoxObject::String(format!("{}", env)))
}

pub fn build_globals() -> Environment {
    let mut globals = Environment::new_global();

    // Repetitive code can be hidden in macro:
    // ```rust
    // macro_rules! define_native_function {
    //     ($name:ident, $callable:ident) => {
    //         globals.define(
    //             $name.to_string(),
    //             LoxObject::NativeFunction(Rc::new(NativeFunction {
    //                 name: $name.to_string(),
    //                 callable: $callable,
    //             })),
    //         );
    //     };
    // }
    // ```
    // And use it like this:
    // ```rust
    // define_native_function!(clock, fun_clock);
    // define_native_function!(sleep, fun_sleep);
    // define_native_function!(concat, fun_concat);
    // define_native_function!(dbgenv, debug_env);
    // ```
    globals.define(
        "clock".to_string(),
        LoxObject::NativeFunction(Rc::new(NativeFunction {
            name: "clock".to_string(),
            callable: fun_clock,
        })),
    );
    globals.define(
        "sleep".to_string(),
        LoxObject::NativeFunction(Rc::new(NativeFunction {
            name: "sleep".to_string(),
            callable: fun_sleep,
        })),
    );
    globals.define(
        "concat".to_string(),
        LoxObject::NativeFunction(Rc::new(NativeFunction {
            name: "concat".to_string(),
            callable: fun_concat,
        })),
    );
    globals.define(
        "dbgenv".to_string(),
        LoxObject::NativeFunction(Rc::new(NativeFunction {
            name: "dbgenv".to_string(),
            callable: debug_env,
        })),
    );
    globals.define(
        "_version_".to_string(),
        LoxObject::String(format!("Lox.rs v{}", env!("CARGO_PKG_VERSION"))),
    );

    globals
}
