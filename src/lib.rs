#![no_std]
#![doc = include_str!("../README.md")]
#![cfg_attr(
    feature = "nightly",
    feature(unsize, fn_ptr_trait, unboxed_closures, fn_traits, tuple_trait)
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use alloc::boxed::Box;
use cc::{AsExtern, CallingConvention, IntoExtern, C};
use core::{ffi::c_void, marker::PhantomData};
use docfg::docfg;

extern crate alloc;

/// Calling conventions
pub mod cc;

/// A closure that can be sent through an FFI boundary.
pub struct Closure<T: ?Sized + AsExtern<Cc>, Cc: CallingConvention = C> {
    f: T::Extern,
    user_data: *mut c_void,
    destructor: Option<Cc::Destructor>,
    _phtm: PhantomData<T>,
}

impl<T: ?Sized + AsExtern<Cc>, Cc: CallingConvention> Closure<T, Cc> {
    /// Creates a new `Closure` from a Rust closure.
    ///
    /// ## Example
    /// ```
    /// # use ffi_closure::Closure;
    /// # use core::ffi::c_void;
    /// let mut square = Closure::<dyn FnMut(u32) -> u32>::new(|x| x * x);
    /// assert_eq!(square.call((3,)), 9);
    /// ```
    #[inline]
    pub fn new<F: IntoExtern<T, Cc>>(f: F) -> Self {
        return Box::new(f).into_extern();
    }

    /// Creates a new `Closure` from an external function pointer and user data.
    ///
    /// ## Example
    /// ```
    /// # use ffi_closure::Closure;
    /// # use core::ffi::c_void;
    /// unsafe extern "C" fn square(x: u32, _: *mut c_void) -> u32 {
    ///     return x * x
    /// }
    ///
    /// let mut square = unsafe { Closure::<dyn FnMut(u32) -> u32>::from_extern(square, core::ptr::null_mut(), None) };
    /// assert_eq!(square.call((3,)), 9);
    /// ```
    #[inline]
    pub unsafe fn from_extern(
        f: T::Extern,
        user_data: *mut c_void,
        destructor: Option<Cc::Destructor>,
    ) -> Self {
        return Self {
            f,
            user_data,
            destructor,
            _phtm: PhantomData,
        };
    }

    /// Returns `true` if this closure has a destructor, `false` otherwise
    #[inline(always)]
    pub fn has_destructor(&self) -> bool {
        return self.destructor.is_some();
    }

    /// Returns the user data of this `Closure`
    #[inline(always)]
    pub fn user_data(&self) -> *mut c_void {
        return self.user_data;
    }

    /// Returns the function pointer of this `Closure`
    #[inline(always)]
    pub fn fn_ptr(&self) -> T::Extern {
        return self.f;
    }

    /// Returns the function pointer and user data of this `Closure`
    ///
    /// ## Example
    /// ```
    /// # use ffi_closure::Closure;
    /// # use core::ffi::c_void;
    /// let mut square = Closure::<dyn FnMut(u32) -> u32>::new(|x| x * x);
    /// let (f, user_data): (unsafe extern "C" fn(u32, *mut c_void) -> u32, *mut c_void) = square.as_extern_parts();
    /// unsafe { assert_eq!(f(3, user_data), 9) }
    /// ```
    #[inline(always)]
    pub fn as_extern_parts(&self) -> (T::Extern, *mut c_void) {
        return (self.fn_ptr(), self.user_data);
    }

    /// Calls the `Closure`'s function pointer with the provided arguments and it's user data
    ///
    /// ## Example
    /// ```
    /// # use ffi_closure::Closure;
    /// # use core::ffi::c_void;
    /// let mut square = Closure::<dyn FnMut(u32) -> u32>::new(|x| x * x);
    /// assert_eq!(square.call((3,)), 9);
    /// ```
    #[inline(always)]
    pub fn call(&mut self, args: T::Args) -> T::Output {
        unsafe { T::call_mut(self.f, args, self.user_data) }
    }
}

#[docfg(feature = "nightly")]
impl<T: ?Sized + AsExtern<Cc>, Cc: CallingConvention> FnOnce<T::Args> for Closure<T, Cc>
where
    T::Args: core::marker::Tuple,
{
    type Output = <T as AsExtern<Cc>>::Output;

    #[inline(always)]
    extern "rust-call" fn call_once(self, args: T::Args) -> Self::Output {
        unsafe { T::call_mut(self.f, args, self.user_data) }
    }
}

#[docfg(feature = "nightly")]
impl<T: ?Sized + AsExtern<Cc>, Cc: CallingConvention> FnMut<T::Args> for Closure<T, Cc>
where
    T::Args: core::marker::Tuple,
{
    #[inline(always)]
    extern "rust-call" fn call_mut(&mut self, args: T::Args) -> Self::Output {
        unsafe { T::call_mut(self.f, args, self.user_data) }
    }
}

impl<T: ?Sized + AsExtern<Cc>, Cc: CallingConvention> Drop for Closure<T, Cc> {
    #[inline]
    fn drop(&mut self) {
        if let Some(f) = self.destructor.take() {
            unsafe { Cc::destroy(f, self.user_data) }
        }
    }
}

unsafe impl<T: ?Sized + AsExtern<Cc> + Send, Cc: CallingConvention> Send for Closure<T, Cc> {}
unsafe impl<T: ?Sized + AsExtern<Cc> + Sync, Cc: CallingConvention> Sync for Closure<T, Cc> {}

/// ```compile_fail
/// fn thread_safety_fail() {
///     let mut res = 0;
///     let mut closure1 = Closure::<dyn Send + FnMut()>::new(|| res += 1);
///     let mut closure2 = Closure::<dyn Send + FnMut()>::new(|| res += 1);
///
///     std::thread::spawn(move || closure1.call(()));
///     closure2.call(());
///
///     println!("{res}")
/// }
/// ```
///
/// ```compile_fail
/// fn lifetime_safety_fail() {
///     let res = AtomicU32::new(0);
///
///     let mut closure = Closure::<dyn Send + FnMut()>::new(|| {
///         res.fetch_add(1, Ordering::AcqRel);
///     });
///
///     std::thread::spawn(move || closure.call(()));
/// }
/// ```
#[doc(hidden)]
mod compile_fail {}
