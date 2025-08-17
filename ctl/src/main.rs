mod idle;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: IdleCommand,
}

#[derive(Subcommand)]
pub enum IdleCommand {
    /// Focus the notification viewer and block idle behavior
    #[command(
        about = "Temporarily disable idle behavior",
        long_about = "Prevents the idle daemon from activating features like screensavers or suspending the session by focusing the notification viewer. This is useful for user-attention-grabbing notifications or dialogs."
    )]
    Inhibit {
        #[command(subcommand)]
        action: SwitchAction,
    },

    /// Show all active inhibitors
    #[command(
        about = "List active idle inhibitors",
        long_about = "Displays all currently registered inhibitors that are preventing the session from going idle. Useful for debugging or monitoring idle state suppression."
    )]
    Inhibitors,
}

#[derive(Subcommand)]
enum SwitchAction {
    On,
    Off,
    Toggle,
    State,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let event = match cli.command {
        IdleCommand::Inhibit { action } => match action {
            SwitchAction::On => idle::Event::Inhibit,
            SwitchAction::Off => idle::Event::Uninhibit,
            SwitchAction::Toggle => idle::Event::ToggleInhibit,
            SwitchAction::State => idle::Event::InhibitState,
        },
        IdleCommand::Inhibitors => idle::Event::Inhibitors,
    };

    idle::emit(event).await.map_err(Into::into)
}
