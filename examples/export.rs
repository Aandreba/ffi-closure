use ffi_closure::Closure;
use std::os::raw::c_void;

pub fn main() {}

#[no_mangle]
pub unsafe extern "C" fn some_lib_fn(
    f: unsafe extern "C" fn(i8, *mut c_void),
    user_data: *mut c_void,
) {
    let mut closure = Closure::<dyn FnMut(i8)>::from_extern(f, user_data, None);
    for i in 0..10 {
        closure(i);
    }
}
