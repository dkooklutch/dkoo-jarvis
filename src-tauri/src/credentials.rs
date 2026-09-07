use crate::error::{JarvisError, Result};

const SERVICE: &str = "ai.dkoo.jarvis";

#[derive(Clone, Default)]
pub struct CredentialStore;

impl CredentialStore {
    fn account(provider: &str) -> Result<&'static str> {
        match provider {
            "groq" => Ok("dkoo_JARVIS"),
            "fish_audio" => Ok("dkoo_JARVIS_voice"),
            _ => Err(JarvisError::Security("unknown credential provider".into())),
        }
    }
    pub fn set(&self, provider: &str, value: &str) -> Result<String> {
        if value.trim().len() < 8 {
            return Err(JarvisError::Security("credential is too short".into()));
        }
        let account = Self::account(provider)?;
        keyring::Entry::new(SERVICE, account)
            .map_err(|e| JarvisError::Storage(e.to_string()))?
            .set_password(value.trim())
            .map_err(|e| JarvisError::Storage(e.to_string()))?;
        Ok(mask(value.trim()))
    }
    pub fn get(&self, provider: &str) -> Result<String> {
        let account = Self::account(provider)?;
        if let Ok(value) = std::env::var(account) {
            if !value.trim().is_empty() {
                return Ok(value);
            }
        }
        keyring::Entry::new(SERVICE, account)
            .map_err(|e| JarvisError::Storage(e.to_string()))?
            .get_password()
            .map_err(|_| JarvisError::CredentialMissing)
    }
    pub fn remove(&self, provider: &str) -> Result<()> {
        let account = Self::account(provider)?;
        let entry = keyring::Entry::new(SERVICE, account)
            .map_err(|e| JarvisError::Storage(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(JarvisError::Storage(e.to_string())),
        }
    }
    pub fn masked(&self, provider: &str) -> Option<String> {
        self.get(provider).ok().map(|v| mask(&v))
    }
}

fn mask(value: &str) -> String {
    let tail: String = value
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("••••••••••{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_never_exposes_full_secret() {
        let secret = "gsk_super_secret_42AB";
        let shown = mask(secret);
        assert_eq!(shown, "••••••••••42AB");
        assert!(!shown.contains("super"));
    }
}
