#![no_implicit_prelude]

extern crate eros as renamed;

fn bailing() -> renamed::Result<()> {
    renamed::bail!("bail {}", 7)
}

fn ensuring() -> renamed::Result<()> {
    renamed::ensure!(false, "ensure",);
    renamed::Result::Ok(())
}

fn main() {
    let _: renamed::ErrorUnion = renamed::error!("literal");
    let _: renamed::ErrorUnion = renamed::error!("formatted {}", 7);
    let _: renamed::ErrorUnion = renamed::error!(renamed::StrError::Static("expression"));
    let _ = bailing();
    let _ = ensuring();
}
