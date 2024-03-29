use core::convert::TryInto;

use crate::{Backend, Unstuffed};

/// A trait that describes how to stuff others and pointers into the pointer sized object.
///
/// This trait is what a user of this crate is expected to implement to use the crate for their own
/// pointer stuffing. It's usually implemented on ZSTs that only serve as stuffing strategies, but
/// it's also completely possible to implement it on the type in [`StuffingStrategy::Other`] directly
/// if possible.
///
/// The generic parameter `B` stands for the [`Backend`](`crate::Backend`) used by the strategy.
pub trait StuffingStrategy<B> {
    /// The type of the other.
    type Other: Copy;

    /// Stuff other data into a usize that is then put into the pointer. This operation
    /// must be infallible.
    fn stuff_other(inner: Self::Other) -> B;

    /// Extract the pointer data or other data
    /// # Safety
    /// `data` must contain data created by [`StuffingStrategy::stuff_other`].
    fn extract(data: B) -> Unstuffed<usize, Self::Other>;

    /// Stuff a pointer address into the pointer sized integer.
    ///
    /// This can be used to optimize away some of the unnecessary parts of the pointer or do other
    /// cursed things with it.
    ///
    /// The default implementation just returns the address directly.
    fn stuff_ptr(addr: usize) -> B;
}

impl<B> StuffingStrategy<B> for ()
where
    B: Backend + Default + TryInto<usize>,
    usize: TryInto<B>,
{
    type Other = ();

    fn stuff_other(_inner: Self::Other) -> B {
        B::default()
    }

    fn extract(data: B) -> Unstuffed<usize, Self::Other> {
        Unstuffed::Ptr(
            data.try_into()
                // note: this can't happen 🤔
                .unwrap_or_else(|_| panic!("Pointer value too big for usize")),
        )
    }

    fn stuff_ptr(addr: usize) -> B {
        addr.try_into()
            .unwrap_or_else(|_| panic!("Address in `stuff_ptr` too big"))
    }
}

#[cfg(test)]
pub(crate) mod test_strategies {
    use core::fmt::{Debug, Formatter};

    use super::StuffingStrategy;
    use crate::Unstuffed;

    macro_rules! impl_usize_max_zst {
        ($ty:ident) => {
            // this one lives in usize::MAX
            impl StuffingStrategy<usize> for $ty {
                type Other = Self;

                #[allow(clippy::forget_copy)]
                fn stuff_other(inner: Self::Other) -> usize {
                    core::mem::forget(inner);
                    usize::MAX
                }

                fn extract(data: usize) -> Unstuffed<usize, Self::Other> {
                    match data == usize::MAX {
                        true => Unstuffed::Other($ty),
                        false => Unstuffed::Ptr(data),
                    }
                }

                fn stuff_ptr(addr: usize) -> usize {
                    addr
                }
            }

            impl StuffingStrategy<u64> for $ty {
                type Other = Self;

                #[allow(clippy::forget_copy)]
                fn stuff_other(inner: Self::Other) -> u64 {
                    core::mem::forget(inner);
                    u64::MAX
                }

                fn extract(data: u64) -> Unstuffed<usize, Self::Other> {
                    match data == u64::MAX {
                        true => Unstuffed::Other($ty),
                        false => Unstuffed::Ptr(data as usize),
                    }
                }

                fn stuff_ptr(addr: usize) -> u64 {
                    addr as u64
                }
            }

            impl StuffingStrategy<u128> for $ty {
                type Other = Self;

                #[allow(clippy::forget_copy)]
                fn stuff_other(inner: Self::Other) -> u128 {
                    core::mem::forget(inner);
                    u128::MAX
                }

                fn extract(data: u128) -> Unstuffed<usize, Self::Other> {
                    match data == u128::MAX {
                        true => Unstuffed::Other($ty),
                        false => Unstuffed::Ptr(data as usize),
                    }
                }

                fn stuff_ptr(addr: usize) -> u128 {
                    addr as u128
                }
            }
        };
    }

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct EmptyInMax;

    impl_usize_max_zst!(EmptyInMax);

    #[derive(Clone, Copy)]
    pub struct HasDebug;

    impl Debug for HasDebug {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            f.write_str("hello!")
        }
    }

    impl_usize_max_zst!(HasDebug);
}
