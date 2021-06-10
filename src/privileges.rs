use nix::unistd::Uid;

use crate::errors::*;

pub fn require_root() -> Result<()> {
    if !Uid::effective().is_root() {
        bail!(ErrorKind::RootPrivilegesRequired(
            env!("CARGO_PKG_NAME").into()
        ));
    } else {
        Ok(())
    }
}
