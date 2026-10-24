pub trait ErrorCode {
    fn code(&self) -> &'static str;
}

pub trait ErrorDescription {
    fn message(&self) -> std::borrow::Cow<'static, str>;
    fn label(&self) -> Option<std::borrow::Cow<'static, str>> { None }
    fn help(&self) -> Option<std::borrow::Cow<'static, str>> { None }
}

pub trait ErrorInfo: ErrorCode + ErrorDescription {}
impl<T> ErrorInfo for T where T: ErrorCode + ErrorDescription {}