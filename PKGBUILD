# Maintainer: Leonardo <leo2001.cmc@gmail.com>
pkgname=quickfind
pkgver=1.0.0
pkgrel=1
pkgdesc="A fast Spotlight/Rofi-like application launcher for Linux (GTK4 + layer-shell)"
arch=('x86_64')
url="https://github.com/Leonardo-Antonio/quickfind"
license=('MIT')
depends=('gtk4' 'gtk4-layer-shell')
makedepends=('cargo')
optdepends=(
  'cliphist: clipboard history support (:cb)'
  'wl-clipboard: copy entries back to the clipboard'
)
# Para empaquetar desde el código local. Para AUR, usa source=() con un tag.
source=()
sha256sums=()

build() {
  cd "$startdir"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --release --frozen --features layer-shell
}

package() {
  cd "$startdir"
  install -Dm755 "target/release/quickfind" "$pkgdir/usr/bin/quickfind"
  install -Dm644 "quickfind.desktop" "$pkgdir/usr/share/applications/quickfind.desktop"
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
