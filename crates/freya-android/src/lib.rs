#![cfg(target_os = "android")]

use std::time::Duration;

use freya_components::theming::{hooks::use_init_theme, themes::light_theme};
use freya_core::{integration::*, prelude::*};
use freya_winit::{
    integration::is_ime_role,
    plugins::{FreyaPlugin, PluginEvent, PluginHandle},
};
use winit::platform::android::activity::AndroidApp;

use crate::{
    hooks::use_provide_android_window_insets,
    window_insets::{AndroidWindowInsets, get_window_insets},
};

pub mod hooks;
mod keyboard;
mod status_bar;
mod window_insets;

/// Extension trait for [`Platform`] providing Android-specific APIs.
pub trait AndroidExt {
    /// Set whether the Android status bar uses light appearance (dark icons for light backgrounds).
    fn set_status_bar_light(&self, light: bool) -> Result<(), jni::errors::Error>;

    /// Show the Android soft keyboard.
    fn show_keyboard(&self) -> Result<(), jni::errors::Error>;

    /// Hide the Android soft keyboard.
    fn hide_keyboard(&self) -> Result<(), jni::errors::Error>;

    fn get_window_insets(&self) -> Option<AndroidWindowInsets>;
}

impl AndroidExt for Platform {
    fn set_status_bar_light(&self, light: bool) -> Result<(), jni::errors::Error> {
        if let Some(app) = GlobalContexts::get().try_get_context::<AndroidApp>() {
            status_bar::set_status_bar_light(&app, light)
        } else {
            Ok(())
        }
    }

    fn show_keyboard(&self) -> Result<(), jni::errors::Error> {
        if let Some(app) = GlobalContexts::get().try_get_context::<AndroidApp>() {
            keyboard::show_keyboard(&app)
        } else {
            Ok(())
        }
    }

    fn hide_keyboard(&self) -> Result<(), jni::errors::Error> {
        if let Some(app) = GlobalContexts::get().try_get_context::<AndroidApp>() {
            keyboard::hide_keyboard(&app)
        } else {
            Ok(())
        }
    }

    fn get_window_insets(&self) -> Option<AndroidWindowInsets> {
        let Some(app) = GlobalContexts::get().try_get_context::<AndroidApp>() else {
            log::error!("Unable to retrieve AndroidApp for window insets");
            return None;
        };

        match get_window_insets(&app) {
            Ok(value) => Some(value),
            Err(jni_error) => {
                log::error!("{:?}", jni_error);
                None
            }
        }
    }
}

/// Freya plugin for Android integration.
///
/// Stores the [`AndroidApp`] handle, provides it as root context,
/// and registers a root component that syncs the status bar
/// appearance with the app theme and manages the soft keyboard.
pub struct AndroidPlugin {
    app: AndroidApp,
}

impl AndroidPlugin {
    pub fn new(app: AndroidApp) -> Self {
        Self { app }
    }
}

impl FreyaPlugin for AndroidPlugin {
    fn plugin_id(&self) -> &'static str {
        "android"
    }

    fn on_event(&mut self, event: &mut PluginEvent, _handle: PluginHandle) {
        if let PluginEvent::RunnerCreated { .. } = event {
            GlobalContexts::get().insert_context(self.app.clone());
        }
    }

    fn root_component(&self, root: Element) -> Element {
        AndroidRoot { inner: root }.into_element()
    }
}

/// Root component that manages Android platform integration.
#[derive(Clone)]
struct AndroidRoot {
    inner: Element,
}

impl PartialEq for AndroidRoot {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Component for AndroidRoot {
    fn render(&self) -> impl IntoElement {
        let theme = use_init_theme(light_theme);

        let mut self_is_keyboard_opened = use_state(move || false);
        let mut keyboard_watcher = use_state(move || Option::<OwnedTaskHandle>::None);

        let mut android_window_insets =
            use_provide_android_window_insets(AndroidWindowInsets::default);

        // Sync status bar appearance with theme
        use_side_effect(move || {
            let platform = Platform::get();
            let is_light = theme.read().name == "light";
            if let Err(err) = platform.set_status_bar_light(is_light) {
                tracing::error!("Failed to set status bar appearance: {err:?}");
            }
        });

        // Show/hide keyboard based on focused node type
        use_side_effect(move || {
            let platform = Platform::get();
            let window = platform.current_window();
            let focused_node = window.focused_accessibility_node.read();
            let result = if is_ime_role(focused_node.role()) {
                (platform.show_keyboard(), true)
            } else {
                (platform.hide_keyboard(), false)
            };
            match result {
                (Ok(()), is_keyboard_opened) => self_is_keyboard_opened.set(is_keyboard_opened),
                (Err(err), _) => tracing::error!("Failed to toggle soft keyboard: {err:?}"),
            }
        });

        use_side_effect(move || {
            let platform = Platform::get();
            android_window_insets.set(platform.get_window_insets().unwrap_or_default());
        });

        use_side_effect(move || {
            if !self_is_keyboard_opened() {
                return;
            }

            let task = spawn(async move {
                loop {
                    timer(Duration::from_millis(200)).await;

                    let platform = Platform::get();
                    let new_window_insets = platform.get_window_insets().unwrap_or_default();
                    let Some(new_window_insets) = platform.get_window_insets() else {
                        log::warn!("Unable to get android window insets.");
                        break;
                    };

                    android_window_insets.set_if_modified(new_window_insets);

                    if new_window_insets.ime_height == 0. {
                        break;
                    }
                }
            })
            .owned();

            keyboard_watcher.set(task.into());
        });

        self.inner.clone()
    }
}
