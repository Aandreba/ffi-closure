#![allow(non_snake_case)]

use super::Closure;
use alloc::boxed::Box;
use core::ffi::c_void;
#[allow(unused_imports)]
use docfg::docfg;

pub trait CallingConvention: sealed::Sealed {
    type Destructor: Copy;

    unsafe fn destroy(f: Self::Destructor, user_data: *mut c_void);
}

pub trait AsExtern<Cc: CallingConvention>: sealed::Sealed {
    type Args;
    type Extern: Copy;
    type Output;

    unsafe fn call_mut(f: Self::Extern, args: Self::Args, user_data: *mut c_void) -> Self::Output;
}

pub trait IntoExtern<Ext: ?Sized + AsExtern<Cc>, Cc: CallingConvention> {
    fn into_extern(self: Box<Self>) -> Closure<Ext, Cc>;
}

macro_rules! impl_ {
    (
        $abi:literal as $ident:ident => ($($arg:ident),*) $(+ $($trait:ident),+)?
    ) => {
        impl<'a, $($arg,)* __OUT__> AsExtern<$ident> for dyn 'a + $($($trait+)+)? FnMut($($arg,)*) -> __OUT__ {
            type Args = ($($arg,)*);
            type Extern = unsafe extern $abi fn($($arg,)* *mut c_void) -> __OUT__;
            type Output = __OUT__;

            #[inline(always)]
            unsafe fn call_mut(f: Self::Extern, ($($arg,)*): ($($arg,)*), user_data: *mut c_void) -> Self::Output {
                (f)($($arg,)* user_data)
            }
        }

        impl<'a, $($arg,)* __F__: 'a + $($($trait+)+)? FnMut($($arg,)*) -> __OUT__, __OUT__> IntoExtern<dyn 'a + $($($trait+)+)? FnMut($($arg,)*) -> __OUT__, $ident> for __F__ {
            #[inline]
            fn into_extern(self: Box<Self>) -> Closure<dyn 'a + $($($trait+)+)? FnMut($($arg,)*) -> __OUT__, $ident> {
                unsafe extern $abi fn call<$($arg,)* __F__: FnMut($($arg,)*) -> __OUT__, __OUT__>($($arg: $arg,)* user_data: *mut c_void) -> __OUT__ {
                    return (&mut *user_data.cast::<__F__>())($($arg),*)
                }

                unsafe extern $abi fn destroy<__F__>(user_data: *mut c_void) {
                    drop(Box::from_raw(user_data.cast::<__F__>()))
                }

                let user_data = Box::into_raw(self).cast();
                return unsafe {
                    Closure::from_extern(
                        call::<$($arg,)* Self, __OUT__> as unsafe extern $abi fn($($arg,)* *mut c_void) -> __OUT__,
                        user_data,
                        Some(destroy::<Self> as unsafe extern $abi fn(*mut c_void))
                    )
                }
            }
        }
    }
}

macro_rules! impl_fn {
    (
        $(#[cfg($cfg:meta)])?
        $abi:literal as $ident:ident {
            $(
                ($($arg:ident),*)
            ),+ $(,)?
        }
    ) => {
        $(
            impl_! { $abi as $ident => ($($arg),*) }
            impl_! { $abi as $ident => ($($arg),*) + Send }
            impl_! { $abi as $ident => ($($arg),*) + Sync }
            impl_! { $abi as $ident => ($($arg),*) + Send, Sync }
        )+
    };
}

macro_rules! cc {
    ($($(#[cfg($cfg:meta)])? $abi:literal as $ident:ident),+ $(,)?) => {
        $(
            $(
                #[cfg(any(docsrs, $cfg))]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
            )?
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
            #[doc = concat!("The \"", $abi, "\" calling convention")]
            pub struct $ident;

            $(
                #[cfg(any(docsrs, $cfg))]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
            )?
            impl CallingConvention for $ident {
                type Destructor = unsafe extern $abi fn(*mut c_void);

                #[inline(always)]
                unsafe fn destroy(f: Self::Destructor, user_data: *mut c_void) {
                    (f)(user_data)
                }
            }

            $(
                #[cfg(any(docsrs, $cfg))]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
            )?
            impl sealed::Sealed for $ident {}

            $(
                #[cfg(any(docsrs, $cfg))]
                #[cfg_attr(docsrs, doc(cfg($cfg)))]
            )?
            impl_fn! {
                $abi as $ident {
                    (),
                    (A),
                    (A, B),
                    (A, B, C_),
                    (A, B, C_, D),
                    (A, B, C_, D, E),
                    (A, B, C_, D, E, F),
                    (A, B, C_, D, E, F, G),
                    (A, B, C_, D, E, F, G, H),
                    (A, B, C_, D, E, F, G, H, I),
                    (A, B, C_, D, E, F, G, H, I, J),
                    (A, B, C_, D, E, F, G, H, I, J, K),
                    (A, B, C_, D, E, F, G, H, I, J, K, L),
                    (A, B, C_, D, E, F, G, H, I, J, K, L, M),
                    (A, B, C_, D, E, F, G, H, I, J, K, L, M, N),
                }
            }
        )+
    };
}

macro_rules! seal {
    (
        $(
            ($($arg:ident),*)
        ),+ $(,)?
    ) => {
        $(
            impl<'a, $($arg,)* __OUT__> sealed::Sealed for dyn 'a + FnMut($($arg,)*) -> __OUT__ {}
            impl<'a, $($arg,)* __OUT__> sealed::Sealed for dyn 'a + Send + FnMut($($arg,)*) -> __OUT__ {}
            impl<'a, $($arg,)* __OUT__> sealed::Sealed for dyn 'a + Sync + FnMut($($arg,)*) -> __OUT__ {}
            impl<'a, $($arg,)* __OUT__> sealed::Sealed for dyn 'a + Send + Sync + FnMut($($arg,)*) -> __OUT__ {}
        )+
    };
}

// https://doc.rust-lang.org/nomicon/ffi.html#foreign-calling-conventions
cc! {
    #[cfg(any(target_arch = "x86"))]
    "stdcall" as StdCall,
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    "aapcs" as Aapcs,
    "cdecl" as Cdecl,
    #[cfg(any(target_arch = "x86"))]
    "fastcall" as FastCall,
    #[cfg(any(target_arch = "x86"))]
    "thiscall" as ThisCall,
    "Rust" as Rust,
    "system" as System,
    "C" as C,
    #[cfg(all(windows, any(target_arch = "x86_64", target_arch = "aarch64")))]
    "win64" as Win64,
    #[cfg(any(target_arch = "x86_64"))]
    "sysv64" as Sysv64
}

seal! {
    (),
    (A),
    (A, B),
    (A, B, C_),
    (A, B, C_, D),
    (A, B, C_, D, E),
    (A, B, C_, D, E, F),
    (A, B, C_, D, E, F, G),
    (A, B, C_, D, E, F, G, H),
    (A, B, C_, D, E, F, G, H, I),
    (A, B, C_, D, E, F, G, H, I, J),
    (A, B, C_, D, E, F, G, H, I, J, K),
    (A, B, C_, D, E, F, G, H, I, J, K, L),
    (A, B, C_, D, E, F, G, H, I, J, K, L, M),
    (A, B, C_, D, E, F, G, H, I, J, K, L, M, N),
}

mod sealed {
    pub trait Sealed {}
}
