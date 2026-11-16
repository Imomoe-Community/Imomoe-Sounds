// Generated from s2sdk.pplugin

#[allow(unused_imports)]
use plugify::vector_enum_traits;

/// Enum representing the possible teams in Counter-Strike.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSTeam {
    /// No team.
    None = 0,
    /// Spectator team.
    Spectator = 1,
    /// Terrorist team.
    T = 2,
    /// Counter-Terrorist team.
    CT = 3,
}
vector_enum_traits!(CSTeam, i32);

/// Enum representing various flags for ConVars and ConCommands.
#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConVarFlag {
    /// The default, no flags at all.
    None = 0,
    /// Linked to a ConCommand.
    LinkedConcommand = 1,
    /// Hidden in released products. Automatically removed if ALLOW_DEVELOPMENT_CVARS is defined.
    DevelopmentOnly = 2,
    /// Defined by the game DLL.
    GameDll = 4,
    /// Defined by the client DLL.
    ClientDll = 8,
    /// Hidden. Doesn't appear in find or auto-complete. Like DEVELOPMENTONLY but cannot be compiled out.
    Hidden = 16,
    /// Server cvar; data is not sent since it's sensitive (e.g., passwords).
    Protected = 32,
    /// This cvar cannot be changed by clients connected to a multiplayer server.
    SpOnly = 64,
    /// Saved to vars.rc.
    Archive = 128,
    /// Notifies players when changed.
    Notify = 256,
    /// Changes the client's info string.
    UserInfo = 512,
    /// Hides the cvar from lookups.
    Missing0 = 1024,
    /// If this is a server cvar, changes are not logged to the file or console.
    Unlogged = 2048,
    /// Hides the cvar from lookups.
    Missing1 = 4096,
    /// Server-enforced setting on clients.
    Replicated = 8192,
    /// Only usable in singleplayer/debug or multiplayer with sv_cheats.
    Cheat = 16384,
    /// Causes auto-generated varnameN for splitscreen slots.
    PerUser = 32768,
    /// Records this cvar when starting a demo file.
    Demo = 65536,
    /// Excluded from demo files.
    DontRecord = 131072,
    /// Reserved for future use.
    Missing2 = 262144,
    /// Cvars tagged with this are available to customers.
    Release = 524288,
    /// Marks the cvar as a menu bar item.
    MenuBarItem = 1048576,
    /// Reserved for future use.
    Missing3 = 2097152,
    /// Cannot be changed by a client connected to a server.
    NotConnected = 4194304,
    /// Enables fuzzy matching for vconsole.
    VconsoleFuzzyMatching = 8388608,
    /// The server can execute this command on clients.
    ServerCanExecute = 16777216,
    /// Allows clients to execute this command.
    ClientCanExecute = 33554432,
    /// The server cannot query this cvar's value.
    ServerCannotQuery = 67108864,
    /// Sets focus in the vconsole.
    VconsoleSetFocus = 134217728,
    /// IVEngineClient::ClientCmd can execute this command.
    ClientCmdCanExecute = 268435456,
    /// Executes the cvar every tick.
    ExecutePerTick = 536870912,
}
vector_enum_traits!(ConVarFlag, i64);

/// Enum representing the possible results of an operation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultType {
    /// The action continues to be processed without interruption.
    Continue = 0,
    /// Indicates that the action has altered the state or behavior during execution.
    Changed = 1,
    /// The action has been successfully handled, and no further action is required.
    Handled = 2,
    /// The action processing is halted, and no further steps will be executed.
    Stop = 3,
}
vector_enum_traits!(ResultType, i32);

/// The command execution context.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConCommandContext {
    /// The command execute from the client's console.
    Console = 0,
    /// The command execute from the client's chat.
    Chat = 1,
}
vector_enum_traits!(ConCommandContext, i32);

/// Enum representing the type of callback.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookMode {
    /// Callback will be executed before the original function
    Pre = 0,
    /// Callback will be executed after the original function
    Post = 1,
}
vector_enum_traits!(HookMode, u8);

/// Enum representing the type of callback.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventHookError {
    /// Indicates that the event hook was successfully created.
    Okay = 0,
    /// Indicates that the event name provided is invalid or does not exist.
    InvalidEvent = 1,
    /// Indicates that the event system is not currently active or initialized.
    NotActive = 2,
    /// Indicates that the callback function provided is invalid or not compatible with the event system.
    InvalidCallback = 3,
}
vector_enum_traits!(EventHookError, i32);

/// Ownership type for RAII wrappers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    Borrowed,
    Owned,
}
