use {
    sdl2::{ Sdl, VideoSubsystem, video::Window },
};

pub struct SdlContext {
    pub sdl:    Sdl,
    pub video:  VideoSubsystem,
    pub window: Window,
}

#[macro_export]
macro_rules! limit {
    ($val:ident, $max:expr) => {
        if $val > $max {
            panic!("Value {} exceeds limit of {} with value {}.", stringify!($val), $max, $val);
        }
    };
    ($val:ident, $max:expr, $err_msg:literal) => {
        if $val > $max {
            panic!("Value {} exceeds limit of {} with value {}: {}", stringify!($val), $max, $val, $err_msg);
        }
    };
    ($val:ident, $min:expr, $max:expr) => {
        if $val < $min && $val > $max {
            panic!("Value {} exceeds limit from {} to {} with value {}.", stringify!($val), $min, $max, $val);
        }
    };
    ($val:expr, $min:expr, $max:expr, $err_msg:literal) => {
        if $val < $min && $val > $max {
            panic!("Value {} exceeds limit from {} to {} with value {}: {}", stringify!($val), $min, $max, $val, $err_msg);
        }
    };
}
