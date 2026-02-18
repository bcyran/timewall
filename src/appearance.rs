use std::thread;

use anyhow::{Context, Result};
use futures_lite::StreamExt;
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

    #[zbus(signal)]
    fn setting_changed(&self, namespace: &str, key: &str, value: OwnedValue) -> zbus::Result<()>;
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

/// Spawn a background thread that listens for system theme changes via D-Bus.
///
/// Calls the provided callback whenever the system color scheme changes. If D-Bus is not
/// accessible, a warning is logged and the thread exits silently.
pub fn start_appearance_listener(on_change: impl Fn() + Send + Sync + 'static) {
    thread::spawn(move || {
        let result = async_io::block_on(listen_for_theme_changes(&on_change));
        if let Err(e) = result {
            log::warn!("appearance change listener stopped: {e}");
        }
    });
}

async fn listen_for_theme_changes(on_change: impl Fn() + Send + Sync) -> Result<()> {
    let connection = zbus::Connection::session()
        .await
        .context("failed to connect to D-Bus session bus")?;
    let proxy = XdgPortalSettingsProxy::new(&connection, PORTAL_DESTINATION)
        .await
        .context("failed to create XDG portal settings proxy")?;
    let mut stream = proxy
        .receive_setting_changed_with_args(&[(0, APPEARANCE_NAMESPACE), (1, COLOR_SCHEME_KEY)])
        .await
        .context("failed to subscribe to setting changes")?;

    debug!("listening for system theme changes via D-Bus");

    while let Some(_signal) = stream.next().await {
        debug!("system theme change detected via D-Bus");
        on_change();
    }

    Ok(())
}
