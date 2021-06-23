use std::fmt;


#[macro_export]
macro_rules! error {
    ($message:expr) => {
        JdfError::generic($message)
    };
    ($message:expr, $($arg:tt)*) => {
        JdfError::generic(format!($message, $($arg)+).as_str())
    }
}

#[derive(Debug)]
pub enum JdfError {
    Generic(GenericError),
    IoError(std::io::Error)
}

impl JdfError {
    pub fn generic(message: &str) -> JdfError{
        JdfError::Generic(GenericError::new(message))
    }
}

impl From<std::io::Error> for JdfError {
    fn from(err: std::io::Error) -> JdfError {
        JdfError::IoError(err)
    }
}

impl fmt::Display for JdfError {
   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JdfError::Generic(ref err) => write!(f, "{}", err),
            JdfError::IoError(ref err) => write!(f, "{}", err)
        }
   }
}


#[derive(Debug)]
pub struct GenericError {
    message: String,
}


impl GenericError {
    pub fn new(message: &str) -> GenericError {
        GenericError {
            message: String::from(message),
        }
    }
}

impl<'a> fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error occur: {}", self.message)
    }
}

impl<'a> std::error::Error for GenericError {
    fn description(&self) -> &str {
        self.message.as_str()
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

