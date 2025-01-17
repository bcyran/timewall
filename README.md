<div align="center">

# timewall

[![CI](https://github.com/bcyran/timewall/actions/workflows/test.yml/badge.svg)](https://github.com/bcyran/timewall/actions/workflows/test.yml)
[![test coverage](https://codecov.io/gh/bcyran/timewall/branch/master/graph/badge.svg?token=Z025ICENDQ)](https://codecov.io/gh/bcyran/timewall)
[![license](https://img.shields.io/github/license/bcyran/timewall)](https://github.com/bcyran/timewall/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/timewall)](https://crates.io/crates/timewall)

Apple dynamic HEIF wallpapers on GNU/Linux.

![timewall_preview](https://user-images.githubusercontent.com/8322846/197593208-0e0b7901-2caf-43b0-b7fd-847c2cb49ff1.gif)

</div>

---

Features:
- Support for original HEIF/HEIC dynamic wallpaper files used in MacOS.
- Support for all schedule types: sun position based, time based, dark/light mode.
- Set wallpaper once and continuously (daemon mode).
- Preview wallpaper changes.
- Display wallpaper metadata.
- Extract all images and metadata as XML.

---

## Installation
### Repositories
[![Packaging status](https://repology.org/badge/vertical-allrepos/timewall.svg)](https://repology.org/project/timewall/versions)

### Prerequisites
`timewall` depends on [`libheif`](https://github.com/strukturag/libheif) for HEIF support, make sure you have it installed.
If you're building it from source you may also need `libheif-dev`, depending on your distribution.

### Binary
You can download tarball containing the latest prebuilt binary and shell completions from the [releases page](https://github.com/bcyran/timewall/releases).
The binary named `timewall` has to be placed in directory in your `$PATH`, e.g. `/usr/local/bin`.

### Cargo
```
cargo install timewall
```

### Nix
This repository contains a flake exposing a Nix package, `nixpkgs` overlay and a Home Manager module.
Add `timewall` input to your `flake.nix`:
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

## Usage

> [!IMPORTANT]
> If you intend to use sun position based wallpapers, you need to configure your geographical location.
> See [Configuration](#configuration).

### Setting the wallpaper
#### One-time mode
To set the wallpaper just run:
```
timewall set path/to/wallpaper.heif
```
This will set your wallpaper to the correct image, taking into account current time or sun position, depending on the wallpaper schedule.
Note that wallpaper set like this will not update with time.
You can update it by repeating the command above, you can also shorten it to just `timewall set` - last used wallpaper is remembered.

See also: [where to find the dynamic wallpapers](#where-to-find-the-dynamic-wallpapers).

#### Daemon mode
You probably don't want to update the wallpaper manually every time.
To do this automatically you can use the daemon mode:
```
timewall set --daemon
```
This command will run continuously and update your wallpaper as time passes.
It's a good idea to run it automatically at startup as a background process.

As you can see, the command above doesn't include the wallpaper to set.
This is because the daemon mode by default uses the last set wallpaper.
If you already ran `timewall set` manually, then daemon will use whatever wallpaper you set then.
Moreover, if you ever want to change your wallpaper, it's enough to run `timewall set path/to/new/wall.heif`.
The daemon will pick up the change and update the new wallpaper from now on.

#### Systemd service
One way to achieve this is using `systemd` service.
Write the following contents to `~/.config/systemd/user/timewall.service`:
```systemd
[Unit]
Description=Dynamic wallpapers daemon

[Service]
Type=simple
ExecStart=timewall set --daemon

[Install]
WantedBy=default.target
```

And run:
```
systemctl --user enable --now timewall.service
```
After this `timewall` should start automatically on boot and update your wallpaper during the day.

### Previewing
To preview the wallpaper, run:
```
timewall preview path/to/wallpaper.heif
```
This will quickly cycle all images in the wallpaper to simulate changes throughout the day.
Preview speed can be controlled by specifying the delay in milliseconds between consecutive wallpaper changes using the `--delay` option.
You can also infinitely loop the preview using `--repeat` option.

### Unpacking
To unpack all images stored in the wallpaper, as well as its metadata in XML format, run:
```
timewall unpack path/to/wallpaper.heif path/to/output/directory
```

### Reading metadata
All metadata known to `timewall` can be displayed using:
```
timewall info path/to/wallpaper.heif
```

### Configuration
`timewall` uses a config file at `$XDG_CONFIG_HOME/timewall/config.toml` (usually `~/.config/timewall/config.toml`).
A default config will be written when you first run `timewall set`.

#### Geographical location
Using sun position based wallpapers requires your approximate geographical location.
This is configured as follows:
```toml
[location]
lat = 51.11
lon = 17.02
```
Adjust the values `lat` (latitude) and `lon` (longitude) to your needs.

#### Custom wallpaper setting command
If the default wallpaper setting doesn't work in your case for some reason, or you just want to customize it, you can specify custom command to use.
For instance, to set the wallpaper using `feh`, you could add the following to your config file:
```toml
[setter]
command = ['feh', '--bg-fill', '%f']
```
`%f` is a placeholder which will be replaced with full absolute path to the image, which should be set as a wallpaper.

> [!NOTE]
> The command is NOT passed through shell.

#### Daemon mode wallpaper update interval
```toml
[daemon]
update_interval_seconds = 600
```

## Where to find the dynamic wallpapers
- Original MacOS dynamic wallpapers.
  If you have access to a computer running MacOS, you can just copy the dynamic wallpapers.
  You can also find those files online with a bit of effort.
  I'm not going to link any of them because of legal reasons.
- [Dynamic Wallpaper Club](https://www.dynamicwallpaper.club/).
  A lot of user-created wallpapers.
  Unfortunately, many of them are of mediocre quality.
  Only a handful makes use of the sun position schedule (which is the best part of the whole concept to me), and those which do, usually do it poorly.
- [dynwalls.com](http://dynwalls.com/).
  Some free, high quality walls.
- [Jetson Creative](https://www.jetsoncreative.com/mojave).
  Three free wallpapers and some bundles you can buy.
- [mczachurski/wallpaper](https://github.com/mczachurski/wallpapper).
  Two high quality custom made walls.

## Resources / credits
The following resources helped me in `timewall` development:
- https://itnext.io/macos-mojave-dynamic-wallpaper-fd26b0698223
- https://itnext.io/macos-mojave-dynamic-wallpapers-ii-f8b1e55c82f
- https://itnext.io/macos-mojave-wallpaper-iii-c747c30935c4
- https://github.com/mczachurski/wallpapper
- https://git.spacesnek.rocks/johannes/heic-to-gnome-xml-wallpaper
