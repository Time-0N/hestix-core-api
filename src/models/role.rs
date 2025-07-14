#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    User,
    NasReader,
    NasWriter,
    StreamConsumer,
    GameAdmin,
    SysAdmin,
    Agent,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::NasReader => "nas_reader",
            Role::NasWriter => "nas_writer",
            Role::StreamConsumer => "stream_consumer",
            Role::GameAdmin => "game_admin",
            Role::SysAdmin => "sys_admin",
            Role::Agent => "agent",
        }
    }
}
