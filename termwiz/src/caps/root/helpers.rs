/// Returns true if the version string `a` is >= `b`
fn version_ge(a: &str, b: &str) -> bool {
    let mut a = a.split('.');
    let mut b = b.split('.');

    loop {
        match (a.next(), b.next()) {
            (Some(a), Some(b)) => match (a.parse::<u64>(), b.parse::<u64>()) {
                (Ok(a), Ok(b)) => {
                    if a > b {
                        return true;
                    }
                    if a < b {
                        return false;
                    }
                }
                _ => {
                    if a > b {
                        return true;
                    }
                    if a < b {
                        return false;
                    }
                }
            },
            (Some(_), None) => {
                // A is greater
                return true;
            }
            (None, Some(_)) => {
                // A is smaller
                return false;
            }
            (None, None) => {
                // Equal
                return true;
            }
        }
    }
}
