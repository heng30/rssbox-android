use crate::slint_generatedAppWindow::{AppWindow, Logic, MessageItem, Store};
use slint::ComponentHandle;
use slint::{Timer, TimerMode, Weak};

#[macro_export]
macro_rules! message_warn {
    ($ui:expr, $msg:expr) => {
        $ui.global::<Logic>()
            .invoke_show_message(slint::format!("{}", $msg), "warning".into())
    };
}

#[macro_export]
macro_rules! message_success {
    ($ui:expr, $msg:expr) => {
        $ui.global::<Logic>()
            .invoke_show_message(slint::format!("{}", $msg), "success".into())
    };
}

#[allow(dead_code)]
#[macro_export]
macro_rules! message_info {
    ($ui:expr, $msg:expr) => {
        $ui.global::<Logic>()
            .invoke_show_message(slint::format!("{}", $msg), "info".into())
    };
}

pub fn async_message_warn(ui: Weak<AppWindow>, msg: String) {
    let _ = slint::invoke_from_event_loop(move || {
        ui.unwrap()
            .global::<Logic>()
            .invoke_show_message(slint::format!("{}", msg), "warning".into());
    });
}

pub fn async_message_success(ui: Weak<AppWindow>, msg: String) {
    let _ = slint::invoke_from_event_loop(move || {
        ui.unwrap()
            .global::<Logic>()
            .invoke_show_message(slint::format!("{}", msg), "success".into());
    });
}

#[allow(dead_code)]
pub fn async_message_info(ui: Weak<AppWindow>, msg: String) {
    let _ = slint::invoke_from_event_loop(move || {
        ui.unwrap()
            .global::<Logic>()
            .invoke_show_message(slint::format!("{}", msg), "info".into());
    });
}

pub fn init(ui: &AppWindow) {
    let timer = Timer::default();
    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_show_message(move |msg, msg_type| {
        let ui = ui_handle.unwrap();

        if timer.running() {
            timer.stop();
        }

        ui.global::<Store>().set_message(MessageItem {
            text: msg,
            text_type: msg_type,
        });

        timer.start(
            TimerMode::SingleShot,
            std::time::Duration::from_secs(2),
            move || {
                ui.global::<Store>().set_message(MessageItem {
                    text: "".into(),
                    text_type: "".into(),
                });
            },
        );
    });
}
