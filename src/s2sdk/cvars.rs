// Generated from s2sdk.pplugin (group: cvars)

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

/// Searches for a console variable.
///
/// # Arguments
/// * `name` - (string): The name of the console variable to search for.
///
/// # Returns - (uint64):
/// A handle to the console variable data if found; otherwise, nullptr.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn FindConVar(name: &Str) -> u64 {
    unsafe { __s2sdk_FindConVar.expect("FindConVar function was not found")(name) }
}
pub type _FindConVar = unsafe extern "C" fn(&Str) -> u64;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_FindConVar: Option<_FindConVar> = None;

/// Retrieves the current value of a boolean console variable.
///
/// # Arguments
/// * `conVarHandle` - (uint64): The handle to the console variable data.
///
/// # Returns - (bool):
/// The current boolean value of the console variable.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetConVarBool(conVarHandle: u64) -> bool {
    unsafe { __s2sdk_GetConVarBool.expect("GetConVarBool function was not found")(conVarHandle) }
}
pub type _GetConVarBool = unsafe extern "C" fn(u64) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetConVarBool: Option<_GetConVarBool> = None;
