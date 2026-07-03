#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use k9::assert_equal as assert_eq;

    #[test]
    fn runs() {
        let text = vec!['א', 'ב', 'ג', 'a', 'b', 'c'];

        let mut context = BidiContext::new();
        context.resolve_paragraph(&text, ParagraphDirectionHint::AutoLeftToRight);
        k9::snapshot!(
            context.runs().collect::<Vec<_>>(),
            "
[
    BidiRun {
        direction: RightToLeft,
        level: Level(
            1,
        ),
        range: 0..3,
        removed_by_x9: [],
    },
    BidiRun {
        direction: LeftToRight,
        level: Level(
            2,
        ),
        range: 3..6,
        removed_by_x9: [],
    },
]
"
        );
    }

    #[test]
    fn mirror() {
        assert_eq!(lookup_closing('{'), Some(('}', BracketType::Open)));
        assert_eq!(lookup_closing('['), Some((']', BracketType::Open)));
        assert_eq!(lookup_closing(']'), Some(('[', BracketType::Close)));
    }

    #[test]
    fn bidi_class_resolve() {
        assert_eq!(bidi_class_for_char('\u{0}'), BidiClass::BoundaryNeutral);
        assert_eq!(bidi_class_for_char('\u{9}'), BidiClass::SegmentSeparator);
        assert_eq!(bidi_class_for_char(' '), BidiClass::WhiteSpace);
        assert_eq!(bidi_class_for_char('a'), BidiClass::LeftToRight);
        assert_eq!(bidi_class_for_char('\u{590}'), BidiClass::RightToLeft);
        assert_eq!(bidi_class_for_char('\u{5d0}'), BidiClass::RightToLeft);
        assert_eq!(bidi_class_for_char('\u{5d1}'), BidiClass::RightToLeft);
    }

    /// This example is taken from
    /// <https://terminal-wg.pages.freedesktop.org/bidi/recommendation/combining.html>
    #[test]
    fn reorder_nsm() {
        let shalom: Vec<char> = vec![
            '\u{5e9}', '\u{5b8}', '\u{5c1}', '\u{5dc}', '\u{5d5}', '\u{05b9}', '\u{5dd}',
        ];
        let mut context = BidiContext::new();
        context.set_reorder_non_spacing_marks(true);
        context.resolve_paragraph(&shalom, ParagraphDirectionHint::LeftToRight);

        let mut reordered = vec![];
        for run in context.reordered_runs(0..shalom.len()) {
            for idx in run.indices {
                reordered.push(shalom[idx]);
            }
        }

        let explicit_ltr = vec![
            '\u{5dd}', '\u{5d5}', '\u{5b9}', '\u{5dc}', '\u{5e9}', '\u{5b8}', '\u{5c1}',
        ];
        assert_eq!(reordered, explicit_ltr);
    }
}
