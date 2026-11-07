use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use masque_h3::Error;

use super::{App, Feed, run};

pub async fn supervise<F>(work: F, app: Option<App>, feed: Feed) -> Result<(), Error>
where
    F: Future<Output = Result<(), Error>> + Send + 'static,
{
    let Some(app) = app else {
        return work.await;
    };
    let stop = Arc::new(AtomicBool::new(false));
    let ui_stop = stop.clone();
    let mut ui = tokio::task::spawn_blocking(move || run(app, feed, &ui_stop));
    let mut work = tokio::spawn(work);
    tokio::select! {
        ui_result = &mut ui => {
            work.abort();
            Ok(ui_result??)
        }
        work_result = &mut work => {
            stop.store(true, Ordering::Relaxed);
            ui.await??;
            work_result?
        }
    }
}
