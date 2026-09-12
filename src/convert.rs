// Two representations of the same thing: a full emoji sequence (which may
// be several codepoints joined by ZWJ or a variation selector) either as
// the literal characters, or as its hex codepoints written out by hand.

pub fn to_codepoints(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn to_glyphs(line: &str) -> Result<String, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    let mut out = String::new();
    for token in trimmed.split_whitespace() {
        let hex = token
            .strip_prefix("U+")
            .or_else(|| token.strip_prefix("u+"))
            .unwrap_or(token);
        let code = u32::from_str_radix(hex, 16)
            .map_err(|_| format!("'{token}' is not a valid hex codepoint"))?;
        let ch = char::from_u32(code)
            .ok_or_else(|| format!("U+{code:04X} is not a valid Unicode scalar value"))?;
        // Codepoints in a sequence combine into one rendered glyph (via ZWJ
        // or variation selectors), so they get no separator when reassembled.
        out.push(ch);
    }
    Ok(out)
}
