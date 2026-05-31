#[allow(dead_code)]
pub(crate) fn try_request_blur(_radius: f32) -> bool {
    has_background_effect_protocol()
}

#[allow(dead_code)]
pub(crate) fn has_background_effect_protocol() -> bool {
    std::env::var("STUK_WAYLAND_BACKGROUND_EFFECT_V1")
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}
