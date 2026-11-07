use std::sync::Arc;

use masque_h3::describe;
use masque_h3::stats::Stats;
use tokio::sync::broadcast::error::RecvError;

pub async fn events(stats: Arc<Stats>) {
    let mut events = stats.subscribe();
    loop {
        match events.recv().await {
            Ok(event) => describe::lines(&event)
                .iter()
                .for_each(|line| eprintln!("{line}")),
            Err(RecvError::Lagged(skipped)) => eprintln!("… {skipped} events skipped"),
            Err(RecvError::Closed) => return,
        }
    }
}
