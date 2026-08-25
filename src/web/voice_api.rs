//! 网页端语音合成（TTS）接口。
//!
//! 供 WebUI 前端按需合成文本为自然语音 MP3，支持自定义音色、音调、语速。

use crate::voice::engines::edge_tts::EdgeTtsEngine;
use crate::web::*;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::Response;

#[derive(Deserialize)]
pub(in crate::web) struct SynthesizeVoiceRequest {
    pub(in crate::web) text: String,
    #[serde(default)]
    pub(in crate::web) voice: Option<String>,
    #[serde(default)]
    pub(in crate::web) pitch: Option<String>,
    #[serde(default)]
    pub(in crate::web) rate: Option<String>,
    #[serde(default)]
    pub(in crate::web) volume: Option<String>,
}

pub(in crate::web) async fn synthesize_voice_http(
    State(state): State<DaemonState>,
    headers: HeaderMap,
    Json(request): Json<SynthesizeVoiceRequest>,
) -> std::result::Result<Response, ApiError> {
    require_auth(&headers, &state)?;

    let text = request.text.trim();
    if text.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "text cannot be empty",
        ));
    }

    let mut config = state.manager.lock().unwrap().config.voice.clone();
    config.enabled = true;
    if let Some(voice) = request.voice {
        config.voice = voice;
    }
    if let Some(pitch) = request.pitch {
        config.pitch = pitch;
    }
    if let Some(rate) = request.rate {
        config.rate = rate;
    }
    if let Some(volume) = request.volume {
        config.volume = volume;
    }

    let engine = EdgeTtsEngine::new();
    let audio_bytes = engine
        .synthesize(text, &config)
        .await
        .map_err(|err| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("TTS synthesis failed: {err:#}")))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "audio/mpeg")
        .header("Cache-Control", "no-cache")
        .body(Body::from(audio_bytes))
        .map_err(|err| ApiError::internal(anyhow::anyhow!("failed to construct response: {err}")))?;

    Ok(response)
}
