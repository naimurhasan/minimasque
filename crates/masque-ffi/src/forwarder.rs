use std::sync::{Arc, Mutex, PoisonError};

use masque_client::forward;
use masque_client::transport::{ConnectOptions, Connected, Transport};
use masque_core::{Target, connect_udp};
use masque_h3::describe;
use masque_h3::event::Event;
use masque_h3::stats::Stats;
use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;

use crate::config::ForwarderConfig;
use crate::error::{MasqueError, failed};
use crate::records::StatsRecord;

#[derive(uniffi::Object)]
pub struct Forwarder {
    _runtime: Runtime,
    stats: Arc<Stats>,
    events: Mutex<Receiver<Event>>,
    task: JoinHandle<Result<(), masque_h3::Error>>,
    transport: String,
    local: String,
}

struct Started {
    stats: Arc<Stats>,
    events: Receiver<Event>,
    task: JoinHandle<Result<(), masque_h3::Error>>,
    transport: String,
    local: String,
}

async fn start(config: ForwarderConfig) -> Result<Started, masque_h3::Error> {
    let options = ConnectOptions {
        proxy: config.proxy.parse()?,
        name: config.server_name.clone(),
        transport: config.transport.parse::<Transport>()?,
    };
    let target: Target = config.target.parse()?;
    let template = config.template.clone().unwrap_or_else(|| {
        connect_udp::well_known_url(&format!("{}:{}", options.name, options.proxy.port()))
    });
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(config.ca_der))?;
    let connected = Connected::connect(&options, roots).await?;
    let transport = connected.label().to_owned();
    let stats = connected.stats();
    let events = stats.subscribe();
    let tunnel = connected.open(&template, &target).await?;
    let socket = UdpSocket::bind(config.local.as_str()).await?;
    let local = socket.local_addr()?.to_string();
    let task = tokio::spawn(forward::run(socket, tunnel));
    Ok(Started {
        stats,
        events,
        task,
        transport,
        local,
    })
}

#[uniffi::export]
impl Forwarder {
    #[uniffi::constructor]
    pub fn start(config: ForwarderConfig) -> Result<Arc<Self>, MasqueError> {
        let runtime = Runtime::new().map_err(failed)?;
        let started = runtime.block_on(start(config)).map_err(failed)?;
        Ok(Arc::new(Self {
            _runtime: runtime,
            stats: started.stats,
            events: Mutex::new(started.events),
            task: started.task,
            transport: started.transport,
            local: started.local,
        }))
    }

    pub fn transport(&self) -> String {
        self.transport.clone()
    }

    pub fn local_address(&self) -> String {
        self.local.clone()
    }

    pub fn stats(&self) -> StatsRecord {
        self.stats.snapshot().into()
    }

    pub fn drain_events(&self) -> Vec<String> {
        let mut events = self.events.lock().unwrap_or_else(PoisonError::into_inner);
        std::iter::from_fn(|| events.try_recv().ok())
            .flat_map(|event| describe::lines(&event))
            .collect()
    }

    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }

    pub fn stop(&self) {
        self.task.abort();
    }
}

impl Drop for Forwarder {
    fn drop(&mut self) {
        self.task.abort();
    }
}
