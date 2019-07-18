use std::time::Duration;

use nalgebra::Vector3;

pub fn format_time(instant: Duration) -> String {
    let seconds = instant.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub fn component_min(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
    Vector3::new(a.x.min(b.x),
                 a.y.min(b.y),
                 a.z.min(b.z)
    )
}

pub fn component_max(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32> {
    Vector3::new(a.x.max(b.x),
                 a.y.max(b.y),
                 a.z.max(b.z)
    )
}
