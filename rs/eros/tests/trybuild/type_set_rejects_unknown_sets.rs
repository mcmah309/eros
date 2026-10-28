type Outside = (std::fmt::Error,);

eros::type_set! {
    type Errors = Outside || (std::io::Error,);
}

fn main() {}
