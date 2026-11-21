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

/// Creates a new boolean console variable.
///
/// # Arguments
/// * `name` - (string): The name of the console variable.
/// * `defaultValue` - (bool): The default value for the console variable.
/// * `description` - (string): A brief description of the console variable.
/// * `flags` - (int64): Flags that define the behavior of the console variable.
/// * `hasMin` - (bool): Indicates if a minimum value is provided.
/// * `min` - (bool): The minimum value if hasMin is true.
/// * `hasMax` - (bool): Indicates if a maximum value is provided.
/// * `max` - (bool): The maximum value if hasMax is true.
///
/// # Returns - (uint64):
/// A handle to the created console variable data.
#[track_caller]
#[allow(dead_code, non_snake_case, clippy::too_many_arguments)]
pub fn CreateConVarBool(
    name: &Str,
    defaultValue: bool,
    description: &Str,
    flags: ConVarFlag,
    hasMin: bool,
    min: bool,
    hasMax: bool,
    max: bool,
) -> u64 {
    unsafe {
        __s2sdk_CreateConVarBool.expect("CreateConVarBool function was not found")(
            name,
            defaultValue,
            description,
            flags,
            hasMin,
            min,
            hasMax,
            max,
        )
    }
}
pub type _CreateConVarBool =
    unsafe extern "C" fn(&Str, bool, &Str, ConVarFlag, bool, bool, bool, bool) -> u64;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_CreateConVarBool: Option<_CreateConVarBool> = None;
