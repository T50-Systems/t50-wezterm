#[test]
fn parse_keepalive() {
    let mut config = Config::new();
    config.add_config_string(
        r#"
        Host foo
            ServerAliveInterval 60
            "#,
    );
    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    config.assign_environment(fake_env);

    let opts = config.for_host("foo");
    snapshot!(
        opts,
        r#"
{
    "hostname": "foo",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "serveraliveinterval": "60",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}

#[test]
fn parse_proxy_command_tokens() {
    let mut config = Config::new();
    config.add_config_string(
        r#"
        Host foo
            ProxyCommand /usr/bin/corp-ssh-helper -dst_username=%r %h %p
            Port 2222
            "#,
    );
    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    config.assign_environment(fake_env);

    let opts = config.for_host("foo");
    snapshot!(
        opts,
        r#"
{
    "hostname": "foo",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "2222",
    "proxycommand": "/usr/bin/corp-ssh-helper -dst_username=me foo 2222",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}

#[test]
fn parse_proxy_command() {
    let mut config = Config::new();
    config.add_config_string(
        r#"
        Host foo
            ProxyCommand /usr/bin/ssh-proxy-helper -oX=Y host 22
            "#,
    );

    snapshot!(
        &config,
        r#"
Config {
    config_files: [
        ParsedConfigFile {
            options: {},
            groups: [
                MatchGroup {
                    criteria: [
                        Host(
                            [
                                Pattern {
                                    negated: false,
                                    pattern: "^foo$",
                                    original: "foo",
                                    is_literal: true,
                                },
                            ],
                        ),
                    ],
                    context: FirstPass,
                    options: {
                        "proxycommand": "/usr/bin/ssh-proxy-helper -oX=Y host 22",
                    },
                },
            ],
            loaded_files: [],
        },
    ],
    options: {},
    tokens: {},
    environment: None,
}
"#
    );
}

#[test]
fn misc_tokens() {
    let mut config = Config::new();

    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    fake_env.insert("WEZTERM_SSH_UID".to_string(), "1000".to_string());
    config.assign_environment(fake_env);

    config.add_config_string(
        r#"
        Host target-host
            LocalCommand C=%C d=%d h=%h i=%i L=%L l=%L n=%n p=%p r=%r T=%T u=%u
            "#,
    );

    let opts = config.for_host("target-host");
    snapshot!(
        opts,
        r#"
{
    "hostname": "target-host",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "localcommand": "C=8de28522efb92214d9c442ea0402863e34d095a4006467ad9136a48e930870ea d=/home/me h=target-host i=1000 L=localhost l=localhost n=target-host p=22 r=me T=NONE u=me",
    "port": "22",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}

#[test]
fn parse_user() {
    let mut config = Config::new();

    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    config.assign_environment(fake_env);

    config.add_config_string(
        r#"
        Host foo
            HostName 10.0.0.1
            User foo
            IdentityFile "%d/.ssh/id_pub.dsa"
            "#,
    );

    snapshot!(
        &config,
        r#"
Config {
    config_files: [
        ParsedConfigFile {
            options: {},
            groups: [
                MatchGroup {
                    criteria: [
                        Host(
                            [
                                Pattern {
                                    negated: false,
                                    pattern: "^foo$",
                                    original: "foo",
                                    is_literal: true,
                                },
                            ],
                        ),
                    ],
                    context: FirstPass,
                    options: {
                        "hostname": "10.0.0.1",
                        "identityfile": "%d/.ssh/id_pub.dsa",
                        "user": "foo",
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

    let opts = config.for_host("foo");
    snapshot!(
        opts,
        r#"
{
    "hostname": "10.0.0.1",
    "identityfile": "/home/me/.ssh/id_pub.dsa",
    "port": "22",
    "user": "foo",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}
