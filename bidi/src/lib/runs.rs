#[derive(Debug)]
struct Run {
    /// char indices for start, end of run
    start: usize,
    end: usize,
    /// length of run
    len: usize,
    /// isolating run sequence id
    seq_id: usize,
    /// Embedding level of this run
    level: Level,
    /// Direction of start of run
    sor: BidiClass,
    /// Direction of end of run
    eor: BidiClass,
}

impl Run {
    fn first_significant_bidi_class(
        &self,
        types: &[BidiClass],
        levels: &[Level],
    ) -> Option<BidiClass> {
        for idx in self.start..self.end {
            if !levels[idx].removed_by_x9() {
                return types.get(idx).cloned();
            }
        }
        None
    }
}

#[derive(Debug)]
struct IsolatingRunSequence {
    /// List of the runs in this sequence. The values are indices
    /// into the runs array
    runs: Vec<usize>,
    /// length of the run
    len: usize,
    /// Embedding level of this run
    level: Level,
    /// Direction of start of run
    sos: BidiClass,
    /// Direction of end of run
    eos: BidiClass,
    /// The sequence of indices into the original paragraph,
    /// across the contained set of runs
    indices: Vec<usize>,
}

/// Starting from `start`, extract the first run containing characters
/// all with the same level
fn span_one_run(types: &[BidiClass], levels: &[Level], start: usize) -> (Level, usize) {
    let mut span_level = Level(NO_LEVEL);
    let mut isolate_init_found = false;
    let mut span_len = 0;

    trace!(
        "span_one_run called with types: {:?}, levels: {:?}, start={}",
        types,
        levels,
        start
    );

    for (idx, (bc, level)) in types
        .iter()
        .skip(start)
        .zip(levels.iter().skip(start))
        .enumerate()
    {
        trace!(
            "span_one_run: consider idx={} bc={:?} level={:?}",
            idx,
            bc,
            level
        );
        if !level.removed_by_x9() {
            if bc.is_iso_init() {
                isolate_init_found = true;
            }
            if span_level.removed_by_x9() {
                span_level = *level;
            } else if *level != span_level {
                // End of run
                break;
            }
        }
        span_len = idx;
        if isolate_init_found {
            break;
        }
    }

    (span_level, span_len + 1)
}

/// 3.3.1 Paragraph level.
/// We've been fed a single paragraph, which takes care of rule P1.
/// This function implements rules P2 and P3.
fn paragraph_level(types: &[BidiClass], respect_pdi: bool, fallback: Direction) -> Level {
    let mut isolate_count = 0;
    for &t in types {
        match t {
            BidiClass::RightToLeftIsolate
            | BidiClass::LeftToRightIsolate
            | BidiClass::FirstStrongIsolate => isolate_count += 1,
            BidiClass::PopDirectionalIsolate => {
                if isolate_count > 0 {
                    isolate_count -= 1;
                } else if respect_pdi {
                    break;
                }
            }
            BidiClass::LeftToRight if isolate_count == 0 => return Level(0),
            BidiClass::RightToLeft | BidiClass::ArabicLetter if isolate_count == 0 => {
                return Level(1)
            }
            _ => {}
        }
    }
    if fallback == Direction::LeftToRight {
        Level(0)
    } else {
        Level(1)
    }
}
