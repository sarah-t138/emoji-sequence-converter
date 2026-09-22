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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_codepoints_single_char() {
        assert_eq!(to_codepoints("A"), "U+0041");
    }

    #[test]
    fn to_codepoints_family_sequence() {
        // man, ZWJ, woman, ZWJ, girl, ZWJ, boy
        assert_eq!(
            to_codepoints("\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}"),
            "U+1F468 U+200D U+1F469 U+200D U+1F467 U+200D U+1F466"
        );
    }

    #[test]
    fn to_codepoints_trims_surrounding_whitespace() {
        assert_eq!(to_codepoints("  A  "), "U+0041");
    }

    #[test]
    fn to_codepoints_empty_line_is_empty() {
        assert_eq!(to_codepoints(""), "");
        assert_eq!(to_codepoints("   "), "");
    }

    #[test]
    fn to_glyphs_accepts_bare_hex() {
        assert_eq!(to_glyphs("1F600").unwrap(), "\u{1F600}");
    }

    #[test]
    fn to_glyphs_accepts_upper_and_lower_u_prefix() {
        assert_eq!(to_glyphs("U+1F600").unwrap(), "\u{1F600}");
        assert_eq!(to_glyphs("u+1F600").unwrap(), "\u{1F600}");
    }

    #[test]
    fn to_glyphs_joins_sequence_with_no_separator() {
        assert_eq!(
            to_glyphs("1F468 200D 1F469 200D 1F467 200D 1F466").unwrap(),
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}"
        );
    }

    #[test]
    fn to_glyphs_collapses_extra_whitespace_between_tokens() {
        assert_eq!(to_glyphs("1F468   200D  1F469").unwrap(), to_glyphs("1F468 200D 1F469").unwrap());
    }

    #[test]
    fn to_glyphs_empty_line_is_empty() {
        assert_eq!(to_glyphs("").unwrap(), "");
        assert_eq!(to_glyphs("   ").unwrap(), "");
    }

    #[test]
    fn to_glyphs_rejects_invalid_hex() {
        let err = to_glyphs("ZZZZ").unwrap_err();
        assert!(err.contains("ZZZZ"));
    }

    #[test]
    fn to_glyphs_rejects_surrogate_codepoints() {
        // D800-DFFF are reserved for UTF-16 surrogates and are not scalar values.
        assert!(to_glyphs("D800").is_err());
    }

    #[test]
    fn to_glyphs_rejects_out_of_range_codepoints() {
        assert!(to_glyphs("110000").is_err());
    }

    #[test]
    fn round_trip_glyphs_to_codepoints_to_glyphs() {
        let original = "\u{1F1EF}\u{1F1F5}";
        let codepoints = to_codepoints(original);
        let back = to_glyphs(&codepoints).unwrap();
        assert_eq!(original, back);
    }
}
