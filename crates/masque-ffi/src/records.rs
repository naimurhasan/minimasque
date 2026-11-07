use masque_h3::stats::Snapshot;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct StatsRecord {
    pub open_tunnels: u32,
    pub opened: u64,
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub packets_up: u64,
    pub packets_down: u64,
    pub dropped: u64,
}

impl From<Snapshot> for StatsRecord {
    fn from(snapshot: Snapshot) -> Self {
        let totals = snapshot.totals;
        Self {
            open_tunnels: u32::try_from(snapshot.tunnels.len()).unwrap_or(u32::MAX),
            opened: snapshot.opened,
            bytes_up: totals.bytes_up,
            bytes_down: totals.bytes_down,
            packets_up: totals.packets_up,
            packets_down: totals.packets_down,
            dropped: totals.dropped,
        }
    }
}
