use async_trait::async_trait;
use russh::{Channel, ChannelId};
use russh::server::{Auth, Handle, Session as SshSession};
use crate::session::{Session};

pub struct JumpServer {
    session: Session,
}

#[async_trait]
impl russh::server::Handler for JumpServer {
    type Error = anyhow::Error;

    async fn auth_none(
        &mut self,
        _: &str,
    ) -> Result<Auth, Self::Error> {
        Ok(Auth::Accept)
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<russh::server::Msg>,
        _: &mut SshSession,
    ) -> Result<(), Self::Error> {
        let sess = self.session.clone();

        // 启动目标 SSH
        tokio::spawn(async move {
            if let Err(e) = forward_shell(channel, sess.clone()).await {
                log::error!("forward error: {:?}", e);
            }
        });

        Ok(())
    }
}
