# Publicar QuickFind en GitHub

Esta app debe distribuirse como ejecutable, no como proyecto Rust para usuarios finales. El flujo recomendado es publicar un GitHub Release con un `.tar.gz` que contenga el binario y el archivo `.desktop`.

## 1. Actualiza el repositorio en el instalador

Antes de publicar, cambia `DEFAULT_REPO` en `install.sh`:

```bash
DEFAULT_REPO="OWNER/QuickFind"
```

También actualiza los ejemplos del `README.md` reemplazando `OWNER` por tu usuario u organización real.

Los usuarios pueden evitar ese cambio pasando:

```bash
./install.sh --repo OWNER/QuickFind
```

pero dejar el repo por defecto correcto hace que el instalador sea más simple.

## 2. Crea un tag

```bash
git tag v1.0.0
git push origin v1.0.0
```

El workflow `.github/workflows/release.yml` compila en Ubuntu 24.04 y sube:

```text
quickfind-linux-x86_64.tar.gz
```

Para generar el mismo archivo localmente:

```bash
cargo build --release --locked
./install.sh --package
```

El archivo queda en:

```text
dist/quickfind-linux-x86_64.tar.gz
```

## 3. Verifica el release

Después de que termine GitHub Actions, prueba en una máquina Linux limpia:

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/QuickFind/main/install.sh | bash -s -- --repo OWNER/QuickFind
quickfind
```

## Dependencias del binario

El artefacto generado es dinámico y necesita GTK 4.12+ en el sistema del usuario. Eso mantiene el paquete pequeño, pero no garantiza compatibilidad con distros antiguas.

Para cubrir más sistemas, considera publicar además:

- AppImage para usuarios que no quieren instalar dependencias del sistema.
- Flatpak para desktops Linux modernos.
- Un build ARM64 si quieres soportar aarch64.
- Un build específico para musl si quieres soportar Alpine.
