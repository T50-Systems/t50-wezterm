#[test]
fn parse_simple() {
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
        Host 192.168.1.8 wopr
            FowardAgent yes
            IdentityFile "%d/.ssh/id_pub.dsa"

        Host !a.b *.b
            ForwardAgent no
            IdentityAgent "${HOME}/.ssh/agent"

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
    "identityagent": "/home/me/.ssh/agent",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "something": "first",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}
