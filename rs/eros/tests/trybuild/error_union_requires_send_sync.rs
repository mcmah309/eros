use eros::ErrorUnion;
use std::{cell::Cell, fmt, rc::Rc};

#[derive(Debug)]
struct NotSendSync(Rc<()>);

impl fmt::Display for NotSendSync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for NotSendSync {}

#[derive(Debug)]
struct NotSync(Cell<()>);

impl fmt::Display for NotSync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for NotSync {}

fn not_send_sync(_: Option<ErrorUnion<(std::io::Error, NotSendSync)>>) {}

fn not_sync(_: Option<ErrorUnion<(std::io::Error, NotSync)>>) {}

fn main() {}
