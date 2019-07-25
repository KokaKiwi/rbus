#[macro_export(crate)]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(_) => {}
            Err(err) => panic!("Expected Ok but got Err({:?})", err),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(_) => {}
            Err(err) => panic!("Expected Ok but got Err({:?}): {:?}", err, $msg),
        }
    };
}

#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Ok(value) => panic!("Expected Err but got Ok({:?})", value),
            Err(_) => {}
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(value) => panic!("Expected Err but got Ok({:?}): {:?}", value, $msg),
            Err(_) => {}
        }
    };
}
