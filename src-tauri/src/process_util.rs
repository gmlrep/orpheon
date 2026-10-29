//! Process helpers — hide console windows on Windows when spawning tools.

use std::ffi::OsStr;
use std::process::Command;

/// `CREATE_NO_WINDOW` — prevents a console flash for GUI apps spawning CLI tools.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Build a [`Command`] that does not flash a console window on Windows.
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
