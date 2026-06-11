# QuickFind

QuickFind es un lanzador de aplicaciones rápido para Linux, inspirado en Spotlight y Rofi. Está escrito en Rust + GTK4 y busca aplicaciones instaladas usando archivos `.desktop` estándar.

## Características

- Búsqueda fuzzy de aplicaciones por nombre, descripción, categoría y keywords.
- Calculadora integrada: escribe una operación y presiona `Enter` para copiar el resultado.
- Búsqueda web rápida con `Ctrl+Enter` usando DuckDuckGo.
- Conversión de timestamps con `:timestamp <epoch>` o `:tsp <epoch>`.
- Historial de portapapeles con `:cb`, `:clipboard` o `:cbhistory` cuando `cliphist` está disponible.
- Caché en memoria de aplicaciones y recarga con `Ctrl+R`.
- Configuración en `~/.config/quickfind/config.toml`.
- Soporte opcional de `gtk4-layer-shell` al compilar desde código fuente.

## Requisitos

QuickFind distribuye un ejecutable Linux dinámico. El usuario final no necesita Rust ni el código fuente, pero sí necesita bibliotecas y comandos de ejecución del sistema.

Dependencias obligatorias en runtime:

| Requisito | Uso |
| --- | --- |
| Linux con sesión gráfica X11 o Wayland | Mostrar la ventana GTK |
| GTK `>= 4.12` | Interfaz gráfica |
| `sh` | Ejecutar comandos de archivos `.desktop` |
| `date` | Convertir timestamps |
| `xdg-open` | Abrir búsquedas web |

Dependencias opcionales:

| Requisito | Uso |
| --- | --- |
| `cliphist` | Leer historial del portapapeles |
| `wl-copy` / `wl-clipboard` | Copiar entradas de `cliphist` en Wayland |
| Un terminal en `$TERMINAL` o `kitty` | Abrir aplicaciones `.desktop` con `Terminal=true` |
| `gtk4-layer-shell` | Overlay Wayland si compilas con `--features layer-shell` |

Notas de compatibilidad:

- El binario oficial esperado es para Linux `x86_64` con glibc. Para ARM64, Alpine/musl o distros muy antiguas, compila localmente o publica un artefacto específico.
- GTK 4.12 o superior es necesario porque el proyecto compila `gtk4` con la feature `v4_12`. Ubuntu 24.04+, Fedora recientes, Arch, openSUSE Tumbleweed y distros rolling suelen cumplirlo. Ubuntu 22.04 y Debian 12 normalmente no traen GTK suficientemente nuevo.

## Instalación para usuarios

Publica un release de GitHub con un archivo llamado:

```text
quickfind-linux-x86_64.tar.gz
```

Ese `.tar.gz` debe contener al menos:

```text
quickfind
quickfind.desktop
```

Luego los usuarios pueden instalar con:

```bash
curl -fsSL https://raw.githubusercontent.com/OWNER/QuickFind/main/install.sh | bash -s -- --repo OWNER/QuickFind
```

O, si clonaron el repo:

```bash
git clone https://github.com/OWNER/QuickFind.git
cd QuickFind
./install.sh --repo OWNER/QuickFind
```

El instalador:

- Detecta la distro por su gestor de paquetes.
- Instala dependencias de ejecución cuando puede.
- Descarga el binario desde GitHub Releases.
- Copia `quickfind` en `/usr/local/bin`.
- Copia `quickfind.desktop` en `/usr/local/share/applications`.

Para instalar un binario local ya compilado:

```bash
./install.sh --local ./target/release/quickfind
```

Para desinstalar:

```bash
./install.sh --uninstall
```

## Paquetes por distro

El instalador cubre estas familias:

| Distro | Paquetes obligatorios |
| --- | --- |
| Debian / Ubuntu / Mint / Pop!_OS | `libgtk-4-1 xdg-utils coreutils bash ca-certificates curl tar` |
| Fedora / RHEL compatibles | `gtk4 xdg-utils coreutils bash ca-certificates curl tar` |
| Arch / Manjaro / EndeavourOS | `gtk4 xdg-utils coreutils bash ca-certificates curl tar` |
| openSUSE | `gtk4 xdg-utils coreutils bash ca-certificates curl tar` |
| Void | `gtk4 xdg-utils coreutils bash ca-certificates curl tar` |
| Alpine | `gtk4.0 xdg-utils coreutils bash ca-certificates curl tar` |

Alpine usa musl, así que el binario glibc oficial puede no funcionar ahí. Para Alpine conviene publicar un build propio para musl o usar Flatpak/AppImage.

## Uso

Lanza la aplicación:

