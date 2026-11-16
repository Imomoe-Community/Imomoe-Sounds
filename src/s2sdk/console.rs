// Generated from s2sdk.pplugin (group: console)

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

/// Sends a reply message to a player or to the server console depending on the command context.
///
/// # Arguments
/// * `context` - (int32): The context from which the command was called (e.g., Console or Chat).
/// * `playerSlot` - (int32): The slot/index of the player receiving the message.
/// * `message` - (string): The message string to be sent as a reply.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn ReplyToCommand(context: ConCommandContext, playerSlot: i32, message: &Str) {
    unsafe {
        __s2sdk_ReplyToCommand.expect("ReplyToCommand function was not found")(
            context, playerSlot, message,
        )
    }
}
pub type _ReplyToCommand = unsafe extern "C" fn(ConCommandContext, i32, &Str);
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_ReplyToCommand: Option<_ReplyToCommand> = None;
