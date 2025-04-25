# Stremio Linux Shell

## Development

```bash
git clone https://github.com/Stremio/stremio-linux-shell
```

This Project is using [`winit`](https://github.com/rust-windowing/winit) + [`glutin`](https://github.com/rust-windowing/glutin) with [`libmpv`](https://github.com/mpv-player/mpv/blob/master/DOCS/man/libmpv.rst) and [`CEF`](https://github.com/chromiumembedded/cef)

`winit` is used to manage windowing  
`glutin` is used to manage OpenGL context  
`libmpv` is used for the player  
`CEF` is used for the web UI  

### Prerequisites

To setup CEF, run this command:  
*This can take a while but only need to be done once*
```bash
cargo build -vv
```

#### Fedora
```bash
dnf install mpv-devel
cargo install cargo-generate-rpm
```

#### Ubuntu
```bash
apt install build-essential libssl-dev libnss3 libmpv-dev
cargo install cargo-deb
```

### Building

#### Fedora
```bash
cargo build --release
strip -s target/release/stremio-linux-shell
cargo generate-rpm
#> target/generate-rpm/*.rpm
```

#### Ubuntu
```bash
cargo build --release
cargo deb
#> target/debian/*.deb
```
