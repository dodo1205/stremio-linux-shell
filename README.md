# Stremio Linux Shell

## Development

```bash
git clone --recurse-submodules https://github.com/Stremio/stremio-linux-shell
```

This Project is using [`winit`](https://github.com/rust-windowing/winit) + [`glutin`](https://github.com/rust-windowing/glutin) with [`libmpv`](https://github.com/mpv-player/mpv/blob/master/DOCS/man/libmpv.rst) and [`CEF`](https://github.com/chromiumembedded/cef)

`winit` is used to manage windowing  
`glutin` is used to manage OpenGL context  
`libmpv` is used for the player  
`CEF` is used for the web UI  

### Building

#### Fedora
```bash
dnf install mpv-devel flatpak-builder
```

```bash
cargo build --release
```

#### Ubuntu
```bash
apt install build-essential libssl-dev libnss3 libmpv-dev flatpak-builder
```

```bash
cargo build --release
```

#### Flatpak
```bash
flatpak install -y \
    org.freedesktop.Sdk//24.08 \
    org.freedesktop.Platform//24.08 \
    org.freedesktop.Sdk.Extension.rust-stable//24.08 \
    org.freedesktop.Platform.ffmpeg-full//24.08
python3 -m pip install toml aiohttp
```

```bash
./build-aux/flatpak/build.sh
```