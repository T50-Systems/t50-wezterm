#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn canonicalize_should_fail_if_resolving_missing_path_with_dots(
    #[future] session: SessionWithSshd,
) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let missing = temp.child(".").child("hello").child("..").child("world");

        let result = session
            .sftp()
            .canonicalize(missing.path().to_path_buf())
            .await;
        assert!(result.is_err(), "Canonicalize unexpectedly succeeded");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn rename_should_support_singular_file(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");
        file.write_str("some text").unwrap();

        let dst = temp.child("dst");

        session
            .sftp()
            .rename(
                file.path().to_path_buf(),
                dst.path().to_path_buf(),
                Default::default(),
            )
            .await
            .expect("Failed to rename file");

        // Verify that file was moved to destination
        file.assert(predicate::path::missing());
        dst.assert("some text");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn rename_should_support_dirtectory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();
        let dir_file = dir.child("file");
        dir_file.write_str("some text").unwrap();
        let dir_dir = dir.child("dir");
        dir_dir.create_dir_all().unwrap();

        let dst = temp.child("dst");

        session
            .sftp()
            .rename(
                dir.path().to_path_buf(),
                dst.path().to_path_buf(),
                Default::default(),
            )
            .await
            .expect("Failed to rename directory");

        // Verify that directory was moved to destination
        dir.assert(predicate::path::missing());
        dir_file.assert(predicate::path::missing());
        dir_dir.assert(predicate::path::missing());

        dst.assert(predicate::path::is_dir());
        dst.child("file").assert("some text");
        dst.child("dir").assert(predicate::path::is_dir());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn rename_should_fail_if_source_path_missing(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let missing = temp.child("missing");
        let dst = temp.child("dst");

        let result = session
            .sftp()
            .rename(
                missing.path().to_path_buf(),
                dst.path().to_path_buf(),
                Default::default(),
            )
            .await;
        assert!(
            result.is_err(),
            "Rename unexpectedly succeeded with missing path: {:?}",
            result
        );
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_file_should_remove_file(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");
        file.touch().unwrap();

        session
            .sftp()
            .remove_file(file.path().to_path_buf())
            .await
            .expect("Failed to remove file");

        file.assert(predicate::path::missing());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_file_should_remove_symlink_to_file(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");
        file.touch().unwrap();
        let link = temp.child("link");
        link.symlink_to_file(file.path()).unwrap();

        session
            .sftp()
            .remove_file(link.path().to_path_buf())
            .await
            .expect("Failed to remove symlink");

        // Verify link removed but file still exists
        link.assert(predicate::path::missing());
        file.assert(predicate::path::is_file());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_file_should_remove_symlink_to_directory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();
        let link = temp.child("link");
        link.symlink_to_dir(dir.path()).unwrap();

        session
            .sftp()
            .remove_file(link.path().to_path_buf())
            .await
            .expect("Failed to remove symlink");

        // Verify link removed but directory still exists
        link.assert(predicate::path::missing());
        dir.assert(predicate::path::is_dir());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_file_should_fail_if_path_to_directory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();

        let result = session.sftp().remove_file(dir.path().to_path_buf()).await;
        assert!(
            result.is_err(),
            "Unexpectedly removed directory: {:?}",
            result
        );

        // Verify directory still here
        dir.assert(predicate::path::is_dir());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_file_should_fail_if_path_missing(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        let result = session
            .sftp()
            .remove_file(temp.child("missing").path().to_path_buf())
            .await;
        assert!(
            result.is_err(),
            "Unexpectedly removed missing path: {:?}",
            result
        );
    })
}
