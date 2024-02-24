# ffi-closure - Send and Receive closures through FFI

## Features

- Simple to use API
- Allows sending Rust functions & closures through FFI boundary
- `no-std` compatible
- Supports multiple calling conventions (defaults to C calling convention)
- Simplifies usage and interaction with FFI _closure-like_ functions
- Closures can have destructors
- Can specify closure's thread-safety level (None, `Send`, `Sync` or `Send + Sync`)

## Examples

**Export closure**

```rust
use ffi_closure::Closure;
use std::ffi::c_void;

# mod sys {
#     use core::ffi::c_void;
#     #[no_mangle]
#     unsafe extern "C" fn some_lib_fn(f: unsafe extern "C" fn(i8, *mut c_void), user_data: *mut c_void) { f(-1, user_data) }
# }

extern "C" {
    fn some_lib_fn(f: unsafe extern "C" fn(i8, *mut c_void), user_data: *mut c_void);
}

pub fn main() {
    let weight = std::env::args().len() as i8;
    let closure = Closure::<dyn FnMut(i8)>::new(move |x| println!("{}", x * weight));

    let (f, user_data): (unsafe extern "C" fn(i8, *mut c_void), *mut c_void) = closure.as_extern_parts();
    unsafe {
        some_lib_fn(f, user_data);
    }
}

```

**Exported closure w/ system calling convention**

```rust
use ffi_closure::{Closure, cc::System};
use std::ffi::c_void;

# mod sys {
#     use core::ffi::c_void;
#     #[no_mangle]
#     unsafe extern "C" fn some_lib_fn(f: unsafe extern "C" fn(i8, *mut c_void), user_data: *mut c_void) { f(-1, user_data) }
# }

extern "C" {
    fn some_lib_fn(f: unsafe extern "system" fn(i8, *mut c_void), user_data: *mut c_void);
}

pub fn main() {
    let weight = std::env::args().len() as i8;
    let closure = Closure::<dyn FnMut(i8), System>::new(move |x| println!("{}", x * weight));

    let (f, user_data): (unsafe extern "system" fn(i8, *mut c_void), *mut c_void) = closure.as_extern_parts();
    unsafe {
        some_lib_fn(f, user_data);
    }
}
```

**Imported closure**

```rust
use ffi_closure::Closure;
use std::ffi::c_void;

#[no_mangle]
pub extern "C" fn some_lib_fn(
    f: unsafe extern "C" fn(i8, *mut c_void),
    user_data: *mut c_void,
) {
    let mut f = unsafe { Closure::<dyn FnMut(i8)>::from_extern(f, user_data, None) };
    for i in 0..10 {
        f(i);
    }
}

```

**Imported, thread-safe closure**

```rust
use ffi_closure::{Closure};
use std::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn some_lib_fn(
    f: unsafe extern "C" fn(i8, *mut c_void),
    user_data: *mut c_void,
) {
    // SAFETY: Caller must ensure the passed function pointer and user data are thread-safe.
    let mut f = unsafe { Closure::<dyn Send + FnMut(i8)>::from_extern(f, user_data, None) };

    std::thread::spawn(move || {
        for i in 0..10 {
            f(i);
        }
    });
}

```

> **Note**\
> For `Closure` to implement `FnOnce` and `FnMut`, the 'nightly' feature must be enabled (and thus, the program must be built with nightly Rust).
> For a non-nightly version, use `Closure::call`
