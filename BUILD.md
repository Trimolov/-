# Сборка XYZ Coords (.hmod)

Нужен ПК (Windows/Linux/macOS) с Rust.

```
rustup target add wasm32-unknown-unknown
cargo install --git https://github.com/horizon-rust/horizon --rev 83f15fa276c44711276b6ceb5f0fe0c9c534dc66 cargo-horizon-mod
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/xyz_coords.hmod
```

Готовый файл: `target/horizon-mod/xyz_coords.hmod`.

Установка на Android: выложить .hmod по прямой HTTPS-ссылке (например, GitHub Release),
в лаунчере Horizon Android открыть Mods, вставить ссылку, затем включить мод во вкладке Installed.
Работает только на 64-битном APK (arm64-v8a / x86_64).
