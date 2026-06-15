use crate::{ErrorUnion, type_set::TypeSet};

pub trait LogExt<O> {
    fn log_error(self) -> O;
    fn log_warn(self) -> O;
    fn log_info(self) -> O;
    fn log_debug(self) -> O;
    fn log_trace(self) -> O;
}

impl<T, E> LogExt<Result<T, ErrorUnion<E>>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn log_error(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_error();
        })
    }

    fn log_warn(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_warn();
        })
    }

    fn log_info(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_info();
        })
    }

    fn log_debug(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_debug();
        })
    }

    fn log_trace(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_trace();
        })
    }
}
