// Conversión de timestamps epoch a fecha legible (UTC y hora local).
// Usa el comando `date` del sistema, así respeta la zona horaria local.

use std::process::Command;

pub struct TsResult {
    pub utc: String,
    pub local: String,
    /// Unidad detectada: "s" (segundos) o "ms" (milisegundos).
    pub unit: &'static str,
}

/// Convierte un timestamp epoch (segundos o milisegundos) a UTC y local.
pub fn convert(input: &str) -> Option<TsResult> {
    let trimmed = input.trim();
    if trimmed.is_empty() || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: i64 = trimmed.parse().ok()?;
    if n <= 0 {
        return None;
    }

    // Heurística: 13+ dígitos o >= 1e12 → milisegundos; si no, segundos.
    let (secs, unit) = if trimmed.len() >= 12 || n >= 1_000_000_000_000 {
        (n / 1000, "ms")
    } else {
        (n, "s")
    };

    let utc = format_date(secs, true)?;
    let local = format_date(secs, false)?;
    Some(TsResult { utc, local, unit })
}

fn format_date(secs: i64, utc: bool) -> Option<String> {
    let mut cmd = Command::new("date");
    if utc {
        cmd.arg("-u");
    }
    cmd.arg("-d")
        .arg(format!("@{}", secs))
        .arg("+%Y-%m-%d %H:%M:%S %Z");

    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
