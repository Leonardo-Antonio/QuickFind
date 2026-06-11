// Historial del portapapeles vía `cliphist` (gestor de clipboard de Wayland).
// QuickFind no es un demonio, así que el historial lo mantiene `cliphist`
// corriendo de fondo (`wl-paste --watch cliphist store`).

use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Clone, Debug)]
pub struct ClipItem {
    pub id: String,
    pub preview: String,
}

/// ¿Está `cliphist` disponible en el sistema?
pub fn is_available() -> bool {
    Command::new("cliphist")
        .arg("version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Devuelve los últimos `limit` registros del portapapeles.
/// `filter` (opcional) filtra por texto en la vista previa.
pub fn history(limit: usize, filter: &str) -> Vec<ClipItem> {
    let output = match Command::new("cliphist").arg("list").output() {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let filter = filter.trim().to_lowercase();
    let mut items = Vec::new();

    // `cliphist list` devuelve lo más reciente primero: "<id>\t<preview>".
    for line in text.lines() {
        let Some((id, preview)) = line.split_once('\t') else {
            continue;
        };
        let preview = preview.trim();
        if preview.is_empty() {
            continue;
        }
        if !filter.is_empty() && !preview.to_lowercase().contains(&filter) {
            continue;
        }
        items.push(ClipItem {
            id: id.to_string(),
            preview: preview.to_string(),
        });
        if items.len() >= limit {
            break;
        }
    }

    items
}

/// Decodifica el registro y lo vuelve a poner en el portapapeles (`wl-copy`).
pub fn copy_to_clipboard(item: &ClipItem) -> std::io::Result<()> {
    let decoded = Command::new("cliphist")
        .arg("decode")
        .arg(&item.id)
        .output()?;

    let mut child = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&decoded.stdout)?;
    }
    child.wait()?;
    Ok(())
}
