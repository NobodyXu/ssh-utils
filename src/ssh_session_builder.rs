use openssh::{Error, Session, SessionBuilder};

#[derive(Debug)]
pub struct SshSessionBuilder<'dest>(SessionBuilder, &'dest str);

impl<'dest> SshSessionBuilder<'dest> {
    pub fn new(builder: SessionBuilder, dest: &'dest str) -> Self {
        Self(builder, dest)
    }

    pub async fn connect(&self) -> Result<Session, Error> {
        self.0.connect_mux(self.1).await
    }

    pub fn dest(&self) -> &'dest str {
        self.1
    }
}
