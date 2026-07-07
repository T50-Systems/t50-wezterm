fn phys_intersection(r1: &Range<PhysRowIndex>, r2: &Range<PhysRowIndex>) -> Range<PhysRowIndex> {
    let start = r1.start.max(r2.start);
    let end = r1.end.min(r2.end);
    if end > start {
        start..end
    } else {
        0..0
    }
}
