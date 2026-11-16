// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::s2sdk::ConCommandContext;
use plugify::Str;

type GetChatPrefix = unsafe extern "C" fn(&mut Str) -> bool;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
static mut __imomoe_chat_prefix_GetChatPrefix: Option<GetChatPrefix> = None;

pub(crate) fn for_reply(
    slot: i32,
    context: ConCommandContext,
    marker: Option<&str>,
    body: &str,
) -> String {
    let message = marker.map_or_else(|| body.to_owned(), |marker| format!("{marker} {body}"));
    replace(slot, context, marker, &message, || {
        let get = unsafe { std::ptr::addr_of!(__imomoe_chat_prefix_GetChatPrefix).read() }?;
        let mut prefix = Str::new();
        unsafe { get(&mut prefix) }.then(|| prefix.as_str().to_owned())
    })
}

fn valid_prefix(prefix: &str) -> bool {
    prefix.len() <= 96
        && prefix.starts_with(' ')
        && prefix.ends_with("\x01 ")
        && prefix
            .chars()
            .all(|ch| !ch.is_control() || ('\x01'..='\x10').contains(&ch))
        && prefix
            .chars()
            .any(|ch| !ch.is_whitespace() && !ch.is_control())
}

fn replace(
    slot: i32,
    context: ConCommandContext,
    marker: Option<&str>,
    message: &str,
    get_prefix: impl FnOnce() -> Option<String>,
) -> String {
    let body = marker
        .and_then(|marker| message.strip_prefix(marker))
        .map(|body| body.strip_prefix(' ').unwrap_or(body));
    if slot >= 0 && context == ConCommandContext::Chat {
        if let Some(body) = body {
            let Some(prefix) = get_prefix().filter(|prefix| valid_prefix(prefix)) else {
                return body.to_owned();
            };
            // S2SDK supplies the leading space and treats a literal LF as a line break.
            let prefix = prefix[1..].replace('\x0A', "{bluegrey}");
            return format!("{prefix}{body}");
        }
        return message.to_owned();
    }
    body.unwrap_or(message)
        .chars()
        .filter(|ch| !('\x01'..='\x10').contains(ch))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ConCommandContext::{Chat, Console};

    #[test]
    fn configured_markers_replace_or_strip_for_chat_and_console() {
        for marker in ["{prefix}", "[community]", "PREFIX:"] {
            let message = format!("{marker} \x04Body");
            let get = || Some(" \x0A[Community]\x01 ".to_owned());
            assert_eq!(
                replace(2, Chat, Some(marker), &message, get),
                "{bluegrey}[Community]\x01 \x04Body"
            );
            assert_eq!(
                replace(2, Chat, Some(marker), &message, || None),
                "\x04Body"
            );
            assert_eq!(
                replace(2, Console, Some(marker), &message, || panic!(
                    "console does not draw"
                )),
                "Body"
            );
        }
        assert_eq!(replace(2, Chat, None, "Unbranded", || None), "Unbranded");
    }

    #[test]
    fn shipped_marker_is_removed_when_provider_is_absent() {
        let config = crate::playlist::Config::parse(include_str!(
            "../configs/imomoe_sounds/sounds.example.jsonc"
        ))
        .unwrap();
        assert_eq!(
            for_reply(2, Chat, config.chat_prefix_marker.as_deref(), "Body"),
            "Body"
        );
    }

    #[test]
    fn invalid_provider_values_are_treated_as_absent() {
        let message = "[marker]  body";
        for invalid in ["", " \x01 ", "[missing padding]", " valid\x11\x01 "] {
            assert_eq!(
                replace(0, Chat, Some("[marker]"), message, || Some(invalid.into())),
                " body"
            );
        }
        let oversized = format!(" {}\x01 ", "x".repeat(96));
        assert_eq!(
            replace(0, Chat, Some("[marker]"), message, || Some(oversized)),
            " body"
        );
    }
}
