fn encode(osc: &OperatingSystemCommand) -> String {
    format!("{}", osc)
}

fn parse(osc: &[&str], expected: &str) -> OperatingSystemCommand {
    let mut v = Vec::new();
    for s in osc {
        v.push(s.as_bytes());
    }
    let result = OperatingSystemCommand::parse(&v);

    assert_eq!(encode(&result), expected);

    result
}
