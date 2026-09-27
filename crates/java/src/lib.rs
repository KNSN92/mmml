
macro_rules! platform {
    { $first_target_os:literal => $first_value:expr, $($target_os:literal => $value:expr,)* _ => $other_value:expr } => {
        if cfg!(target_os = $first_target_os) {
            $first_value
        } $( else if cfg!(target_os = $target_os) {
            $value
        } )* else {
            $other_value
        }
    }
}

macro_rules! arch {
    { $first_target_arch:literal => $first_value:expr, $($target_arch:literal => $value:expr,)* _ => $other_value:expr } => {
        if cfg!(target_arch = $first_target_arch) {
            $first_value
        } $( else if cfg!(target_arch = $target_arch) {
            $value
        } )* else {
            $other_value
        }
    }
}

pub mod distribution;
mod extract;
mod runtime;

pub use runtime::*;
