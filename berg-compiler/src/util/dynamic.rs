use std;
use std::any::TypeId;

///
/// A value that can be any Rust value underneath.
///
/// Example:
///
///    #[derive(Debug)]
///    struct Counter<'a>(&'a mut u32);
///
///    enum BergError {
///        BadType,
///    }
///
///    impl<'a> Dynamic for Counter<'a> {
///        type StaticType = Counter<'static>;
///    }
///    impl Dynamic for bool {
///        type StaticType = Self;
///    }
///    fn increment<'a, V: Dynamic + 'a>(value: V) -> Result<Counter<'a>, V> {
///        let value = value.downcast::<Counter<'a>>()?;
///        *value.0 += 1;
///        Ok(value)
///    }
///    fn main() {
///        let mut x: u32 = 10;
///        let counter = Counter(&mut x);
///        let y: bool = true;
///        println!("{:?}", increment(counter)); // Prints Ok(Counter(11))
///        println!("{:?}", increment(y)); // Prints Err(true)
///    }
///
pub trait Dynamic {
    ///
    /// This type will be used for the dynamic Berg value. This exists so that
    /// types with lifetimes can be given a type id.
    ///
    /// This exists mainly to make it easy to build type_id implementations for
    /// types with both lifetimes and generic type parameters.
    ///
    type StaticType: 'static;

    ///
    /// The TypeId of this type, so is() and downcast() will work.
    ///
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self::StaticType>()
    }

    ///
    /// Check whether this value is of the given type.
    ///
    fn is<T: Dynamic>(&self) -> bool
    where
        Self: Sized,
    {
        self.type_id() == TypeId::of::<T::StaticType>()
    }

    ///
    /// Downcast a value (or return an error).
    ///
    /// This will only ever succeed on something that is already the right type,
    /// and exists to make it possible to write methods against generic parameters.
    /// For example: )
    ///
    fn downcast<T: Dynamic + Sized>(mut self) -> Result<T, Self>
    where
        Self: Sized,
    {
        if self.is::<T>() {
            Ok(unsafe { std::ptr::read(&mut self as *mut Self as *mut T) })
        } else {
            Err(self)
        }
    }

    ///
    /// Downcast a reference to the given type (or return an error).
    ///
    fn downcast_ref<T: Dynamic + Sized>(&self) -> Result<&T, ()> {
        if self.type_id() == TypeId::of::<T::StaticType>() {
            unsafe { Ok(&*(self as *const Self as *const T)) }
        } else {
            Err(())
        }
    }

    ///
    /// Downcast a reference to the given type (or return an error).
    ///
    fn downcast_mut<T: Dynamic + Sized>(&mut self) -> Result<&mut T, ()> {
        if self.type_id() == TypeId::of::<T::StaticType>() {
            unsafe { Ok(&mut *(self as *mut Self as *mut T)) }
        } else {
            Err(())
        }
    }
}
