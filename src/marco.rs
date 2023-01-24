#[macro_export]
macro_rules! simple {
    ($x: expr) => {
        RESPType::SimpleString($x)
    };
}

#[macro_export]
macro_rules! err {
    ($x: expr) => {
        RESPType::Error($x)
    };
}

#[macro_export]
macro_rules! i64 {
    ($x: expr) => {
        RESPType::Integer($x)
    };
}

#[macro_export]
macro_rules! bulk {
    ($x: expr) => {
        RESPType::BulkString(($x).as_bytes().to_vec())
    };
}

#[macro_export]
macro_rules! array {
    ($($x: expr),* $(,)?) => {
        RESPType::Array(vec![$($x),*])
    };
}

#[macro_export]
macro_rules! none {
    () => {
        RESPType::None
    };
}