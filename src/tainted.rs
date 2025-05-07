use std::fmt::Debug;

#[derive(Debug)]
pub struct Tainted<T, Taint: Debug> {
    pub value: T,
    pub taint: Option<Taint>,
}

impl<T, Taint: Debug> Tainted<T, Taint> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Tainted<U, Taint> {
        Tainted {
            value: f(self.value),
            taint: self.taint,
        }
    }
}

#[macro_export]
macro_rules! untaint {
    ($val:expr, bool $error_var:ident) => {{
        let tainted = $val;
        if let Some(taint) = tainted.taint {
            $error_var = true;
            #[cfg(debug_assertions)]
            eprintln!("taint: {:?}", taint)
        }
        tainted.value
    }};
}
