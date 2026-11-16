// Generated from s2sdk.pplugin (group: commands)

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

/// Creates a console command or hooks an already existing one.
///
/// # Arguments
/// * `name` - (string): The name of the console command.
/// * `description` - (string): A brief description of what the command does.
/// * `flags` - (int64): Command flags that define the behavior of the command.
/// * `callback` - (function): A callback function that is invoked when the command is executed.
/// * `type_` - (uint8): Whether the hook was in post mode (after processing) or pre mode (before processing).
///
/// # Returns - (bool):
/// true if the command was successfully created; otherwise, false.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn AddConsoleCommand(
    name: &Str,
    description: &Str,
    flags: ConVarFlag,
    callback: ConCommandCallback,
    type_: HookMode,
) -> bool {
    unsafe {
        __s2sdk_AddConsoleCommand.expect("AddConsoleCommand function was not found")(
            name,
            description,
            flags,
            callback,
            type_,
        )
    }
}
pub type _AddConsoleCommand =
    unsafe extern "C" fn(&Str, &Str, ConVarFlag, ConCommandCallback, HookMode) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_AddConsoleCommand: Option<_AddConsoleCommand> = None;

/// Local compatibility bridge: checks live callbacks, not retained command names.
///
/// # Arguments
/// * `name` - (string): Command name.
///
/// # Returns - (bool):
/// Whether the engine command has callbacks registered.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn HasConsoleCommandCallbacks(name: &Str) -> bool {
    unsafe {
        __s2sdk_HasConsoleCommandCallbacks
            .expect("HasConsoleCommandCallbacks function was not found")(name)
    }
}
pub type _HasConsoleCommandCallbacks = unsafe extern "C" fn(&Str) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_HasConsoleCommandCallbacks: Option<_HasConsoleCommandCallbacks> = None;

/// Removes a console command from the system.
///
/// # Arguments
/// * `name` - (string): The name of the command to be removed.
/// * `callback` - (function): The callback function associated with the command to be removed.
///
/// # Returns - (bool):
/// true if the command was successfully removed; otherwise, false.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn RemoveCommand(name: &Str, callback: ConCommandCallback) -> bool {
    unsafe { __s2sdk_RemoveCommand.expect("RemoveCommand function was not found")(name, callback) }
}
pub type _RemoveCommand = unsafe extern "C" fn(&Str, ConCommandCallback) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_RemoveCommand: Option<_RemoveCommand> = None;
