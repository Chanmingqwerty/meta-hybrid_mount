#[cfg(any(target_os = "linux", target_os = "android"))]
pub use crate::utils::send_unmountable;

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn send_unmountable<P>(_target: P) 
where P: AsRef<std::path::Path> {
}
