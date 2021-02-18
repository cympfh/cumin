#[macro_export]
macro_rules! assert_args_eq {
    ($name:expr, $given:expr, $expected:expr) => {
        if $given != $expected {
            bail!(
                "ArgumentError: wrong number of arguments for `{}` (given {}, expected {})",
                $name,
                $given,
                $expected
            );
        }
    };
}

#[macro_export]
macro_rules! assert_args_leq {
    ($name:expr, $given:expr, $expected:expr) => {
        if $given > $expected {
            bail!(
                "ArgumentError: wrong number of arguments for `{}` (given {}, expected <={})",
                $name,
                $given,
                $expected
            );
        }
    };
}

#[macro_export]
macro_rules! bail_type_error {
    (compute $x:tt $op:tt $y:tt) => {
        bail!("TypeError: Cant compute {:?} `{}` {:?}.", $x, $op, $y);
    };
    (compute $op:tt $x:tt) => {
        bail!("TypeError: Cant compute `{}` {:?}.", $op, $x);
    };
}
