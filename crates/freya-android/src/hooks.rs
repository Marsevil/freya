use freya_core::{
    integration::State,
    lifecycle::{
        base::use_hook,
        context::{provide_context, use_consume},
    },
};

use crate::window_insets::AndroidWindowInsets;

pub fn use_provide_android_window_insets(
    builder: impl FnOnce() -> AndroidWindowInsets,
) -> State<AndroidWindowInsets> {
    use_hook(move || {
        let state = State::create(builder());
        provide_context(state);
        state
    })
}

pub fn use_android_window_insets() -> State<AndroidWindowInsets> {
    use_consume()
}
