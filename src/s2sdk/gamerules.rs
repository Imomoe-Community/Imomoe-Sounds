// Generated from s2sdk.pplugin (group: gamerules)

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

#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetGameRules() -> usize {
    unsafe { __s2sdk_GetGameRules.expect("GetGameRules function was not found")() }
}
pub type _GetGameRules = unsafe extern "C" fn() -> usize;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetGameRules: Option<_GetGameRules> = None;
