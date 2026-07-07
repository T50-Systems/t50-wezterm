#[test]
fn hostname_expansion() {
    let mut config = Config::new();

    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    config.assign_environment(fake_env);

    config.add_config_string(
        r#"
        Host foo0 foo1 foo2
            HostName server-%h
            "#,
    );

    let opts = config.for_host("foo0");
    snapshot!(
        opts,
        r#"
{
    "hostname": "server-foo0",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );

    let opts = config.for_host("foo1");
    snapshot!(
        opts,
        r#"
{
    "hostname": "server-foo1",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );

    let opts = config.for_host("foo2");
    snapshot!(
        opts,
        r#"
{
    "hostname": "server-foo2",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}

#[test]
fn parse_proxy_command_hostname_expansion() {
    let mut config = Config::new();

    let mut fake_env = ConfigMap::new();
    fake_env.insert("HOME".to_string(), "/home/me".to_string());
    fake_env.insert("USER".to_string(), "me".to_string());
    config.assign_environment(fake_env);

    config.add_config_string(
        r#"
        Host foo
            HostName server-%h
            ProxyCommand nc -x localhost:1080 %h %p
            "#,
    );

    let opts = config.for_host("foo");
    snapshot!(
        opts,
        r#"
{
    "hostname": "server-foo",
    "identityfile": "/home/me/.ssh/id_dsa /home/me/.ssh/id_ecdsa /home/me/.ssh/id_ed25519 /home/me/.ssh/id_rsa",
    "port": "22",
    "proxycommand": "nc -x localhost:1080 server-foo 22",
    "user": "me",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}

#[test]
fn multiple_identityfile() {
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
            IdentityFile "~/.ssh/id_pub.dsa"
            IdentityFile "~/.ssh/id_pub.rsa"
            "#,
    );

    let opts = config.for_host("foo");
    snapshot!(
        opts,
        r#"
{
    "hostname": "10.0.0.1",
    "identityfile": "/home/me/.ssh/id_pub.dsa /home/me/.ssh/id_pub.rsa",
    "port": "22",
    "user": "foo",
    "userknownhostsfile": "/home/me/.ssh/known_hosts /home/me/.ssh/known_hosts2",
}
"#
    );
}

#[test]
fn sub_tilde() {
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
            IdentityFile "~/.ssh/id_pub.dsa"
            "#,
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
