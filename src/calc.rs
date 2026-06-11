// Evaluador de expresiones matemáticas simple (descenso recursivo).
// Soporta: + - * / % ^, paréntesis, decimales y signo unario.

/// Evalúa `input` como expresión matemática.
/// Devuelve `None` si no es una expresión válida o si es solo un número.
pub fn evaluate(input: &str) -> Option<f64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Debe contener al menos un operador (si no, es solo texto o un número).
    if !trimmed.chars().any(|c| matches!(c, '+' | '-' | '*' | '/' | '%' | '^')) {
        return None;
    }
    // Un número suelto (ej. "-5") no es una operación interesante.
    if trimmed.parse::<f64>().is_ok() {
        return None;
    }
    // Solo aceptamos caracteres propios de una expresión matemática.
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '*' | '/' | '%' | '^' | '.' | '(' | ')' | ' '))
    {
        return None;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let mut parser = Parser { chars, pos: 0 };
    let value = parser.parse_expr()?;
    parser.skip_ws();
    if parser.pos != parser.chars.len() {
        return None; // sobra texto sin consumir
    }
    if value.is_finite() {
        Some(value)
    } else {
        None
    }
}

/// Formatea el resultado: entero si no tiene decimales, si no recortado.
pub fn format_result(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.10}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn skip_ws(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] == ' ' {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.skip_ws();
        self.chars.get(self.pos).copied()
    }

    // expr := term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Option<f64> {
        let mut value = self.parse_term()?;
        while let Some(op) = self.peek() {
            match op {
                '+' => {
                    self.pos += 1;
                    value += self.parse_term()?;
                }
                '-' => {
                    self.pos += 1;
                    value -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Some(value)
    }

    // term := factor (('*' | '/' | '%') factor)*
    fn parse_term(&mut self) -> Option<f64> {
        let mut value = self.parse_factor()?;
        while let Some(op) = self.peek() {
            match op {
                '*' => {
                    self.pos += 1;
                    value *= self.parse_factor()?;
                }
                '/' => {
                    self.pos += 1;
                    let rhs = self.parse_factor()?;
                    if rhs == 0.0 {
                        return None;
                    }
                    value /= rhs;
                }
                '%' => {
                    self.pos += 1;
                    let rhs = self.parse_factor()?;
                    if rhs == 0.0 {
                        return None;
                    }
                    value %= rhs;
                }
                _ => break,
            }
        }
        Some(value)
    }

    // factor := unary ('^' factor)?   (potencia, asociativa por la derecha)
    fn parse_factor(&mut self) -> Option<f64> {
        let base = self.parse_unary()?;
        if let Some('^') = self.peek() {
            self.pos += 1;
            let exp = self.parse_factor()?;
            Some(base.powf(exp))
        } else {
            Some(base)
        }
    }

    // unary := ('-' | '+') unary | primary
    fn parse_unary(&mut self) -> Option<f64> {
        match self.peek() {
            Some('-') => {
                self.pos += 1;
                Some(-self.parse_unary()?)
            }
            Some('+') => {
                self.pos += 1;
                self.parse_unary()
            }
            _ => self.parse_primary(),
        }
    }

    // primary := number | '(' expr ')'
    fn parse_primary(&mut self) -> Option<f64> {
        match self.peek() {
            Some('(') => {
                self.pos += 1;
                let value = self.parse_expr()?;
                if self.peek() == Some(')') {
                    self.pos += 1;
                    Some(value)
                } else {
                    None
                }
            }
            Some(c) if c.is_ascii_digit() || c == '.' => self.parse_number(),
            _ => None,
        }
    }

    fn parse_number(&mut self) -> Option<f64> {
        self.skip_ws();
        let start = self.pos;
        let mut seen_dot = false;
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == '.' && !seen_dot {
                seen_dot = true;
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return None;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse::<f64>().ok()
    }
}
