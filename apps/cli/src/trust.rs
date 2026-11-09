use std::path::Path;

use masque_h3::Error;
use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;

pub fn roots(ca: &Path) -> Result<RootCertStore, Error> {
    let der =
        std::fs::read(ca).map_err(|err| format!("cannot read --ca {}: {err}", ca.display()))?;
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(der))?;
    Ok(roots)
}
