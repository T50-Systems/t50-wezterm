#[derive(Debug)]
struct ClusterInfo {
    start: usize,
    byte_len: usize,
    cell_width: u8,
    incomplete: bool,
}

#[derive(Default, Debug)]
struct ClusterResolver<'a> {
    map: HashMap<usize, ClusterInfo>,
    presentation_width: Option<&'a PresentationWidth<'a>>,
    start_by_cell_idx: HashMap<usize, usize>,
}

impl<'a> ClusterResolver<'a> {
    pub fn build(&mut self, hb_infos: &[harfbuzz::hb_glyph_info_t], s: &str, range: &Range<usize>) {
        #[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone)]
        struct Item {
            cell_idx: Option<usize>,
            start: usize,
        }

        let mut map = HashMap::new();

        for info in hb_infos.iter() {
            let start = info.cluster as usize;

            // Collect the cell index, which is the "true cluster"
            // position from our perspective: the start of the grapheme
            let cell_idx = match self.presentation_width {
                Some(pw) => {
                    let cell_idx = pw.byte_to_cell_idx(start);

                    let entry = self.start_by_cell_idx.entry(cell_idx).or_insert(start);
                    *entry = (*entry).min(start);

                    Some(cell_idx)
                }
                None => None,
            };

            map.entry(start).or_insert_with(|| Item { start, cell_idx });
        }

        let mut cluster_starts: Vec<Item> = map.into_values().collect();
        cluster_starts.sort();

        // If we have multiple entries with the same starting cell index,
        // remove the duplicates.  Don't do this for `None` as that will
        // falsely remove valid cluster locations in the case where
        // we have no presentation_width information.
        cluster_starts.dedup_by(|a, b| match (a.cell_idx, b.cell_idx) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        });

        let mut iter = cluster_starts.iter().peekable();
        while let Some(item) = iter.next().copied() {
            let start = item.start;
            let next_start = iter.peek().map(|&&s| s.start).unwrap_or(range.end);
            let byte_len = next_start - start;
            let cell_width = match self.presentation_width {
                Some(p) => p.num_cells(start..next_start),
                None => unicode_column_width(&s[start..next_start], None) as u8,
            };
            self.map.entry(start).or_insert_with(|| ClusterInfo {
                start,
                byte_len,
                cell_width,
                incomplete: false,
            });
        }
    }

    pub fn get_mut(&mut self, start: usize) -> Option<&mut ClusterInfo> {
        match self.presentation_width {
            Some(pw) => {
                let cell_idx = pw.byte_to_cell_idx(start);
                let actual_start = self.start_by_cell_idx.get(&cell_idx)?;
                self.map.get_mut(&actual_start)
            }
            None => self.map.get_mut(&start),
        }
    }

    pub fn get(&self, start: usize) -> Option<&ClusterInfo> {
        match self.presentation_width {
            Some(pw) => {
                let cell_idx = pw.byte_to_cell_idx(start);
                let actual_start = self.start_by_cell_idx.get(&cell_idx)?;
                self.map.get(&actual_start)
            }
            None => self.map.get(&start),
        }
    }
}
