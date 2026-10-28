#![no_implicit_prelude]
#![deny(unused_braces)]

extern crate eros as renamed;

renamed::type_set! {
    type Errors = (::core::fmt::Error,);
    type Combined = Errors || Errors;
}

fn bailing() -> renamed::Result<()> {
    let value = 7;
    renamed::bail!("bail {value}")
}

fn ensuring() -> renamed::Result<()> {
    let value = 7;
    renamed::ensure!(false, "ensure {value}",);
    renamed::Result::Ok(())
}

fn main() {
    let _: renamed::ErrorUnion<Combined> = renamed::ErrorUnion::new(::core::fmt::Error);
    static ERROR: &str = "static message";
    let _: renamed::ErrorUnion = renamed::error!(renamed::MsgError::from_static(ERROR));
    const ROOT_ERROR: renamed::MsgError = renamed::MsgError::from_static("error");
    let _: renamed::ErrorUnion = renamed::error!(ROOT_ERROR);
    let _: renamed::Result<()> = (|| renamed::bail!(renamed::MsgError::from_static(ERROR)))();
    let _: renamed::Result<()> = (|| {
        renamed::ensure!(false, renamed::MsgError::from_static(ERROR));
        renamed::Result::Ok(())
    })();
    let _: renamed::ErrorUnion = renamed::error!("literal");
    let _: renamed::ErrorUnion = renamed::error!("formatted {}", 7);
    let value = 7;
    let _: renamed::ErrorUnion = renamed::error!("captured {value}");
    let _: renamed::ErrorUnion = renamed::error!(renamed::MsgError::from_static("expression"));
    let _: renamed::Result<(), (renamed::MsgError,)> =
        (|| renamed::bail!(renamed::MsgError::from_static(ERROR)))();
    let _: renamed::Result<(), (renamed::MsgError,)> = (|| renamed::bail!("value {}", 7))();
    let _: renamed::Result<(), (renamed::MsgError,)> = (|| {
        renamed::ensure!(false, "typed {value}");
        renamed::Result::Ok(())
    })();
    let _ = bailing();
    let _ = ensuring();
}
