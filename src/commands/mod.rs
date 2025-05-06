mod cmds;
pub use cmds::{CommandResult, cmd_manager};

mod echo;
pub use echo::echo;

mod help;
pub use help::help;

mod whoami;
pub use whoami::whoami;
