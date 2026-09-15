fn main() {}

fn typed() -> eros::Result<(), (eros::MsgError,)> {
    eros::bail!(std::fmt::Error)
}
