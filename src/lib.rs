#![no_std]
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
pub mod cc;

pub struct Closure<T: ?Sized, Cc: CallingConvention = C> {
    f: *const (),
    user_data: *mut c_void,
    destructor: Option<Cc::Destructor>,
    _phtm: PhantomData<T>,
}

// Generic
impl<T: ?Sized, Cc: CallingConvention> Closure<T, Cc> {
    #[inline(always)]
    pub fn user_data(&self) -> *mut c_void {
        return self.user_data;
    }

    #[inline(always)]
    pub fn has_destructor(&self) -> bool {
        return self.destructor.is_some();
    }
}

impl<T: ?Sized, Cc: CallingConvention> Closure<T, Cc> {
    #[inline]
    pub fn new<F: IntoExtern<T, Cc>>(f: F) -> Self {
        return Box::new(f).into_extern();
    }

    #[inline]
    pub unsafe fn from_extern<Args>(
        f: T::Extern,
        user_data: *mut c_void,
        destructor: Option<Cc::Destructor>,
    ) -> Self
    where
        T: AsExtern<Args, Cc>,
    {
        return Self {
            f: T::obfuscate(f),
            user_data,
            destructor,
            _phtm: PhantomData,
        };
    }

    #[inline(always)]
    pub fn fn_ptr<Args>(&self) -> T::Extern
    where
        T: AsExtern<Args, Cc>,
    {
        return unsafe { T::deobfuscate(self.f) };
    }

    #[inline(always)]
    pub fn as_extern_parts<Args>(&self) -> (T::Extern, *mut c_void)
    where
        T: AsExtern<Args, Cc>,
    {
        return (self.fn_ptr(), self.user_data);
    }

    #[inline(always)]
    pub fn call<Args>(&mut self, args: Args) -> T::Output
    where
        T: AsExtern<Args, Cc>,
    {
        unsafe { T::call_mut(self.f, args, self.user_data) }
    }
}

#[docfg(feature = "nightly")]
impl<T: ?Sized, Args: std::marker::Tuple, Cc: CallingConvention> FnOnce<Args> for Closure<T, Cc>
where
    T: AsExtern<Args, Cc>,
{
    type Output = <T as AsExtern<Args, Cc>>::Output;

    #[inline(always)]
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output {
        unsafe { T::call_mut(self.f, args, self.user_data) }
    }
}

#[docfg(feature = "nightly")]
impl<T: ?Sized, Args: std::marker::Tuple, Cc: CallingConvention> FnMut<Args> for Closure<T, Cc>
where
    T: AsExtern<Args, Cc>,
{
    #[inline(always)]
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        unsafe { T::call_mut(self.f, args, self.user_data) }
    }
}

impl<T: ?Sized, Cc: CallingConvention> Drop for Closure<T, Cc> {
    #[inline]
    fn drop(&mut self) {
        if let Some(f) = self.destructor.take() {
            unsafe { Cc::destroy(f, self.user_data) }
        }
    }
}

unsafe impl<T: ?Sized + Send, Cc: CallingConvention> Send for Closure<T, Cc> {}
unsafe impl<T: ?Sized + Sync, Cc: CallingConvention> Sync for Closure<T, Cc> {}
