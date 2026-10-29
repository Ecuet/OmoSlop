use slint::Window;

#[macro_export]
macro_rules! setup_window {
    ($window: expr) => {
        let gif_ui_clone = $window.as_weak();
        $window.on_MoveWindow(move |x, y| {
            let pin_win_clone = gif_ui_clone.unwrap();
            let logical_pos = pin_win_clone.window().position().to_logical(pin_win_clone.window().scale_factor());
            pin_win_clone.window().set_position(slint::LogicalPosition::new(logical_pos.x + x, logical_pos.y + y));
        });
    };
}