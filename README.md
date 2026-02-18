
<div align="center">

# timewall

[![CI](https://github.com/bcyran/timewall/actions/workflows/test.yml/badge.svg)](https://github.com/bcyran/timewall/actions/workflows/test.yml)
[![Test Coverage](https://codecov.io/gh/bcyran/timewall/branch/master/graph/badge.svg?token=Z025ICENDQ)](https://codecov.io/gh/bcyran/timewall)
[![License](https://img.shields.io/github/license/bcyran/timewall)](https://github.com/bcyran/timewall/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/timewall)](https://crates.io/crates/timewall)

**Apple dynamic HEIF wallpapers on GNU/Linux**

![timewall_preview](https://user-images.githubusercontent.com/8322846/197593208-0e0b7901-2caf-43b0-b7fd-847c2cb49ff1.gif)

</div>

---

## Features

- Supports original HEIF/HEIC dynamic wallpaper files from macOS.
- Handles all schedule types: sun position, time-based, and dark/light mode.
- Automatic location detection via GeoClue 2.
- Automatic system theme detection via D-Bus.
- Set wallpapers once or continuously (daemon mode).
- Preview wallpaper transitions.
- Display wallpaper metadata.
- Extract all images and metadata as XML.

---

## Installation

### Package Repositories

[![Packaging status](https://repology.org/badge/vertical-allrepos/timewall.svg)](https://repology.org/project/timewall/versions)

### Prerequisites

`timewall` requires [`libheif`](https://github.com/strukturag/libheif) >= 1.19.7 for HEIF support. Ensure it is installed.
If building from source, you may also need `libheif-dev`, depending on your distribution.

### Binary

Download the latest prebuilt binary and shell completions from the [releases page](https://github.com/bcyran/timewall/releases).
Place the `timewall` binary in a directory included in your `$PATH`, e.g., `/usr/local/bin`.

### Cargo

Install via Cargo:

```
cargo install timewall
```

### Nix

This repository provides a Nix flake, a `nixpkgs` overlay, and a Home Manager module.

Add `timewall` as an input to your `flake.nix`:

```nix
inputs.timewall.url = "github:bcyran/timewall";
```

#### Package

```nix
environment.systemPackages = [
  inputs.timewall.packages.${pkgs.system}.timewall
];
```

#### Overlay

```nix
nixpkgs.overlays = [
  inputs.timewall.overlays.default
]
environment.systemPackages = [
  pkgs.timewall
];
```

#### Home Manager

```nix
services.timewall = {
  enable = true;
  wallpaperPath = ./wallpaper.heif; # optional, can be set at runtime
  config = {} # optional, see the configuration section
}
```

---

## Usage

> [!IMPORTANT]
> For sun position-based wallpapers, `timewall` needs your approximate location.
> Ensure GeoClue 2 is available or configure your location manually.
> See [Configuration](#configuration).

### Setting the Wallpaper

#### One-Time Mode

Set the wallpaper by running:

```
timewall set path/to/wallpaper.heif
```

This sets the wallpaper according to the current time or sun position, based on the wallpaper's schedule.
Note: This does not update automatically. To refresh, rerun the command or simply use `timewall set` (the last wallpaper is remembered).

See also: [Where to find dynamic wallpapers](#where-to-find-the-dynamic-wallpapers).

#### Daemon Mode

To update the wallpaper automatically:

```
timewall set --daemon
```

This runs continuously, updating your wallpaper as time passes.
It's recommended to run this at startup as a background process.

By default, daemon mode uses the last set wallpaper.
To change wallpapers, run `timewall set path/to/new/wall.heif`; the daemon will pick up the change.

#### Systemd Service

To start `timewall` automatically, create `~/.config/systemd/user/timewall.service` with:

```systemd
[Unit]
Description=Dynamic wallpapers daemon

[Service]
Type=simple
ExecStart=timewall set --daemon

[Install]
WantedBy=default.target
```

Enable and start the service:

```
systemctl --user enable --now timewall.service
```

`timewall` will now start on boot and update your wallpaper throughout the day.

### Previewing

Preview wallpaper transitions with:

```
timewall preview path/to/wallpaper.heif
```

This cycles through all images in the wallpaper, simulating changes over the day.
Control preview speed with `--delay` (milliseconds) and loop with `--repeat`.

### Unpacking

Extract all images and metadata as XML:

```
timewall unpack path/to/wallpaper.heif path/to/output/directory
```

### Reading Metadata

Display all metadata:

```
timewall info path/to/wallpaper.heif
```

---

## Configuration

`timewall` uses a config file at `$XDG_CONFIG_HOME/timewall/config.toml` (typically `~/.config/timewall/config.toml`).
A default config is created when you first run `timewall set`.

### Automatic Location

Sun position-based wallpapers require your approximate location.
By default, GeoClue 2 is used unless manually configured (see below).
Disable GeoClue by setting `geoclue.enable = false`.

If GeoClue cannot retrieve your location (e.g., offline), `timewall` uses the last known location by default.
Disable this fallback with `geoclue.cache_fallback = false`.

`geoclue.prefer` determines whether GeoClue is prioritized over manual location when both are available.
This is useful if you prefer automatic detection but want to fall back to manual configuration if GeoClue is unavailable.

`geoclue.timeout` sets the maximum time (milliseconds) to wait for GeoClue.
If exceeded, `timewall` falls back to manual location or fails, depending on `geoclue.prefer`.

Example:

```toml
[geoclue]
enable = true
cache_fallback = true
prefer = false
timeout = 1000
```

> [!IMPORTANT]
> You may need to grant `timewall` access to location data in GeoClue configuration.
>
> `/etc/geoclue/geoclue.conf`:
>
> ```conf
> [timewall]
> allowed=true
> system=false
> users=
> ```
>
> On NixOS:
>
> ```nix
> services.geoclue2.appConfig.timewall = {
>   isAllowed = true;
>   isSystem = false;
> }
> ```

### Manual Location

Set your location manually in the `location` section.
By default, manual configuration is prioritized over GeoClue.

```toml
[location]
lat = 51.11
lon = 17.02
```

`lat` and `lon` specify latitude and longitude.

### Custom Wallpaper Setting Command

If the default wallpaper setting does not work for your setup, or you wish to customize it, specify a custom command:

Example using `feh`:

```toml
[setter]
command = ['feh', '--bg-fill', '%f']
quiet = true
overlap = 0
```

`%f` is replaced with the absolute path to the image.

Commands are NOT passed through the shell.
To use shell features (e.g., environment variables, chaining), call the shell explicitly:

```toml
[setter]
command = ['bash', '-c', 'command_1 %f && command_2 %f']
```

See also: [Wallpaper setting commands](#wallpaper-setting-commands).

By default, `stdout` and `stderr` are suppressed.
Set `setter.quiet = false` to change this.

`setter.overlap` defines the time (milliseconds) between starting a new command and terminating the old one.
Useful for commands that continue running after setting the wallpaper (e.g., `swaybg`), ensuring a smooth transition.

### Daemon Mode Wallpaper Update Interval

Set the update interval (seconds):

```toml
[daemon]
update_interval_seconds = 600
```

---

## Where to Find Dynamic Wallpapers

- **Original macOS dynamic wallpapers:**
  If you have access to a Mac, you can copy the dynamic wallpapers directly.
  These files can also be found online, but links are omitted for legal reasons.
- **[Dynamic Wallpaper Club](https://www.dynamicwallpaper.club/):**
  Many user-created wallpapers, though quality varies. Few use sun position schedules effectively.
- **[Jetson Creative](https://www.jetsoncreative.com/mojave):**
  Three free wallpapers and additional bundles for purchase.
- **[mczachurski/wallpaper](https://github.com/mczachurski/wallpapper):**
  Two high-quality custom wallpapers.

---

## Wallpaper Setting Commands

Useful commands for setting wallpapers in various desktop environments.
`%f` is a placeholder for the image path.

### GNOME

```shell
# Light mode wallpaper
gsettings set org.gnome.desktop.background picture-uri file://%f

# Dark mode wallpaper
gsettings set org.gnome.desktop.background picture-uri-dark file://%f

# Lockscreen background
gsettings set org.gnome.desktop.screensaver picture-uri file://%f
```

### KDE Plasma

```shell
# Wallpaper
plasma-apply-wallpaperimage %f

# Lockscreen background
kwriteconfig6 --file kscreenlockerrc --group Greeter --group Wallpaper --group org.kde.image --group General --key Image %f
```

Use `kwriteconfig5` for Plasma 5.

---

## Resources & Credits

The following resources were invaluable during `timewall` development:

- <https://itnext.io/macos-mojave-dynamic-wallpaper-fd26b0698223>
- <https://itnext.io/macos-mojave-dynamic-wallpapers-ii-f8b1e55c82f>
- <https://itnext.io/macos-mojave-wallpaper-iii-c747c30935c4>
- <https://github.com/mczachurski/wallpapper>
- <https://git.spacesnek.rocks/johannes/heic-to-gnome-xml-wallpaper>
