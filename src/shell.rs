use russh::client::{self, Config};
use russh_keys::key;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::session::{Session};

async fn connect_target(target: &str) -> anyhow::Result<client::Handle> {
    let key = russh_keys::load_secret_key(
        format!("{}/.ssh/id_rsa", std::env::var("HOME")?),
        None,
    )?;

    let cfg = Arc::new(Config::default());
    let addr: SocketAddr = format!("{}:1122", target).parse()?;

    let mut session = client::connect(cfg, addr, client::Handler {}).await?;
    session.authenticate_publickey("root", Arc::new(key)).await?;

    Ok(session)
}

async fn forward_shell(
    mut client_channel: Channel<russh::server::Msg>,
    sess: Session,
) -> anyhow::Result<()> {
    let mut target = connect_target(&sess.target).await?;
    let mut target_channel = target.channel_open_session().await?;

    // 请求 shell
    target_channel.request_shell(true).await?;

    // IO 转发
    let mut client_rx = client_channel.stream(0);
    let mut target_rx = target_channel.stream(0);

    let cancel = sess.cancel.clone();

    let c2t = tokio::spawn(async move {
        while let Some(data) = client_rx.recv().await {
            sess.touch();
            target_channel.data(&data).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    let t2c = tokio::spawn(async move {
        while let Some(data) = target_rx.recv().await {
            sess.touch();
            client_channel.data(&data).await?;
        }
        Ok::<_, anyhow::Error>(())
    });

    tokio::select! {
        _ = cancel.notified() => {
            client_channel.close().await?;
            target_channel.close().await?;
        }
        _ = c2t => {}
        _ = t2c => {}
    }

    Ok(())
}
