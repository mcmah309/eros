use eros::ErrorUnion;

fn single(_: Option<ErrorUnion<(u16,)>>) {}

fn non_error_first(_: Option<ErrorUnion<(u16, std::io::Error)>>) {}

fn non_error_last(_: Option<ErrorUnion<(std::io::Error, u16)>>) {}

fn result_alias(_: Option<eros::Result<(), (std::io::Error, u16)>>) {}

type E = std::fmt::Error;

fn largest_tuple(
    _: Option<
        ErrorUnion<(
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            E,
            u16,
        )>,
    >,
) {
}

fn main() {}
