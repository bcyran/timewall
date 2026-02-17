use anyhow::{Context, Result};
use log::debug;
use zbus::proxy;
use zbus::zvariant::OwnedValue;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Appearance {
    Light,
    Dark,
}

const PORTAL_DESTINATION: &str = "org.freedesktop.portal.Desktop";
const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";
const COLOR_SCHEME_KEY: &str = "color-scheme";

#[proxy(
    interface = "org.freedesktop.portal.Settings",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait XdgPortalSettings {
    fn read_one(&self, namespace: &str, key: &str) -> zbus::Result<OwnedValue>;
}

/// Get the current system theme (light/dark) via the XDG Desktop Portal.
///
/// Returns `None` if the system has no preference.
pub fn get_system_appearance() -> Result<Option<Appearance>> {
    let connection =
        zbus::blocking::Connection::session().context("failed to connect to D-Bus session bus")?;
    let portal = XdgPortalSettingsProxyBlocking::new(&connection, PORTAL_DESTINATION)
        .context("failed to create XDG portal settings proxy")?;
    let color_scheme_value: u32 = portal
        .read_one(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY)
        .context("failed to read color-scheme from XDG portal")?
        .try_into()
        .context("failed to convert color-scheme value")?;

    let appearance = match color_scheme_value {
        1 => Some(Appearance::Dark),
        2 => Some(Appearance::Light),
        _ => None,
    };
    debug!("detected system appearance: {appearance:?}");
    Ok(appearance)
}
