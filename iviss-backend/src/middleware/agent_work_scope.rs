use crate::app_state::AppState;
use crate::errors::AppError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;
use time::{OffsetDateTime, UtcOffset};

// Cameroon Standard Time — UTC+1,
const CAMEROON_OFFSET_HOURS: i8 = 1;

// rejects requests outside configured shift hours (local time UTC+1)
pub async fn require_shift_hours(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let offset = UtcOffset::from_hms(CAMEROON_OFFSET_HOURS, 0, 0)
        .map_err(|_| AppError::internal_error("Failed to build UTC offset"))?;

    let current_hour = OffsetDateTime::now_utc().to_offset(offset).hour() as u32;

    // Shift window: shift_start_hour (inclusive) to shift_end_hour (exclusive)
    if current_hour < state.shift_start_hour || current_hour >= state.shift_end_hour {
        tracing::warn!(
            target: "shift_hours",
            current_hour,
            shift_start = state.shift_start_hour,
            shift_end   = state.shift_end_hour,
            "OTP request rejected — outside shift hours"
        );
        return Err(AppError::unauthorized(format!(
            "Outside shift hours — login is available from {:02}:00 to {:02}:00 local time",
            state.shift_start_hour, state.shift_end_hour
        )));
    }

    Ok(next.run(request).await)
}
