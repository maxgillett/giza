/// Common trait for all cli commands
pub trait Cmd: clap::Parser + Sized {
    type Output;

    fn run(self) -> Self::Output;
}