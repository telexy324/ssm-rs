mod session;
mod monitor;
mod handler;
mod shell;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let key = russh_keys::load_secret_key("server_host_key", None)?;

    let mut cfg = russh::server::Config::default();
    cfg.keys.push(key);
    let cfg = Arc::new(cfg);

    russh::server::run(cfg, "0.0.0.0:44488", |conn| {
        let target = conn.username().unwrap_or_default().to_string();
        let sess = Session::new("".into(), target);

        tokio::spawn(monitor_session(sess.clone()));

        JumpServer { session: sess }
    }).await?;

    Ok(())
}
