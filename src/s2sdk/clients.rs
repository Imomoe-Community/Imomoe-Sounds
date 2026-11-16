// Generated from s2sdk.pplugin (group: clients)

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

/// Returns the client's SteamID64 - a unique 64-bit identifier of a Steam account.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot.
///
/// # Returns - (uint64):
/// uint64_t The client's SteamID64.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetClientSteamID64(playerSlot: i32) -> u64 {
    unsafe {
        __s2sdk_GetClientSteamID64.expect("GetClientSteamID64 function was not found")(playerSlot)
    }
}
pub type _GetClientSteamID64 = unsafe extern "C" fn(i32) -> u64;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetClientSteamID64: Option<_GetClientSteamID64> = None;

/// Retrieves a client's language as an ISO code. Unlike GetClientLanguage, which returns the raw cl_language value ("english", "schinese"), this returns the code translation files are keyed by ("en", "zh-CN"). Empty until the client answers the cl_language query, and always empty for bots. Reliably available starting from OnClientAuthenticated and OnClientPutInServer.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot.
///
/// # Returns - (string):
/// The client's ISO language code.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetClientLanguageCode(playerSlot: i32) -> Str {
    unsafe {
        __s2sdk_GetClientLanguageCode.expect("GetClientLanguageCode function was not found")(
            playerSlot,
        )
    }
}
pub type _GetClientLanguageCode = unsafe extern "C" fn(i32) -> Str;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetClientLanguageCode: Option<_GetClientLanguageCode> = None;

/// Checks if a certain player has entered the game.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot.
///
/// # Returns - (bool):
/// true if the player is in the game, false otherwise.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn IsClientInGame(playerSlot: i32) -> bool {
    unsafe { __s2sdk_IsClientInGame.expect("IsClientInGame function was not found")(playerSlot) }
}
pub type _IsClientInGame = unsafe extern "C" fn(i32) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_IsClientInGame: Option<_IsClientInGame> = None;

/// Checks if a certain player is a fake client.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot.
///
/// # Returns - (bool):
/// true if the client is a fake client, false otherwise.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn IsFakeClient(playerSlot: i32) -> bool {
    unsafe { __s2sdk_IsFakeClient.expect("IsFakeClient function was not found")(playerSlot) }
}
pub type _IsFakeClient = unsafe extern "C" fn(i32) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_IsFakeClient: Option<_IsFakeClient> = None;

/// Retrieves the team number of an client.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot whose team number is to be retrieved.
///
/// # Returns - (int32):
/// The team number of the client, or 0 if the client is invalid.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetClientTeam(playerSlot: i32) -> CSTeam {
    unsafe { __s2sdk_GetClientTeam.expect("GetClientTeam function was not found")(playerSlot) }
}
pub type _GetClientTeam = unsafe extern "C" fn(i32) -> CSTeam;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetClientTeam: Option<_GetClientTeam> = None;

/// Retrieves the absolute origin of an client.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot whose absolute origin is to be retrieved.
///
/// # Returns - (vec3):
/// A vector where the absolute origin will be stored.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetClientAbsOrigin(playerSlot: i32) -> Vec3 {
    unsafe {
        __s2sdk_GetClientAbsOrigin.expect("GetClientAbsOrigin function was not found")(playerSlot)
    }
}
pub type _GetClientAbsOrigin = unsafe extern "C" fn(i32) -> Vec3;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetClientAbsOrigin: Option<_GetClientAbsOrigin> = None;

/// Retrieves the pawn entity pointer associated with a client.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot.
///
/// # Returns - (ptr64):
/// A pointer to the client's pawn entity, or nullptr if the client or controller is invalid.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn GetClientPawn(playerSlot: i32) -> usize {
    unsafe { __s2sdk_GetClientPawn.expect("GetClientPawn function was not found")(playerSlot) }
}
pub type _GetClientPawn = unsafe extern "C" fn(i32) -> usize;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_GetClientPawn: Option<_GetClientPawn> = None;

/// Checks whether the current client pawn is alive.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn IsClientAlive(playerSlot: i32) -> bool {
    unsafe { __s2sdk_IsClientAlive.expect("IsClientAlive function was not found")(playerSlot) }
}
pub type _IsClientAlive = unsafe extern "C" fn(i32) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_IsClientAlive: Option<_IsClientAlive> = None;

/// Checks if a certain player is the SourceTV bot.
///
/// # Arguments
/// * `playerSlot` - (int32): The index of the player's slot.
///
/// # Returns - (bool):
/// true if the client is the SourceTV bot, false otherwise.
#[track_caller]
#[allow(dead_code, non_snake_case)]
pub fn IsClientSourceTV(playerSlot: i32) -> bool {
    unsafe {
        __s2sdk_IsClientSourceTV.expect("IsClientSourceTV function was not found")(playerSlot)
    }
}
pub type _IsClientSourceTV = unsafe extern "C" fn(i32) -> bool;
#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __s2sdk_IsClientSourceTV: Option<_IsClientSourceTV> = None;
