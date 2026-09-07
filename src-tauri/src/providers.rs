use crate::error::{JarvisError, Result};
use async_trait::async_trait;
use reqwest::{multipart, Client};
use serde_json::{json, Value};
use std::path::Path;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(&self, api_key: &str, messages: &[Value]) -> Result<String>;
    async fn health_check(&self, api_key: &str) -> Result<()>;
}
#[async_trait]
pub trait SpeechToTextProvider: Send + Sync {
    async fn transcribe(&self, api_key: &str, audio: &Path) -> Result<String>;
}
#[async_trait]
pub trait VoiceProvider: Send + Sync {
    async fn synthesize(&self, api_key: &str, text: &str, reference_id: &str) -> Result<Vec<u8>>;
}
pub trait WakeWordProvider: Send + Sync {
    fn is_local_only(&self) -> bool;
    fn model_name(&self) -> &str;
}

#[derive(Clone)]
pub struct GroqProvider {
    client: Client,
    pub model: String,
}
impl Default for GroqProvider {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("HTTP client"),
            model: "openai/gpt-oss-120b".into(),
        }
    }
}

#[async_trait]
impl AIProvider for GroqProvider {
    async fn chat(&self, key: &str, messages: &[Value]) -> Result<String> {
        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .bearer_auth(key)
            .json(&json!({"model":self.model,"messages":messages,"temperature":0.2,"stream":false}))
            .send()
            .await
            .map_err(provider)?
            .error_for_status()
            .map_err(provider)?;
        let body: Value = response.json().await.map_err(provider)?;
        body.pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| JarvisError::Provider("Groq returned no message".into()))
    }
    async fn health_check(&self, key: &str) -> Result<()> {
        self.client
            .get("https://api.groq.com/openai/v1/models")
            .bearer_auth(key)
            .send()
            .await
            .map_err(provider)?
            .error_for_status()
            .map_err(provider)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct GroqSpeechToTextProvider {
    client: Client,
}
impl Default for GroqSpeechToTextProvider {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("HTTP client"),
        }
    }
}
#[async_trait]
impl SpeechToTextProvider for GroqSpeechToTextProvider {
    async fn transcribe(&self, key: &str, audio: &Path) -> Result<String> {
        let bytes = tokio::fs::read(audio).await?;
        let filename = audio
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("command.wav")
            .to_string();
        let form = multipart::Form::new()
            .text("model", "whisper-large-v3-turbo")
            .text("response_format", "json")
            .part(
                "file",
                multipart::Part::bytes(bytes)
                    .file_name(filename)
                    .mime_str("audio/wav")
                    .map_err(provider)?,
            );
        let body: Value = self
            .client
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .bearer_auth(key)
            .multipart(form)
            .send()
            .await
            .map_err(provider)?
            .error_for_status()
            .map_err(provider)?
            .json()
            .await
            .map_err(provider)?;
        body.get("text")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| JarvisError::Provider("Groq returned no transcript".into()))
    }
}

#[derive(Clone)]
pub struct FishAudioProvider {
    client: Client,
}
impl Default for FishAudioProvider {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("HTTP client"),
        }
    }
}
#[async_trait]
impl VoiceProvider for FishAudioProvider {
    async fn synthesize(&self, key: &str, text: &str, reference_id: &str) -> Result<Vec<u8>> {
        if reference_id.trim().is_empty() {
            return Err(JarvisError::Provider(
                "Fish Audio voice/reference ID is not configured".into(),
            ));
        }
        let response=self.client.post("https://api.fish.audio/v1/tts").bearer_auth(key).header("model","s2-pro").json(&json!({"text":speech_safe(text),"reference_id":reference_id,"format":"mp3","latency":"balanced","normalize":true})).send().await.map_err(provider)?.error_for_status().map_err(provider)?;
        Ok(response.bytes().await.map_err(provider)?.to_vec())
    }
}

#[derive(Clone, Default)]
pub struct LocalWakeWordProvider;
impl WakeWordProvider for LocalWakeWordProvider {
    fn is_local_only(&self) -> bool {
        true
    }
    fn model_name(&self) -> &str {
        "openWakeWord hey_jarvis"
    }
}

fn speech_safe(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```")
            || line.len() > 500
            || line.contains("http://")
            || line.contains("https://")
        {
            continue;
        }
        if out.len() + line.len() > 650 {
            break;
        }
        out.push_str(line);
        out.push(' ')
    }
    if out.is_empty() {
        "The detailed response is displayed on screen.".into()
    } else {
        out
    }
}
fn provider<E: std::fmt::Display>(e: E) -> JarvisError {
    JarvisError::Provider(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wake_word_declares_local() {
        let p = LocalWakeWordProvider;
        assert!(p.is_local_only())
    }
    #[test]
    fn voice_summary_excludes_urls_and_code() {
        let s = speech_safe("Done.\n```\nsecret\n```\nhttps://example.com/long");
        assert!(!s.contains("http"));
        assert!(!s.contains("```"));
    }
}
