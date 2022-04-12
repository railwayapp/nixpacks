#[macro_export]
macro_rules! chain {
    ($x:expr => $($func:expr),*) => {
        {
            $x$(.and_then($func))*
        }
    };
    ($x:expr => $(($($func:expr),*));+) => {
        {
            None$(.or_else(|| chain!($x => $($func),*)))*
        }
    }
}
