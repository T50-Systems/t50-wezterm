use super::*;

pub(super) fn from_char_part11(c: u32) -> Option<BlockKey> {
    Some(match c {
        // [] Branch drawing outline circle connected to left, right, up, and down
        0xF60D => BlockKey::Branches(
            Branch::CIRCLE_OUTLINE | Branch::LEFT | Branch::RIGHT | Branch::UP | Branch::DOWN,
        ),
        _ => return None,
    })
}
