fn build_aliases() -> Vec<Alias> {
    let mut aliases = vec![];
    let start = std::time::Instant::now();

    fn push(aliases: &mut Vec<Alias>, alias: Alias) {
        aliases.push(alias);
    }

    if let Ok(recents) = load_recents() {
        for r in recents {
            let character = if let Some(emoji) = emojis::get(&r.glyph) {
                Character::Emoji(emoji)
            } else {
                Character::Unicode {
                    name: "",
                    value: r.glyph.chars().next().unwrap(),
                }
            };

            aliases.push(Alias {
                name: Cow::Owned(r.name.clone()),
                character,
                group: CharSelectGroup::RecentlyUsed,
            });
        }
    }

    for emoji in emojis::iter() {
        let group = match emoji.group() {
            Group::SmileysAndEmotion => CharSelectGroup::SmileysAndEmotion,
            Group::PeopleAndBody => CharSelectGroup::PeopleAndBody,
            Group::AnimalsAndNature => CharSelectGroup::AnimalsAndNature,
            Group::FoodAndDrink => CharSelectGroup::FoodAndDrink,
            Group::TravelAndPlaces => CharSelectGroup::TravelAndPlaces,
            Group::Activities => CharSelectGroup::Activities,
            Group::Objects => CharSelectGroup::Objects,
            Group::Symbols => CharSelectGroup::Symbols,
            Group::Flags => CharSelectGroup::Flags,
        };
        match emoji.skin_tones() {
            Some(iter) => {
                for entry in iter {
                    push(
                        &mut aliases,
                        Alias {
                            name: Cow::Borrowed(entry.name()),
                            character: Character::Emoji(entry),
                            group,
                        },
                    );
                }
            }
            None => {
                push(
                    &mut aliases,
                    Alias {
                        name: Cow::Borrowed(emoji.name()),
                        character: Character::Emoji(emoji),
                        group,
                    },
                );
            }
        }
        for short in emoji.shortcodes() {
            push(
                &mut aliases,
                Alias {
                    name: Cow::Borrowed(short),
                    character: Character::Emoji(emoji),
                    group: CharSelectGroup::ShortCodes,
                },
            );
        }
    }

    for (name, value) in crate::unicode_names::NAMES {
        push(
            &mut aliases,
            Alias {
                name: Cow::Borrowed(name),
                character: Character::Unicode {
                    name,
                    value: char::from_u32(*value).unwrap(),
                },
                group: CharSelectGroup::UnicodeNames,
            },
        );
    }

    for (name, value) in termwiz::nerdfonts::NERD_FONT_GLYPHS {
        push(
            &mut aliases,
            Alias {
                name: Cow::Borrowed(name),
                character: Character::Unicode {
                    name,
                    value: *value,
                },
                group: CharSelectGroup::NerdFonts,
            },
        );
    }

    log::trace!(
        "Took {:?} to build {} aliases",
        start.elapsed(),
        aliases.len()
    );

    aliases
}

#[derive(Debug, Copy, Clone)]
struct MatchResult {
    row_idx: usize,
    score: u32,
}

impl MatchResult {
    fn new(row_idx: usize, score: u32, selection: &str, aliases: &[Alias]) -> Self {
        Self {
            row_idx,
            score: if aliases[row_idx].name == selection {
                // Pump up the score for an exact match, otherwise
                // the order may be undesirable if there are a lot
                // of candidates with the same score
                u32::max_value()
            } else {
                score
            },
        }
    }
}

fn compute_matches(selection: &str, aliases: &[Alias], group: CharSelectGroup) -> Vec<usize> {
    if selection.is_empty() {
        aliases
            .iter()
            .enumerate()
            .filter(|(_idx, a)| a.group == group)
            .map(|(idx, _a)| idx)
            .collect()
    } else {
        let pattern = matcher_pattern(selection);

        let numeric_selection = if selection.chars().all(|c| c.is_ascii_hexdigit()) {
            // Make this uppercase so that eg: `e1` matches `U+E1` rather
            // than HENTAIGANA LETTER E-1.
            // <https://github.com/wezterm/wezterm/issues/2581#issuecomment-1267662040>
            Some(format!("U+{}", selection.to_ascii_uppercase()))
        } else if selection.starts_with("U+") {
            Some(selection.to_string())
        } else {
            None
        };
        let start = std::time::Instant::now();

        let all_matches: Vec<(String, MatchResult)> = aliases
            .par_iter()
            .enumerate()
            .filter_map(|(row_idx, entry)| {
                let glyph = entry.glyph();

                let alias_result = matcher_score(&pattern, &entry.name)
                    .map(|score| MatchResult::new(row_idx, score, selection, aliases));

                match &numeric_selection {
                    Some(sel) => {
                        let codepoints = entry.codepoints();
                        if codepoints == *sel {
                            Some((
                                glyph,
                                MatchResult {
                                    row_idx,
                                    score: u32::max_value(),
                                },
                            ))
                        } else {
                            let number_result = matcher_score(&pattern, &codepoints)
                                .map(|score| MatchResult::new(row_idx, score, selection, aliases));

                            match (alias_result, number_result) {
                                (
                                    Some(MatchResult { score: a, .. }),
                                    Some(MatchResult { score: b, .. }),
                                ) => Some((
                                    glyph,
                                    MatchResult {
                                        row_idx,
                                        score: a.max(b),
                                    },
                                )),
                                (Some(a), None) | (None, Some(a)) => Some((glyph, a)),
                                (None, None) => None,
                            }
                        }
                    }
                    None => alias_result.map(|a| (glyph, a)),
                }
            })
            .collect();

        let mut matches = HashMap::<String, MatchResult>::new();
        for (glyph, value) in all_matches {
            let entry = matches.entry(glyph).or_insert(value);
            // Retain the best scoring match for a given glyph
            if entry.score < value.score {
                *entry = value;
            }
        }
        let mut scores: Vec<MatchResult> = matches.into_values().collect();
        scores.sort_by(|a, b| a.score.cmp(&b.score).reverse());
        log::trace!(
            "matching took {:?} for {} entries",
            start.elapsed(),
            scores.len()
        );

        scores.iter().map(|result| result.row_idx).collect()
    }
}
