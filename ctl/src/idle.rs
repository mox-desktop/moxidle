use std::io::{self, Write};

pub enum Event {
    Inhibit,
    Uninhibit,
    ToggleInhibit,
    InhibitState,
    Inhibitors,
}

#[zbus::proxy(
    interface = "pl.mox.Idle",
    default_service = "pl.mox.Idle",
    default_path = "/pl/mox/Idle"
)]
trait Idle {
    async fn inhibited(&self) -> zbus::Result<bool>;

    async fn inhibit(&self) -> zbus::Result<()>;

    async fn uninhibit(&self) -> zbus::Result<()>;

    async fn inhibitors(&self) -> zbus::Result<Vec<String>>;
}

pub async fn emit(event: Event) -> zbus::Result<()> {
    let conn = zbus::Connection::session().await?;

    let idle = IdleProxy::new(&conn).await?;
    let mut out = io::stdout().lock();

    match event {
        Event::ToggleInhibit => {
            if idle.inhibited().await? {
                idle.uninhibit().await?;
            } else {
                idle.inhibit().await?;
            }
        }
        Event::InhibitState => {
            if idle.inhibited().await? {
                writeln!(out, "inhibited")?;
            } else {
                writeln!(out, "uninhibited")?;
            }
        }
        Event::Inhibitors => {
            let inhibitors = idle.inhibitors().await?;
            for inhibitor in inhibitors {
                writeln!(out, "{inhibitor}")?;
            }
        }
        Event::Inhibit => idle.inhibit().await?,
        Event::Uninhibit => idle.uninhibit().await?,
    }

    Ok(())
}
