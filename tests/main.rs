use ffi_closure::Closure;
use std::ffi::c_void;

#[test]
fn export() {
    let sq = Closure::<dyn Send + FnMut(u32) -> u32>::new(|x| x * x);
    let (f, user_data) = sq.as_extern_parts();

    unsafe {
        let res = (f)(3, user_data);
        assert_eq!(res, 9)
    }
}

#[test]
fn import() {
    unsafe extern "C" fn square(x: u32, _: *mut c_void) -> u32 {
        return x * x;
    }

    unsafe {
        let mut sq =
            Closure::<dyn FnMut(u32) -> u32>::from_extern(square, std::ptr::null_mut(), None);
        assert_eq!(sq.call((3,)), 9);
        #[cfg(feature = "nightly")]
        assert_eq!(sq(3), 9)
    }
}
