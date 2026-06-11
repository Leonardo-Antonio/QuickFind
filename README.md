# QuickFind

Un lanzador de aplicaciones ultrarrápido para Linux, inspirado en Spotlight (macOS) y Rofi. Construido con **Rust + GTK4** para máximo rendimiento y una experiencia fluida.

## Características

- **Búsqueda fuzzy** instantánea con priorización inteligente
- **Interfaz moderna** con tema oscuro y transparencias
- **Soporte para iconos** de aplicaciones
- **Navegación con teclado**: ↑ ↓ para seleccionar, Enter para lanzar, Escape para cerrar
- **Atajos numéricos**: presiona 1-9 para lanzar directamente el resultado
- **Soporte para apps en terminal** detectadas automáticamente
- **Caché de aplicaciones** para arranque inmediato
- **Configuración vía TOML** en `~/.config/quickfind/config.toml`
- **Soporte opcional para gtk4-layer-shell** (overlay en Wayland)
- **Single instance**: si ya está abierto, se enfoca la ventana existente
- Bajo consumo de recursos y arranque casi instantáneo

## Dependencias

### Obligatorias
- `gtk4` (>= 4.12)
- `pkg-config`

### Opcionales (para Wayland overlay)
- `gtk4-layer-shell`

### Instalar dependencias por distribución

**Arch Linux:**
```bash
sudo pacman -S gtk4 pkgconf
# Opcional:
sudo pacman -S gtk4-layer-shell
```

**Fedora:**
```bash
sudo dnf install gtk4-devel pkgconfig
# Opcional:
sudo dnf install gtk4-layer-shell
```

**Debian/Ubuntu:**
```bash
sudo apt install libgtk-4-dev pkg-config
# Opcional:
sudo apt install libgtk4-layer-shell-dev
```

## Compilación

```bash
# Compilar versión estándar
cargo build --release

# Compilar con soporte para layer-shell (Wayland overlay)
cargo build --release --features layer-shell
```

El binario se generará en `target/release/quickfind`.

## Instalación

```bash
sudo make install
# o con layer-shell:
sudo make install FEATURES=layer-shell
```

O manualmente:
```bash
sudo cp target/release/quickfind /usr/local/bin/
sudo cp quickfind.desktop /usr/share/applications/
```

## Uso

### Lanzar
```bash
quickfind
```

### Atajo de teclado global

Configura un atajo en tu compositor/WM:

**Sway/i3:**
```
bindsym $mod+d exec quickfind
```

**Hyprland:**
```
bind = SUPER, D, exec, quickfind
```

**GNOME (via Settings > Keyboard > Custom Shortcuts):**
```
Name: QuickFind
Command: quickfind
Shortcut: Super+Space
```

**KDE Plasma:**
Configura en System Settings > Shortcuts > Custom Shortcuts

### Controles

| Tecla | Acción |
|-------|--------|
| `↑` / `↓` | Navegar resultados |
| `Enter` | Lanzar aplicación seleccionada |
| `1`-`9` | Lanzar aplicación del resultado N |
| `Escape` | Cerrar |
| `Ctrl+R` | Recargar caché de aplicaciones |

## Configuración

QuickFind lee la configuración desde `~/.config/quickfind/config.toml`. Se crea automáticamente con valores por defecto al primer lanzamiento.

```toml
max_results = 15
window_width = 700
window_height = 500
icon_size = 32
show_icons = true
terminal_emulator = "kitty"
launch_on_single_result = true
cache_apps = true
```

### Opciones

| Opción | Descripción | Default |
|--------|-------------|---------|
| `max_results` | Cantidad máxima de resultados mostrados | 15 |
| `window_width` | Ancho de la ventana en píxeles | 700 |
| `window_height` | Alto de la ventana en píxeles | 500 |
| `icon_size` | Tamaño de los iconos en píxeles | 32 |
| `show_icons` | Mostrar u ocultar iconos | true |
| `terminal_emulator` | Emulador de terminal para apps de terminal | "kitty" |
| `launch_on_single_result` | Lanzar automáticamente si hay un único resultado | true |
| `cache_apps` | Cachear lista de aplicaciones | true |

## Créditos

Hecho con Rust y GTK4.
