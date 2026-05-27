use stuk_layout::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerformanceSample {
    pub frame_time_ms: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: Option<f32>,
    pub layout_time_ms: f32,
    pub render_time_ms: f32,
    pub text_shaping_time_ms: f32,
    pub draw_calls: u32,
    pub dirty_region: Option<Rect>,
    pub glyph_cache_bytes: u64,
    pub image_cache_bytes: u64,
    pub memory_bytes: u64,
}

impl PerformanceSample {
    pub fn frame_time_ms(frame_time_ms: f32) -> Self {
        Self {
            frame_time_ms,
            cpu_time_ms: 0.0,
            gpu_time_ms: None,
            layout_time_ms: 0.0,
            render_time_ms: 0.0,
            text_shaping_time_ms: 0.0,
            draw_calls: 0,
            dirty_region: None,
            glyph_cache_bytes: 0,
            image_cache_bytes: 0,
            memory_bytes: 0,
        }
    }

    pub fn fps(self) -> f32 {
        if self.frame_time_ms <= 0.0 {
            0.0
        } else {
            1000.0 / self.frame_time_ms
        }
    }

    pub fn health(self) -> FrameHealth {
        if self.frame_time_ms <= 16.7 {
            FrameHealth::Smooth
        } else if self.frame_time_ms <= 33.4 {
            FrameHealth::Slow
        } else {
            FrameHealth::Dropped
        }
    }

    pub fn overlay_lines(self) -> Vec<String> {
        let mut lines = vec![
            format!("FPS {:.0}", self.fps()),
            format!("Frame {:.2} ms", self.frame_time_ms),
            format!("CPU {:.2} ms", self.cpu_time_ms),
            format!("Layout {:.2} ms", self.layout_time_ms),
            format!("Render {:.2} ms", self.render_time_ms),
            format!("Text {:.2} ms", self.text_shaping_time_ms),
            format!("Draw calls {}", self.draw_calls),
        ];
        if let Some(gpu_time_ms) = self.gpu_time_ms {
            lines.push(format!("GPU {gpu_time_ms:.2} ms"));
        }
        if let Some(rect) = self.dirty_region {
            lines.push(format!(
                "Dirty {:.0}x{:.0} @ {:.0},{:.0}",
                rect.width, rect.height, rect.x, rect.y
            ));
        }
        lines.push(format!(
            "Glyph cache {}",
            format_bytes(self.glyph_cache_bytes)
        ));
        lines.push(format!(
            "Image cache {}",
            format_bytes(self.image_cache_bytes)
        ));
        lines.push(format!("Memory {}", format_bytes(self.memory_bytes)));
        lines
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameHealth {
    Smooth,
    Slow,
    Dropped,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PerformanceOverlay {
    samples: Vec<PerformanceSample>,
    capacity: usize,
}

impl PerformanceOverlay {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn push(&mut self, sample: PerformanceSample) {
        if self.samples.len() == self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(sample);
    }

    pub fn samples(&self) -> &[PerformanceSample] {
        &self.samples
    }

    pub fn latest(&self) -> Option<PerformanceSample> {
        self.samples.last().copied()
    }

    pub fn average_frame_time_ms(&self) -> Option<f32> {
        if self.samples.is_empty() {
            return None;
        }
        Some(
            self.samples
                .iter()
                .map(|sample| sample.frame_time_ms)
                .sum::<f32>()
                / self.samples.len() as f32,
        )
    }

    pub fn average_fps(&self) -> Option<f32> {
        self.average_frame_time_ms()
            .map(|ms| if ms <= 0.0 { 0.0 } else { 1000.0 / ms })
    }

    pub fn health(&self) -> Option<FrameHealth> {
        self.latest().map(PerformanceSample::health)
    }

    pub fn overlay_lines(&self) -> Vec<String> {
        self.latest()
            .map(PerformanceSample::overlay_lines)
            .unwrap_or_default()
    }
}

impl Default for PerformanceOverlay {
    fn default() -> Self {
        Self::new(120)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f32 = 1024.0;
    const MB: f32 = KB * 1024.0;
    let bytes = bytes as f32;
    if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{bytes:.0} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_latest_samples_with_capacity() {
        let mut overlay = PerformanceOverlay::new(2);
        overlay.push(PerformanceSample::frame_time_ms(12.0));
        overlay.push(PerformanceSample::frame_time_ms(18.0));
        overlay.push(PerformanceSample::frame_time_ms(42.0));

        assert_eq!(overlay.samples().len(), 2);
        assert_eq!(overlay.average_frame_time_ms(), Some(30.0));
        assert_eq!(overlay.average_fps(), Some(1000.0 / 30.0));
        assert_eq!(overlay.health(), Some(FrameHealth::Dropped));
        assert!(overlay.overlay_lines()[0].starts_with("FPS "));
    }
}
