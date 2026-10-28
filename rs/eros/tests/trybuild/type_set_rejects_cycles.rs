eros::type_set! {
    type First = (std::fmt::Error,) || Second;
    type Second = First;
}

fn main() {}
