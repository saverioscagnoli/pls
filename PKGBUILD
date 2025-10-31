pkgname=pls
pkgver=0.1.0
pkgrel=1
pkgdesc="A modern successor of 'ls'"
arch=('x86_64')
url="https://github.com/saverioscagnoli/pls"
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('c3dd291ca02d43e52e78e3b5b6f221016972880f72c3b7d41150628b871fd01a')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}


package() {
    cd "$pkgname-$pkgver"
    install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}