```bash
quickfind
```

También puedes abrir QuickFind con una consulta inicial:

```bash
quickfind -q firefox
quickfind --query "1 + 2 * 3"
```

Atajos dentro de QuickFind:

| Tecla | Acción |
| --- | --- |
| `Arriba` / `Abajo` | Navegar resultados |
| `Enter` | Abrir resultado seleccionado o copiar resultado especial |
| `Ctrl+Enter` | Buscar el texto actual en DuckDuckGo |
| `Ctrl+1` a `Ctrl+9` | Abrir/copiar resultado directo |
| `Ctrl+R` | Recargar caché de aplicaciones |
| `Escape` | Cerrar |

Comandos internos:

| Comando | Acción |
| --- | --- |
| `:cb texto` | Busca en historial de portapapeles si `cliphist` está instalado |
| `:timestamp 1718041112` | Convierte epoch a UTC y hora local |
| `:tsp 1718041112000` | Igual que `:timestamp`, aceptando milisegundos |

Para habilitar historial de portapapeles en Wayland:

```bash
wl-paste --watch cliphist store
```

Configura ese comando en el autostart de tu compositor si quieres que el historial persista entre sesiones.

## Atajo global recomendado

Configura un atajo en tu entorno gráfico para ejecutar `quickfind`.

| Entorno | Ejemplo |
| --- | --- |
| Sway / i3 | `bindsym $mod+d exec quickfind` |
| Hyprland | `bind = SUPER, D, exec, quickfind` |
| GNOME | Ajustes -> Teclado -> Atajos personalizados -> comando `quickfind` |
| KDE Plasma | Preferencias del Sistema -> Atajos -> Atajos personalizados |
| bspwm | `super + d : quickfind` |

## Configuración

QuickFind crea `~/.config/quickfind/config.toml` en el primer arranque.

```toml
max_results = 10
window_width = 640
window_height = 420
icon_size = 48
show_icons = true
terminal_emulator = "kitty"
launch_on_single_result = true
cache_apps = true
```

Opciones:

| Opción | Descripción |
| --- | --- |
| `max_results` | Cantidad máxima de aplicaciones mostradas |
| `window_width` | Ancho inicial de la ventana |
| `window_height` | Alto base de la ventana |
| `icon_size` | Tamaño de iconos |
| `show_icons` | Mostrar iconos de aplicaciones |
| `terminal_emulator` | Valor documentado; el launcher actualmente usa `$TERMINAL` o `kitty` |
| `launch_on_single_result` | Reservado por configuración |
| `cache_apps` | Reservado por configuración |

## Desarrollo

Instala dependencias de compilación:

| Distro | Paquetes |
| --- | --- |
| Debian / Ubuntu | `build-essential libgtk-4-dev pkg-config` |
| Fedora | `gcc gtk4-devel pkgconfig` |
| Arch | `base-devel gtk4 pkgconf` |
| openSUSE | `gcc gtk4-devel pkgconf` |
| Void | `base-devel gtk4-devel pkg-config` |

Compila:

```bash
cargo build --release
```

Compila con layer-shell:

```bash
cargo build --release --features layer-shell
```

Instala desde el árbol local:

```bash
./install.sh --local ./target/release/quickfind
```

O con `make`:

```bash
sudo make install
sudo make uninstall
```

## Crear el artefacto de release

Después de compilar:

```bash
cargo build --release --locked
./install.sh --package
```

Sube `dist/quickfind-linux-x86_64.tar.gz` a un GitHub Release. El instalador lo buscará con ese nombre.

Si quieres hacerlo manualmente, los comandos equivalentes son:

```bash
mkdir -p dist/quickfind-linux-x86_64
install -Dm755 target/release/quickfind dist/quickfind-linux-x86_64/quickfind
install -Dm644 quickfind.desktop dist/quickfind-linux-x86_64/quickfind.desktop
tar -C dist/quickfind-linux-x86_64 -czf dist/quickfind-linux-x86_64.tar.gz quickfind quickfind.desktop
```

## Troubleshooting

`quickfind: command not found`

Verifica que `/usr/local/bin` esté en tu `PATH`.

```bash
echo "$PATH"
```

La ventana no abre

QuickFind requiere sesión gráfica activa y GTK 4.12+. En SSH sin X11/Wayland no funcionará.

El historial de portapapeles no aparece

Instala `cliphist` y `wl-clipboard`, y ejecuta:

```bash
wl-paste --watch cliphist store
```

Las aplicaciones de terminal no abren

Define un terminal existente:

```bash
export TERMINAL=alacritty
```

## Licencia

MIT. Ver [LICENSE](LICENSE).
