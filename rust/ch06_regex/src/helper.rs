pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait SafeAdd: Sized {
    fn safe_add(&self, other: &Self) -> Option<Self>;
}

impl SafeAdd for usize {
    fn safe_add(&self, other: &Self) -> Option<Self> {
        self.checked_add(*other)
    }
}

pub fn safe_add<T, F, E>(dst: &mut T, src: &T, f: F) -> Result<(), E>
where
    T: SafeAdd,
    F: Fn() -> E,
{
    if let Some(n) = dst.safe_add(src) {
        *dst = n;
        Ok(())
    } else {
        Err(f())
    }
}
