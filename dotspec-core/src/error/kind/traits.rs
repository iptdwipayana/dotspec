use std::borrow::Cow;

pub trait ErrorCode {
    fn code(&self) -> &'static str;
}

pub trait ErrorDescription {
    fn message(&self) -> Cow<'static, str>;
    fn label(&self) -> Option<Cow<'static, str>> { None }
    fn help(&self) -> Option<Cow<'static, str>> { None }
}

pub trait ErrorInfo: ErrorCode + ErrorDescription {}
impl<T> ErrorInfo for T where T: ErrorCode + ErrorDescription {}