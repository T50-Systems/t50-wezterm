#[inline(always)]
fn lookup(state: State, b: u8) -> (Action, State) {
    let v = unsafe {
        TRANSITIONS
            .get_unchecked(state as usize)
            .get_unchecked(b as usize)
    };
    (Action::from_u16(v >> 8), State::from_u16(v & 0xff))
}

#[inline(always)]
#[cfg(not(test))]
fn lookup_entry(state: State) -> Action {
    unsafe { *ENTRY.get_unchecked(state as usize) }
}

#[inline(always)]
#[cfg(test)]
fn lookup_entry(state: State) -> Action {
    *ENTRY
        .get(state as usize)
        .unwrap_or_else(|| panic!("State {:?} has no entry in ENTRY", state))
}

#[inline(always)]
#[cfg(test)]
fn lookup_exit(state: State) -> Action {
    *EXIT
        .get(state as usize)
        .unwrap_or_else(|| panic!("State {:?} has no entry in EXIT", state))
}

#[inline(always)]
#[cfg(not(test))]
fn lookup_exit(state: State) -> Action {
    unsafe { *EXIT.get_unchecked(state as usize) }
}
