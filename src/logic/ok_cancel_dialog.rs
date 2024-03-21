use crate::slint_generatedAppWindow::{AppWindow, Logic};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_handle_ok_cancel_dialog(move |handle_type, handle_uuid| {
            let ui = ui_handle.unwrap();

            match handle_type.as_str() {
                "address-book-delete-item" => {
                    ui.global::<Logic>()
                        .invoke_address_book_delete_item(handle_uuid);
                }
                "activity-delete-item" => {
                    ui.global::<Logic>()
                        .invoke_activity_delete_item(handle_uuid);
                }
                _ => (),
            }
        });
}
