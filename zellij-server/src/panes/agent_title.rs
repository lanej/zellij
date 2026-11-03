pub(super) fn compact_codex_title(title: &str) -> Option<String> {
    let (blocked, title) = match title.strip_prefix("[ ! ] Action Required | ") {
        Some(title) => (true, title),
        None => (false, title),
    };
    let mut fields = title.split(" | ");
    if fields.next()? != "codex" {
        return None;
    }
    let thread = fields.next()?;
    let (short_id, _) = thread.split_once('-')?;
    if short_id.len() != 8 || !short_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let status = fields.next()?;
    let mut glyph = if blocked {
        ""
    } else {
        match status {
            "Working" | "Thinking" => "",
            "Ready" => "",
            "Idle" | "Interrupted" => "",
            "Blocked" | "Waiting" => "",
            "Error" | "Failed" => "",
            _ => status,
        }
    };
    let context = fields.collect::<Vec<_>>().join(" | ");
    let context = if !blocked && matches!(status, "Working" | "Thinking") {
        match context.rsplit_once(' ') {
            Some((context, spinner))
                if spinner.chars().count() == 1
                    && spinner.chars().all(|ch| {
                        matches!(
                            ch,
                            '⠋' | '⠙' | '⠹' | '⠸' | '⠼' | '⠴' | '⠦' | '⠧' | '⠇' | '⠏'
                        )
                    }) =>
            {
                glyph = spinner;
                context
            },
            _ => &context,
        }
    } else {
        &context
    };
    let compact = format!("󰚩 {short_id} {glyph}");
    Some(if context.is_empty() {
        compact
    } else {
        format!("{compact} | {context}")
    })
}
