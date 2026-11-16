// Generated from s2sdk.pplugin (group: entities)

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
pub fn EntHandleToEntPointer(entityHandle: i32) -> usize {
    unsafe {
        __s2sdk_EntHandleToEntPointer.expect("EntHandleToEntPointer function was not found")(
            entityHandle,
        )
    }
}
pub type _EntHandleToEntPointer = unsafe extern "C" fn(i32) -> usize;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_EntHandleToEntPointer: Option<_EntHandleToEntPointer> = None;

/// Converts an entity index into an entity handle.
///
/// # Arguments
/// * `entityIndex` - (int32): The index of the entity to convert.
///
/// # Returns - (int32):
/// The entity handle as an integer, or -1 if the entity index is invalid.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn EntIndexToEntHandle(entityIndex: i32) -> i32 {
    unsafe {
        __s2sdk_EntIndexToEntHandle.expect("EntIndexToEntHandle function was not found")(
            entityIndex,
        )
    }
}
pub type _EntIndexToEntHandle = unsafe extern "C" fn(i32) -> i32;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_EntIndexToEntHandle: Option<_EntIndexToEntHandle> = None;

/// Checks if the provided entity handle is valid.
///
/// # Arguments
/// * `entityHandle` - (int32): The entity handle to check.
///
/// # Returns - (bool):
/// True if the entity handle is valid, false otherwise.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn IsValidEntHandle(entityHandle: i32) -> bool {
    unsafe {
        __s2sdk_IsValidEntHandle.expect("IsValidEntHandle function was not found")(entityHandle)
    }
}
pub type _IsValidEntHandle = unsafe extern "C" fn(i32) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_IsValidEntHandle: Option<_IsValidEntHandle> = None;

/// Retrieves the absolute origin of an entity.
///
/// # Arguments
/// * `entityHandle` - (int32): The handle of the entity whose absolute origin is to be retrieved.
///
/// # Returns - (vec3):
/// A vector where the absolute origin will be stored.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetEntityAbsOrigin(entityHandle: i32) -> Vec3 {
    unsafe {
        __s2sdk_GetEntityAbsOrigin.expect("GetEntityAbsOrigin function was not found")(entityHandle)
    }
}
pub type _GetEntityAbsOrigin = unsafe extern "C" fn(i32) -> Vec3;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetEntityAbsOrigin: Option<_GetEntityAbsOrigin> = None;
