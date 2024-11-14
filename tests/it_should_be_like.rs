#[test]
fn it_should_be_like() {
    #[allow(clippy::unwrap_used)]
    let res = from_utf8_error().unwrap_err();
    println!("{res}");
    match res {
        Error::BizError(biz_error) => {
            assert_eq!(biz_error.code(), 2001);
            assert_eq!(
                biz_error.msg().expect("msg is None"),
                "FromUtf8Error Error"
            );
        }
    }
}

use core::panic::Location;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error(transparent)]
    BizError(#[from] BizError),
}

#[derive(Default, ThisError)]
pub struct BizError
// where C: BizCause
{
    /// for outer
    pub code: u16,
    pub msg:  Option<String>,

    /// for inner
    pub name:    String,
    pub cause:   Option<Box<dyn core::error::Error>>,
    pub context: Vec<String>,
    pub loc:     Option<&'static Location<'static>>,
    // pub _phantom: PhantomData<C>
}

impl core::fmt::Display for BizError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error: {}. Code: {}.", self.name, self.code,)?;
        if let Some(msg) = self.msg.as_ref() {
            write!(f, "\nText = {msg}.")?;
        }
        Ok(())
    }
}

impl core::fmt::Debug for BizError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error: {}. Code: {}.", self.name, self.code,)?;
        if let Some(msg) = self.msg.as_ref() {
            write!(f, "\nText = {msg}.")?;
        }

        if let Some(cause) = &self.cause {
            write!(
                f,
                "\nCaused by: {} \nAt {}",
                cause,
                self.loc.expect("loc is None")
            )?;
        } else {
            write!(f, "\nAt {}", self.loc.expect("loc is None"))?;
        }

        if !self.context.is_empty() {
            write!(f, "\nContext: {:?}", self.context.join(", "))?;
        }

        Ok(())
    }
}

//----------------------------------------------------------------

// #[derive(Debug, BizError)]
// pub enum ApiError {
//     #[bizerror(2001, "FromUtf8Error Error")]
//     FromUtf8Error(std::string::FromUtf8Error),
// }

//----------------------------------------------------------------
// Turn into implementations

impl From<std::string::FromUtf8Error> for BizError {
    #[track_caller]
    fn from(err: std::string::FromUtf8Error) -> Self {
        let loc = core::panic::Location::caller();
        Self {
            code:    2001,
            msg:     Some("FromUtf8Error Error".to_string()),
            name:    "FromUtf8Error".to_string(),
            cause:   Some(Box::new(err)),
            context: vec![],
            loc:     Some(loc),
        }
    }
}

impl BizError {
    #[must_use]
    pub const fn with_loc(mut self, loc: &'static Location<'static>) -> Self {
        self.loc = Some(loc);
        self
    }

    pub const fn code(&self) -> u16 {
        self.code
    }

    pub const fn msg(&self) -> Option<&String> {
        self.msg.as_ref()
    }
}
// ----------------------------------------------------------------

fn from_utf8_error() -> Result<String, Error> {
    let bytes = vec![0, 159];
    let s = String::from_utf8(bytes).map_biz()?;

    Ok(s)
}

pub trait ResultExt<T, E> {
    fn map_biz(self) -> Result<T, BizError>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: core::error::Error + 'static,
    BizError: From<E>,
{
    #[track_caller]
    fn map_biz(self) -> Result<T, BizError> {
        let loc = core::panic::Location::caller();
        self.map_err(|e| BizError::from(e).with_loc(loc))
    }
}
