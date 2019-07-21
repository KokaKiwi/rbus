#![allow(dead_code)]

pub fn bt<T, E: std::fmt::Debug>(expr: Result<T, E>) -> Result<T, E> {
    use backtrace::Backtrace;

    match expr {
        Ok(value) => Ok(value),
        Err(err) => {
            let bt = Backtrace::new();
            println!("{:?}", bt);
            dbg!(&err);

            Err(err)
        }
    }
}
