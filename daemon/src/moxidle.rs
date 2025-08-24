use crate::{Event, InhibitState};
use futures_lite::stream::StreamExt;
use tokio::sync::oneshot;
use zbus::fdo::DBusProxy;
use zbus::fdo::RequestNameFlags;

struct MoxidleInterface {
    event_sender: calloop::channel::Sender<Event>,
    //emit_receiver: broadcast::Receiver<EmitEvent>,
}

#[zbus::interface(name = "pl.mox.Idle")]
impl MoxidleInterface {
    async fn inhibited(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        if let Err(e) = self.event_sender.send(Event::GetCtlInhibitState(tx)) {
            log::warn!("{e}");
        }

        rx.await.unwrap_or(InhibitState::Uninhibited) == InhibitState::Inhibited
    }

    async fn inhibitors(&self) -> Vec<String> {
        Vec::new()
    }

    async fn inhibit(&self) {
        if let Err(e) = self.event_sender.send(Event::CtlInhibited(true)) {
            log::warn!("{e}");
        }
    }

    async fn uninhibit(&self) {
        if let Err(e) = self.event_sender.send(Event::CtlInhibited(false)) {
            log::warn!("{e}");
        }
    }
}

pub async fn serve(event_sender: calloop::channel::Sender<Event>) -> zbus::Result<()> {
    let server = MoxidleInterface { event_sender };

    let conn = zbus::connection::Builder::session()?
        .serve_at("/pl/mox/Idle", server)?
        .build()
        .await?;

    conn.request_name_with_flags(
        "pl.mox.Idle",
        RequestNameFlags::ReplaceExisting | RequestNameFlags::AllowReplacement,
    )
    .await?;

    let acquired_stream = DBusProxy::new(&conn).await?.receive_name_lost().await?;
    tokio::spawn(async move {
        let mut acquired_stream = acquired_stream;
        if acquired_stream.next().await.is_some() {
            log::info!("Request to ReplaceExisting on pl.mox.Idle received");
            std::process::exit(0);
        }
    });

    Ok(())
}
