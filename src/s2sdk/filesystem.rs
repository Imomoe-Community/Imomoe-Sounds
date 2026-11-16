// Generated from s2sdk.pplugin (group: filesystem)

#[allow(unused_imports)]
use super::aliases::*;
#[allow(unused_imports)]
use super::delegates::*;
#[allow(unused_imports)]
use super::enums::*;
#[allow(unused_imports)]
use plugify::{Arr, Mat4x4, Str, Var, Vec2, Vec3, Vec4, scope};
#[allow(unused_imports)]
use std::sync::RwLock;

/// Reads a file and returns its contents as a string.
///
/// # Arguments
/// * `localFileName` - (string): The relative path of the file to read.
/// * `pathId` - (string): The filesystem search path ID (e.g., "GAME"). If empty, uses "GAME".
///
/// # Returns - (string):
/// The file contents, or an empty string on failure.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn ReadFileVPK(localFileName: &Str, pathId: &Str) -> Str {
    unsafe {
        __s2sdk_ReadFileVPK.expect("ReadFileVPK function was not found")(localFileName, pathId)
    }
}
pub type _ReadFileVPK = unsafe extern "C" fn(&Str, &Str) -> Str;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_ReadFileVPK: Option<_ReadFileVPK> = None;
