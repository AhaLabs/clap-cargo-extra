use std::process::Command;

use crate::Args;

pub trait ToCmd {
    fn add_args<'a>(&'a self, cmd: &'a mut Command) -> &'a mut Command;
}

impl<T> ToCmd for T
where
    T: Args,
{
    fn add_args<'a>(&'a self, cmd: &'a mut Command) -> &'a mut Command {
        cmd.args(self.to_args())
    }
}
