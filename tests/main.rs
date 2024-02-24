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
/// ````
mod compile_fail {}
