# emoji-sequence-converter

Unicode defines most emoji as sequences of codepoints, not single characters.
A "family" emoji is really five codepoints - man, zero-width joiner, woman,
zero-width joiner, child - that a font renders as one glyph. Unicode's own
data files (`emoji-test.txt`, `emoji-sequences.txt`) and most bug reports
write these sequences as space-separated hex, like `1F468 200D 1F469 200D
1F467`, because plain text tools don't render every sequence reliably and
because pasting the rendered glyph loses information: two emoji that look
identical on screen can differ by a hidden variation selector or joiner.

`emojiconv` converts between the two representations, one sequence per line,
so you can turn a codepoint list from a spec into something you can actually
see, or take an emoji you don't trust and find out exactly what it's made of.

## Usage

Build with cargo (standard library only, nothing to download):

    cargo build --release

Convert codepoints to glyphs:

    $ echo "1F468 200D 1F469 200D 1F467 200D 1F466" | ./target/release/emojiconv --to-glyphs
    👨‍👩‍👧‍👦

Convert a glyph back to its codepoints:

    $ echo "👨‍👩‍👧‍👦" | ./target/release/emojiconv --to-codepoints
    U+1F468 U+200D U+1F469 U+200D U+1F467 U+200D U+1F466

Works on files too, one sequence per line:

    $ cat sequences.txt
    1F1EF 1F1F5
    0031 FE0F 20E3
    $ ./target/release/emojiconv --to-glyphs sequences.txt
    🇯🇵
    1️⃣

The `U+` prefix on input codepoints is optional - `1F600` and `U+1F600` are
both accepted. Output always uses the `U+` form.

## Format

- Codepoint format: hex codepoints separated by spaces, one full sequence
  per line.
- Glyph format: the actual emoji characters, one sequence per line.

Lines that fail to parse are reported to stderr with their line number;
the rest of the file is still processed.

## Status

Early. It reads and writes the two formats above and nothing more.

## Roadmap

- validate against the official Unicode emoji-sequences.txt data
- reject codepoint sequences that parse fine but aren't actually emoji
- optional CLDR short-name output (`:family_man_woman_girl_boy:`)
- batch mode that annotates each line with its sequence name
