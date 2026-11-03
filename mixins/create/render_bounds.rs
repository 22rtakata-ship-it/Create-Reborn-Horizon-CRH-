fn create_render_bounds(block: &str, bounds: &mut [f32; 6]) {
    if block == "create:large_water_wheel" {
        *bounds = [-2.0, -2.0, -2.0, 3.0, 3.0, 3.0];
    }
}
