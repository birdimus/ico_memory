pub trait MaybeNull: Sized {
    #[inline]
    fn is_null(&self) -> bool;
    #[inline]
    fn null() -> Self;
    /// Takes the value out , leaving a null in its place.
    #[inline]
    fn take(&mut self) -> Self;
    #[inline]
    fn replace(&mut self, new: Self) -> Self;
}

#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Nullable<T> {
    value: T,
}
impl<T> Clone for Nullable<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        return Nullable::new(self.value.clone());
    }
}

impl<T> Nullable<T> {
    #[inline]
    pub const fn new(value: T) -> Nullable<T> {
        return Nullable { value: value };
    }
}
impl<T: MaybeNull> Nullable<T> {
    #[inline]
    pub fn null() -> Nullable<T> {
        return Nullable {
            value: MaybeNull::null(),
        };
    }
    #[inline]
    pub fn is_null(&self) -> bool {
        return self.value.is_null();
    }
    #[inline]
    pub fn get(self) -> Option<T> {
        if self.is_null() {
            return None;
        }
        return Some(self.value);
    }
    #[inline]
    pub fn get_ref(&self) -> Option<&T> {
        if self.is_null() {
            return None;
        }
        return Some(&self.value);
    }
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_null() {
            return None;
        }
        return Some(&mut self.value);
    }
    #[inline]
    pub fn take(&mut self) -> Nullable<T> {
        return Nullable {
            value: self.value.take(),
        };
    }
    #[inline]
    pub fn replace(&mut self, value: T) -> Nullable<T> {
        return Nullable {
            value: self.value.replace(value),
        };
    }
    pub fn map<U: MaybeNull, F>(self, f: F) -> Nullable<U>
    where
        F: FnOnce(T) -> U,
    {
        if !self.is_null() {
            return Nullable::<U> {
                value: f(self.value),
            };
        }
        return Nullable::<U> {
            value: MaybeNull::null(),
        };
    }
}
