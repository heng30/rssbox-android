use crate::slint_generatedAppWindow::{AppWindow, Logic};
use crate::util::translator::tr;
use crate::{message_success, message_warn};
use anyhow::{bail, Result};
use slint::ComponentHandle;

#[cfg(not(target_os = "android"))]
fn copy_to_clipboard(msg: &str) -> Result<()> {
    use clipboard::{ClipboardContext, ClipboardProvider};
    let ctx: Result<ClipboardContext, _> = ClipboardProvider::new();

    match ctx {
        Ok(mut ctx) => match ctx.set_contents(msg.to_string()) {
            Err(e) => bail!("{e:?}"),
            _ => Ok(()),
        },
        Err(e) => bail!("{e:?}"),
    }
}

#[cfg(not(target_os = "android"))]
fn copy_from_clipboard() -> Result<String> {
    use clipboard::{ClipboardContext, ClipboardProvider};
    let ctx: Result<ClipboardContext, _> = ClipboardProvider::new();

    match ctx {
        Ok(mut ctx) => match ctx.get_contents() {
            Err(e) => bail!("{e:?}"),
            Ok(msg) => Ok(msg),
        },
        Err(e) => bail!("{e:?}"),
    }
}

#[cfg(target_os = "android")]
fn copy_to_clipboard(msg: &str) -> Result<()> {
    match terminal_clipboard::set_string(msg) {
        Err(e) => bail!("{e:?}"),
        _ => Ok(()),
    }
}

#[cfg(target_os = "android")]
fn copy_from_clipboard() -> Result<String> {
    match terminal_clipboard::get_string() {
        Err(e) => bail!("{e:?}"),
        Ok(msg) => Ok(msg),
    }
}

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_copy_to_clipboard(move |msg| {
        let ui = ui_handle.unwrap();
        match copy_to_clipboard(&msg) {
            Err(e) => message_warn!(ui, format!("{}. {}: {e:?}", tr("复制失败"), tr("原因"))),
            _ => message_success!(ui, tr("复制成功")),
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_copy_from_clipboard(move || {
        let ui = ui_handle.unwrap();
        match copy_from_clipboard() {
            Err(e) => {
                message_warn!(ui, format!("{}. {}: {e:?}", tr("粘贴失败"), tr("原因")));
                slint::SharedString::default()
            }
            Ok(msg) => msg.into(),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_clipboard() -> Result<()> {
        let msg = "hello world";
        copy_to_clipboard(msg)?;
        let res = copy_from_clipboard()?;

        assert_eq!(msg, res);
        Ok(())
    }
}
