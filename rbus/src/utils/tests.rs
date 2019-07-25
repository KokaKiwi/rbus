#[macro_export(crate)]
macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(_) => {}
            None => panic!("Expected Some but got None"),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(_) => {}
            None => panic!("Expected Some but got None"),
        }
    };
}

#[macro_export]
macro_rules! assert_none {
    ($expr:expr) => {
        match $expr {
            Some(value) => panic!("Expected None but got Some({:?})", value),
            None => {}
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(value) => panic!("Expected None but got Some({:?}): {:?}", value, $msg),
            None => {}
        }
    };
}

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
