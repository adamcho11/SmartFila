//! Cédula de identidad boliviana (RN-06).
//!
//! - Número: solo dígitos, de 5 a 10.
//! - Complemento (opcional): 2 caracteres alfanuméricos que asigna el SEGIP a
//!   cédulas duplicadas (ej. `1A`). Se guarda en mayúsculas.
//!
//! Sin verificación externa: el sistema confía en lo que escribe el ciudadano.
//! El par (número, complemento) es único en todo el sistema (lo garantiza la
//! base con `UNIQUE(ci, ci_complement)`).

use std::fmt;

use super::error::DetalleCampo;

const DIGITOS_MIN: usize = 5;
const DIGITOS_MAX: usize = 10;
const LARGO_COMPLEMENTO: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ci {
    numero: String,
    complemento: Option<String>,
}

impl Ci {
    /// Valida y normaliza. Quita espacios; un complemento vacío cuenta como ausente.
    pub fn nuevo(numero: &str, complemento: Option<&str>) -> Result<Self, Vec<DetalleCampo>> {
        let mut errores = Vec::new();

        let numero = numero.trim();
        if numero.is_empty() {
            errores.push(DetalleCampo::new("ci", "obligatorio"));
        } else if !numero.chars().all(|c| c.is_ascii_digit()) {
            errores.push(DetalleCampo::new("ci", "solo dígitos"));
        } else if !(DIGITOS_MIN..=DIGITOS_MAX).contains(&numero.len()) {
            errores.push(DetalleCampo::new(
                "ci",
                format!("entre {DIGITOS_MIN} y {DIGITOS_MAX} dígitos"),
            ));
        }

        let complemento = complemento
            .map(|c| c.trim().to_ascii_uppercase())
            .filter(|c| !c.is_empty());
        if let Some(c) = &complemento {
            let valido =
                c.len() == LARGO_COMPLEMENTO && c.chars().all(|ch| ch.is_ascii_alphanumeric());
            if !valido {
                errores.push(DetalleCampo::new(
                    "ciComplement",
                    format!("{LARGO_COMPLEMENTO} caracteres alfanuméricos (ej. 1A)"),
                ));
            }
        }

        if errores.is_empty() {
            Ok(Self {
                numero: numero.to_owned(),
                complemento,
            })
        } else {
            Err(errores)
        }
    }

    pub fn numero(&self) -> &str {
        &self.numero
    }

    pub fn complemento(&self) -> Option<&str> {
        self.complemento.as_deref()
    }

    /// Valor para la columna `ci_complement`, que es `NOT NULL DEFAULT ''`: sin complemento
    /// se guarda `''` para que `UNIQUE(ci, ci_complement)` funcione (en PostgreSQL dos
    /// NULL no chocan entre sí).
    pub fn complemento_bd(&self) -> &str {
        self.complemento.as_deref().unwrap_or("")
    }
}

/// Forma canónica `1234567` o `1234567-1A`. Es la que va en el claim `ci` del JWT y la
/// que Ventas compara con el C.I. del pagador (RN-18), así que el banco simulado debe
/// enviar el mismo formato.
impl fmt::Display for Ci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.complemento {
            Some(c) => write!(f, "{}-{}", self.numero, c),
            None => f.write_str(&self.numero),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acepta_ci_sin_complemento() {
        let ci = Ci::nuevo(" 1234567 ", None).unwrap();
        assert_eq!(ci.numero(), "1234567");
        assert_eq!(ci.complemento(), None);
        assert_eq!(ci.to_string(), "1234567");
        assert_eq!(ci.complemento_bd(), "");
    }

    #[test]
    fn normaliza_el_complemento_a_mayusculas() {
        let ci = Ci::nuevo("1234567", Some(" 1a ")).unwrap();
        assert_eq!(ci.complemento(), Some("1A"));
        assert_eq!(ci.to_string(), "1234567-1A");
    }

    #[test]
    fn complemento_vacio_cuenta_como_ausente() {
        assert_eq!(
            Ci::nuevo("1234567", Some("  ")).unwrap(),
            Ci::nuevo("1234567", None).unwrap()
        );
    }

    #[test]
    fn mismo_numero_con_y_sin_complemento_son_personas_distintas() {
        assert_ne!(
            Ci::nuevo("1234567", Some("1A")).unwrap(),
            Ci::nuevo("1234567", None).unwrap()
        );
    }

    #[test]
    fn rechaza_letras_en_el_numero() {
        let e = Ci::nuevo("12A4567", None).unwrap_err();
        assert_eq!(e, vec![DetalleCampo::new("ci", "solo dígitos")]);
    }

    #[test]
    fn rechaza_largos_fuera_de_rango() {
        assert!(Ci::nuevo("1234", None).is_err());
        assert!(Ci::nuevo("12345", None).is_ok());
        assert!(Ci::nuevo("1234567890", None).is_ok());
        assert!(Ci::nuevo("12345678901", None).is_err());
    }

    #[test]
    fn rechaza_complemento_mal_formado() {
        for malo in ["1", "1AB", "1-", "ñA"] {
            let e = Ci::nuevo("1234567", Some(malo)).unwrap_err();
            assert_eq!(e[0].campo, "ciComplement", "complemento {malo:?}");
        }
    }

    #[test]
    fn reporta_numero_y_complemento_juntos() {
        let e = Ci::nuevo("", Some("XYZ")).unwrap_err();
        let campos: Vec<_> = e.iter().map(|d| d.campo).collect();
        assert_eq!(campos, ["ci", "ciComplement"]);
    }
}
