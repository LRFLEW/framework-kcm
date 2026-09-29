// SPDX-License-Identifier: GPL-3.0-or-later
//! Asks polkit whether the D-Bus caller may perform an action, letting polkit
//! prompt for a password.

use std::collections::HashMap;

use zbus::message::Header;
use zbus::Connection;
use zbus_polkit::policykit1::{AuthorityProxy, CheckAuthorizationFlags, Subject};

pub const ACTION_BATTERY: &str = "io.github.frameworkkcm.battery";
pub const ACTION_FAN: &str = "io.github.frameworkkcm.fan";
pub const ACTION_INPUT: &str = "io.github.frameworkkcm.input";

/// Returns Ok(true) if the sender of the message with header `hdr` is authorized for `action_id`.
pub async fn check(conn: &Connection, hdr: &Header<'_>, action_id: &str) -> zbus::Result<bool> {
    let subject = Subject::new_for_message_header(hdr).map_err(|e| zbus::Error::Failure(e.to_string()))?;
    let result = AuthorityProxy::new(conn)
        .await?
        .check_authorization(
            &subject,
            action_id,
            &HashMap::new(),
            CheckAuthorizationFlags::AllowUserInteraction.into(),
            "",
        )
        .await?;
    Ok(result.is_authorized)
}
