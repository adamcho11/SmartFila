//! Firma RS256 del JWT (ADR-012) y publicación de la clave pública en el JWKS.
//!
//! Solo Identidad tiene la clave privada. Cola, Inventario, Ventas y el gateway
//! validan con la clave pública: el archivo montado `/keys/jwt.pub` o
//! `GET /.well-known/jwks.json`.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{
    pkcs1::DecodeRsaPrivateKey, pkcs8::DecodePrivateKey, traits::PublicKeyParts, RsaPrivateKey,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::dominio::token::{Claims, AUDIENCIA, EMISOR};

#[derive(Debug, thiserror::Error)]
pub enum ErrorClave {
    #[error("la clave privada no es un PEM RSA válido (PKCS#8 o PKCS#1)")]
    PemInvalido,
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

/// Clave pública en formato JWK (RFC 7517).
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Jwk {
    #[schema(example = "RSA")]
    pub kty: String,
    #[schema(example = "RS256")]
    pub alg: String,
    #[serde(rename = "use")]
    #[schema(example = "sig")]
    pub uso: String,
    #[schema(example = "identidad-1")]
    pub kid: String,
    /// Módulo en base64url.
    pub n: String,
    /// Exponente en base64url.
    #[schema(example = "AQAB")]
    pub e: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

#[derive(Clone)]
pub struct Claves {
    kid: String,
    firma: EncodingKey,
    verificacion: DecodingKey,
    jwk: Jwk,
}

impl Claves {
    pub fn desde_pem(pem_privado: &str, kid: impl Into<String>) -> Result<Self, ErrorClave> {
        let privada = RsaPrivateKey::from_pkcs8_pem(pem_privado)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem_privado))
            .map_err(|_| ErrorClave::PemInvalido)?;
        let publica = privada.to_public_key();
        let n = URL_SAFE_NO_PAD.encode(publica.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(publica.e().to_bytes_be());
        let kid = kid.into();

        Ok(Self {
            firma: EncodingKey::from_rsa_pem(pem_privado.as_bytes())?,
            verificacion: DecodingKey::from_rsa_components(&n, &e)?,
            jwk: Jwk {
                kty: "RSA".into(),
                alg: "RS256".into(),
                uso: "sig".into(),
                kid: kid.clone(),
                n,
                e,
            },
            kid,
        })
    }

    pub fn firmar(&self, claims: &Claims) -> Result<String, ErrorClave> {
        let mut cabecera = Header::new(Algorithm::RS256);
        cabecera.kid = Some(self.kid.clone());
        Ok(jsonwebtoken::encode(&cabecera, claims, &self.firma)?)
    }

    /// Valida la firma, `iss`, `aud` y `exp` (sin margen). Solo acepta RS256: un
    /// token con `alg: none` o HS256 se rechaza.
    pub fn verificar(&self, token: &str) -> Result<Claims, ErrorClave> {
        let mut v = Validation::new(Algorithm::RS256);
        v.set_issuer(&[EMISOR]);
        v.set_audience(&[AUDIENCIA]);
        v.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        v.leeway = 0;
        Ok(jsonwebtoken::decode::<Claims>(token, &self.verificacion, &v)?.claims)
    }

    pub fn jwks(&self) -> Jwks {
        Jwks {
            keys: vec![self.jwk.clone()],
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use chrono::Utc;
    use rsa::pkcs8::{EncodePrivateKey, LineEnding};
    use std::sync::OnceLock;
    use uuid::Uuid;

    /// Una sola clave de 2048 bits para todas las pruebas (generarla tarda ~1 s).
    pub(crate) fn pem_de_prueba() -> &'static str {
        static PEM: OnceLock<String> = OnceLock::new();
        PEM.get_or_init(|| {
            let clave = RsaPrivateKey::new(&mut rand::rngs::OsRng, 2048).unwrap();
            clave.to_pkcs8_pem(LineEnding::LF).unwrap().to_string()
        })
    }

    fn claims(exp_en_segundos: i64) -> Claims {
        let ahora = Utc::now().timestamp();
        Claims {
            iss: EMISOR.into(),
            aud: AUDIENCIA.into(),
            sub: Uuid::from_u128(1),
            sid: Uuid::from_u128(2),
            roles: vec!["CIUDADANO".into()],
            org: None,
            ci: "1234567".into(),
            name: "Ana Pérez".into(),
            iat: ahora,
            exp: ahora + exp_en_segundos,
        }
    }

    #[test]
    fn firma_y_verifica_ida_y_vuelta() {
        let claves = Claves::desde_pem(pem_de_prueba(), "k1").unwrap();
        let c = claims(600);
        let token = claves.firmar(&c).unwrap();
        assert_eq!(claves.verificar(&token).unwrap(), c);
        let cabecera = jsonwebtoken::decode_header(&token).unwrap();
        assert_eq!(cabecera.alg, Algorithm::RS256);
        assert_eq!(cabecera.kid.as_deref(), Some("k1"));
    }

    #[test]
    fn rechaza_un_token_vencido() {
        let claves = Claves::desde_pem(pem_de_prueba(), "k1").unwrap();
        let token = claves.firmar(&claims(-1)).unwrap();
        assert!(claves.verificar(&token).is_err());
    }

    #[test]
    fn rechaza_otra_audiencia() {
        let claves = Claves::desde_pem(pem_de_prueba(), "k1").unwrap();
        let mut c = claims(600);
        c.aud = "otro-sistema".into();
        assert!(claves.verificar(&claves.firmar(&c).unwrap()).is_err());
    }

    #[test]
    fn rechaza_un_token_hs256() {
        let claves = Claves::desde_pem(pem_de_prueba(), "k1").unwrap();
        let falso = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims(600),
            &EncodingKey::from_secret(b"adivinado"),
        )
        .unwrap();
        assert!(claves.verificar(&falso).is_err());
    }

    #[test]
    fn el_jwks_sirve_para_verificar() {
        let claves = Claves::desde_pem(pem_de_prueba(), "k1").unwrap();
        let jwk = &claves.jwks().keys[0];
        assert_eq!(
            (jwk.kty.as_str(), jwk.alg.as_str(), jwk.e.as_str()),
            ("RSA", "RS256", "AQAB")
        );

        let token = claves.firmar(&claims(600)).unwrap();
        let publica = DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap();
        let mut v = Validation::new(Algorithm::RS256);
        v.set_audience(&[AUDIENCIA]);
        assert!(jsonwebtoken::decode::<Claims>(&token, &publica, &v).is_ok());
    }

    #[test]
    fn un_pem_invalido_da_error() {
        assert!(matches!(
            Claves::desde_pem("no es un pem", "k1"),
            Err(ErrorClave::PemInvalido)
        ));
    }
}
