// Generated from s2sdk.pplugin (group: schema)

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
pub fn GetSchemaOffset(className: &Str, memberName: &Str) -> i32 {
    unsafe {
        __s2sdk_GetSchemaOffset.expect("GetSchemaOffset function was not found")(
            className, memberName,
        )
    }
}
pub type _GetSchemaOffset = unsafe extern "C" fn(&Str, &Str) -> i32;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetSchemaOffset: Option<_GetSchemaOffset> = None;

#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetEntSchema2(entity: usize, className: &Str, memberName: &Str, element: i32) -> i64 {
    unsafe {
        __s2sdk_GetEntSchema2.expect("GetEntSchema2 function was not found")(
            entity, className, memberName, element,
        )
    }
}
pub type _GetEntSchema2 = unsafe extern "C" fn(usize, &Str, &Str, i32) -> i64;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetEntSchema2: Option<_GetEntSchema2> = None;
