#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_should_create_symlink_pointing_to_file(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");
        file.touch().unwrap();

        let link = temp.child("link");

        session
            .sftp()
            .symlink(file.path().to_path_buf(), link.path().to_path_buf())
            .await
            .expect("Failed to create symlink");

        assert!(
            std::fs::symlink_metadata(link.path())
                .unwrap()
                .file_type()
                .is_symlink(),
            "Symlink is not a symlink!"
        );

        // TODO: This fails even though the type is a symlink:
        //       https://github.com/assert-rs/assert_fs/issues/70
        // link.assert(predicate::path::is_symlink());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_should_create_symlink_pointing_to_directory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();

        let link = temp.child("link");

        session
            .sftp()
            .symlink(dir.path().to_path_buf(), link.path().to_path_buf())
            .await
            .expect("Failed to create symlink");

        link.assert(predicate::path::is_symlink());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_should_succeed_even_if_path_missing(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");

        let link = temp.child("link");

        session
            .sftp()
            .symlink(file.path().to_path_buf(), link.path().to_path_buf())
            .await
            .expect("Failed to create symlink");

        link.assert(predicate::path::is_symlink());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn read_link_should_return_the_target_of_the_symlink(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        // Test a symlink to a directory
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();
        let link = temp.child("link");
        link.symlink_to_dir(dir.path()).unwrap();

        let path = session
            .sftp()
            .read_link(link.path().to_path_buf())
            .await
            .expect("Failed to read symlink");
        assert_eq!(path, dir.path());

        // Test a symlink to a file
        let file = temp.child("file");
        file.touch().unwrap();
        let link = temp.child("link2");
        link.symlink_to_file(file.path()).unwrap();

        let path = session
            .sftp()
            .read_link(link.path().to_path_buf())
            .await
            .expect("Failed to read symlink");
        assert_eq!(path, file.path());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn read_link_should_fail_if_path_is_not_a_symlink(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        // Test missing path
        let result = session
            .sftp()
            .read_link(temp.child("missing").path().to_path_buf())
            .await;
        assert!(
            result.is_err(),
            "Unexpectedly read link for missing path: {:?}",
            result
        );

        // Test a directory
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();
        let result = session.sftp().read_link(dir.path().to_path_buf()).await;
        assert!(
            result.is_err(),
            "Unexpectedly read link for directory: {:?}",
            result
        );

        // Test a file
        let file = temp.child("file");
        file.touch().unwrap();
        let result = session.sftp().read_link(file.path().to_path_buf()).await;
        assert!(
            result.is_err(),
            "Unexpectedly read link for file: {:?}",
            result
        );
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn canonicalize_should_resolve_absolute_path_for_relative_path(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        // For resolving parts of a path, all components must exist
        let temp = TempDir::new().unwrap();
        temp.child("hello").create_dir_all().unwrap();
        temp.child("world").touch().unwrap();

        let rel = temp.child(".").child("hello").child("..").child("world");

        // NOTE: Because sftp realpath can still resolve symlinks within a missing path, there
        //       is no guarantee that the resulting path matches the missing path. In fact,
        //       on mac the /tmp dir is a symlink to /private/tmp; so, we cannot successfully
        //       check the accuracy of the path itself, meaning that we can only validate
        //       that the operation was okay.
        let result = session.sftp().canonicalize(rel.path().to_path_buf()).await;
        assert!(
            result.is_ok(),
            "Canonicalize unexpectedly failed: {:?}",
            result
        );
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn canonicalize_should_either_return_resolved_path_or_error_if_missing(
    #[future] session: SessionWithSshd,
) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let missing = temp.child("missing");

        // NOTE: Because sftp realpath can still resolve symlinks within a missing path, there
        //       is no guarantee that the resulting path matches the missing path. In fact,
        //       on mac the /tmp dir is a symlink to /private/tmp; so, we cannot successfully
        //       check the accuracy of the path itself, meaning that we can only validate
        //       that the operation was okay.
        //
        //       Additionally, this has divergent behavior. On some platforms, this returns
        //       the path as is whereas on others this returns a missing path error. We
        //       have to support both checks.
        let result = session
            .sftp()
            .canonicalize(missing.path().to_path_buf())
            .await;
        match result {
            Ok(_) => {}
            Err(SftpChannelError::Sftp(SftpError::NoSuchFile)) => {}
            #[cfg(feature = "libssh-rs")]
            Err(SftpChannelError::LibSsh(libssh_rs::Error::Sftp(_))) => {}
            x => panic!(
                "Unexpected result from canonicalize({}: {:?}",
                missing.path().display(),
                x
            ),
        }
    })
}
