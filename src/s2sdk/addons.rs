// Generated from s2sdk.pplugin (group: addons)

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

/// Retrieves the workshop addons that are mounted on the server and sent to all clients.
///
/// # Returns - (uint64[]):
/// An array of workshop IDs from s2_extra_addons.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetServerAddons() -> Arr<u64> {
    unsafe { __s2sdk_GetServerAddons.expect("GetServerAddons function was not found")() }
}
pub type _GetServerAddons = unsafe extern "C" fn() -> Arr<u64>;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetServerAddons: Option<_GetServerAddons> = None;
