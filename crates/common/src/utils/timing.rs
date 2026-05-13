#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::js_sys::Date;

pub struct Timing {
    #[cfg(not(target_family = "wasm"))]
    started_at: std::time::Instant,
    #[cfg(target_family = "wasm")]
    started_at_ms: f64,
}

impl Timing {
    pub fn now() -> Self {
        Self {
            #[cfg(not(target_family = "wasm"))]
            started_at: std::time::Instant::now(),
            #[cfg(target_family = "wasm")]
            started_at_ms: Date::now(),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        #[cfg(not(target_family = "wasm"))]
        {
            self.started_at.elapsed().as_millis() as u64
        }

        #[cfg(target_family = "wasm")]
        {
            let elapsed = Date::now() - self.started_at_ms;
            if elapsed.is_finite() && elapsed > 0.0 {
                elapsed as u64
            } else {
                0
            }
        }
    }
}
