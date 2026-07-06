include!("shapecache/types.rs");

#[cfg(test)]
mod test {
    include!("shapecache/test_support.rs");
    include!("shapecache/ligatures_fira.rs");
    include!("shapecache/bench_shaping.rs");
    include!("shapecache/ligatures_jetbrains.rs");
}
