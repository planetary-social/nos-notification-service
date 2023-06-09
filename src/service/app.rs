pub mod commands;
pub mod common;

pub struct Application<'a> {
    pub commands: &'a Commands<'a>,
    pub queries: &'a Queries,
}

impl<'a> Application<'_> {
    pub fn new(commands: &'a Commands, queries: &'a Queries) -> Application<'a> {
        Application { commands, queries }
    }
}

pub struct Commands<'a> {
    pub register: &'a (dyn commands::RegisterHandler + Sync),
}

impl Commands<'_> {
    pub fn new(register: &(dyn commands::RegisterHandler + Sync)) -> Commands {
        Commands { register }
    }
}

pub struct Queries {}

impl Queries {
    pub fn new() -> Queries {
        Queries {}
    }
}
