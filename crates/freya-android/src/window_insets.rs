use jni::{Env, JValue, jni_sig, jni_str, objects::JObject};
use winit::platform::android::activity::AndroidApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InsetWindowType {
    Ime,
    SystemBar,
    Gestures,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct AndroidWindowInsets {
    pub ime_height: f32,
    pub system_bar_height: f32,
    pub gestures_height: f32,
}

impl InsetWindowType {
    pub fn to_jni(self, env: &mut Env) -> jni::errors::Result<i32> {
        match self {
            InsetWindowType::Ime => env
                .call_static_method(
                    jni_str!("android/view/WindowInsets$Type"),
                    jni_str!("ime"),
                    jni_sig!("()I"),
                    &[],
                )?
                .i(),
            InsetWindowType::SystemBar => env
                .call_static_method(
                    jni_str!("android/view/WindowInsets$Type"),
                    jni_str!("systemBars"),
                    jni_sig!("()I"),
                    &[],
                )?
                .i(),
            InsetWindowType::Gestures => env
                .call_static_method(
                    jni_str!("android/view/WindowInsets$Type"),
                    jni_str!("systemGestures"),
                    jni_sig!("()I"),
                    &[],
                )?
                .i(),
        }
    }
}

pub fn get_window_insets(app: &AndroidApp) -> jni::errors::Result<AndroidWindowInsets> {
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) };
    vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, app.activity_as_ptr() as *mut _) };

        let window = env
            .call_method(
                &activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let decor = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let insets = env
            .call_method(
                &decor,
                jni_str!("getRootWindowInsets"),
                jni_sig!("()Landroid/view/WindowInsets;"),
                &[],
            )?
            .l()?;
        if insets.is_null() {
            return Ok(AndroidWindowInsets::default());
        }

        let ime_inset_jni_type = InsetWindowType::Ime.to_jni(env)?;
        let ime_insets = env
            .call_method(
                &insets,
                jni_str!("getInsets"),
                jni_sig!("(I)Landroid/graphics/Insets;"),
                &[JValue::Int(ime_inset_jni_type)],
            )?
            .l()?;
        let ime_height = env
            .get_field(&ime_insets, jni_str!("bottom"), jni_sig!("I"))?
            .i()
            .map(|val| val as f32)?;

        let system_bar_jni_type = InsetWindowType::SystemBar.to_jni(env)?;
        let system_bar_insets = env
            .call_method(
                &insets,
                jni_str!("getInsets"),
                jni_sig!("(I)Landroid/graphics/Insets;"),
                &[JValue::Int(system_bar_jni_type)],
            )?
            .l()?;
        let system_bar_height = env
            .get_field(&system_bar_insets, jni_str!("bottom"), jni_sig!("I"))?
            .i()
            .map(|val| val as f32)?;

        let gestures_jni_type = InsetWindowType::Gestures.to_jni(env)?;
        let gestures_insets = env
            .call_method(
                &insets,
                jni_str!("getInsets"),
                jni_sig!("(I)Landroid/graphics/Insets;"),
                &[JValue::Int(gestures_jni_type)],
            )?
            .l()?;
        let gestures_height = env
            .get_field(&gestures_insets, jni_str!("bottom"), jni_sig!("I"))?
            .i()
            .map(|val| val as f32)?;

        Ok(AndroidWindowInsets {
            ime_height,
            system_bar_height,
            gestures_height,
        })
    })
}
