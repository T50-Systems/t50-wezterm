#[test]
fn parse_match() {
    let mut config = Config::new();

    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    config.assign_environment(fake_env);

    config.add_config_string(
        r#"
        # I am a comment
        Something first
        # the prior Something takes precedence
        Something ignored
        Match Host 192.168.1.8,wopr
            FowardAgent yes
            IdentityFile "%d/.ssh/id_pub.dsa"

        Match Host !a.b,*.b User fred
            ForwardAgent no
            IdentityAgent "${HOME}/.ssh/agent"

        Match Host !a.b,*.b User me
            ForwardAgent no
            IdentityAgent "${HOME}/.ssh/agent-me"

        Host *
            Something  else
            "#,
    );

    snapshot!(
        &config,
        r#"
Config {
    config_files: [
        ParsedConfigFile {
            options: {
                "something": "first",
            },
            groups: [
                MatchGroup {
                    criteria: [
                        Host(
                            [
                                Pattern {
                                    negated: false,
                                    pattern: "^192\\.168\\.1\\.8$",
                                    original: "192.168.1.8",
                                    is_literal: true,
                                },
                                Pattern {
                                    negated: false,
                                    pattern: "^wopr$",
                                    original: "wopr",
                                    is_literal: true,
                                },
                            ],
                        ),
                    ],
                    context: FirstPass,
                    options: {
                        "fowardagent": "yes",
                        "identityfile": "%d/.ssh/id_pub.dsa",
                    },
                },
                MatchGroup {
                    criteria: [
                        Host(
                            [
                                Pattern {
                                    negated: true,
                                    pattern: "^a\\.b$",
                                    original: "a.b",
                                    is_literal: true,
                                },
                                Pattern {
                                    negated: false,
                                    pattern: "^.*\\.b$",
                                    original: "*.b",
                                    is_literal: false,
                                },
                            ],
                        ),
                        User(
                            [
                                Pattern {
                                    negated: false,
                                    pattern: "^fred$",
                                    original: "fred",
                                    is_literal: true,
                                },
                            ],
                        ),
                    ],
                    context: FirstPass,
                    options: {
                        "forwardagent": "no",
                        "identityagent": "${HOME}/.ssh/agent",
                    },
                },
                MatchGroup {
                    criteria: [
                        Host(
                            [
                                Pattern {
                                    negated: true,
                                    pattern: "^a\\.b$",
                                    original: "a.b",
                                    is_literal: true,
                                },
                                Pattern {
                                    negated: false,
                                    pattern: "^.*\\.b$",
                                    original: "*.b",
                                    is_literal: false,
                                },
                            ],
                        ),
                        User(
                            [
                                Pattern {
                                    negated: false,
                                    pattern: "^me$",
                                    original: "me",
                                    is_literal: true,
                                },
                            ],
                        ),
                    ],
                    context: FirstPass,
                    options: {
                        "forwardagent": "no",
                        "identityagent": "${HOME}/.ssh/agent-me",
                    },
                },
                MatchGroup {
                    criteria: [
                        Host(
                            [
                                Pattern {
                                    negated: false,
                                    pattern: "^.*$",
                                    original: "*",
                                    is_literal: false,
                                },
                            ],
                        ),
                    ],
                    context: FirstPass,
                    options: {
                        "something": "else",
                    },
                },
            ],
            loaded_files: [],
        },
    ],
    options: {},
    tokens: {},
    environment: Some(
        {
            "HOME": "/home/me",
            "USER": "me",
        },
    ),
}
"#
    );

    snapshot!(
        config.enumerate_hosts(),
        r#"
[
    "192.168.1.8",
    "wopr",
]
"#
    );

    let opts = config.for_host("random");
    snapshot!(
        opts,
        r#"
{
    "hostname": "random",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "something": "first",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );

    let opts = config.for_host("192.168.1.8");
    snapshot!(
        opts,
        r#"
{
    "fowardagent": "yes",
    "hostname": "192.168.1.8",
    "identityfile": "/home/me/.ssh/id_pub.dsa",
    "port": "22",
    "something": "first",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );

    let opts = config.for_host("a.b");
    snapshot!(
        opts,
        r#"
{
    "hostname": "a.b",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "something": "first",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );

    let opts = config.for_host("b.b");
    snapshot!(
        opts,
        r#"
{
    "forwardagent": "no",
    "hostname": "b.b",
    "identityagent": "/home/me/.ssh/agent-me",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "something": "first",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );

    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/fred".to_string());
    fake_env.insert("USER".to_string(), "fred".to_string());
    config.assign_environment(fake_env);

    let opts = config.for_host("b.b");
    snapshot!(
        opts,
        r#"
{
    "forwardagent": "no",
    "hostname": "b.b",
    "identityagent": "/home/fred/.ssh/agent",
    "identityfile": "/home/fred/.ssh/id_dsa /home/fred/.ssh/id_ecdsa /home/fred/.ssh/id_ed25519 /home/fred/.ssh/id_rsa",
    "port": "22",
    "something": "first",
    "user": "fred",
    "userknownhostsfile": "/home/fred/.ssh/known_hosts /home/fred/.ssh/known_hosts2",
}
"#
    );
}
