#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn metadata_should_return_metadata_about_a_file(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");
        file.touch().unwrap();

        let metadata = session
            .sftp()
            .metadata(file.path().to_path_buf())
            .await
            .expect("Failed to get metadata for file");

        // Verify that file metadata makes sense
        assert!(metadata.is_file(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn metadata_should_return_metadata_about_a_directory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();

        let metadata = session
            .sftp()
            .metadata(dir.path().to_path_buf())
            .await
            .expect("Failed to get metadata for dir");

        // Verify that file metadata makes sense
        assert!(metadata.is_dir(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn metadata_should_return_metadata_about_the_file_pointed_to_by_a_symlink(
    #[future] session: SessionWithSshd,
) {
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

        let metadata = session
            .sftp()
            .metadata(link.path().to_path_buf())
            .await
            .expect("Failed to get metadata for symlink");

        // Verify that file metadata makes sense
        assert!(metadata.is_file(), "Invalid file metadata returned");
        assert!(metadata.ty.is_file(), "Invalid file metadata returned");
        assert!(!metadata.ty.is_symlink(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn metadata_should_return_metadata_about_the_dir_pointed_to_by_a_symlink(
    #[future] session: SessionWithSshd,
) {
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

        let metadata = session
            .sftp()
            .metadata(link.path().to_path_buf())
            .await
            .expect("Failed to get metadata for symlink");

        // Verify that file metadata makes sense
        assert!(metadata.is_dir(), "Invalid file metadata returned");
        assert!(metadata.ty.is_dir(), "Invalid file metadata returned");
        assert!(!metadata.ty.is_symlink(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn metadata_should_fail_if_path_missing(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        let result = session
            .sftp()
            .metadata(temp.child("missing").path().to_path_buf())
            .await;
        assert!(
            result.is_err(),
            "Metadata unexpectedly succeeded: {:?}",
            result
        );
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_metadata_should_return_metadata_about_a_file(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let file = temp.child("file");
        file.touch().unwrap();

        let symlink_metadata = session
            .sftp()
            .symlink_metadata(file.path().to_path_buf())
            .await
            .expect("Failed to get metadata for file");

        // Verify that file metadata makes sense
        assert!(symlink_metadata.is_file(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_metadata_should_return_metadata_about_a_directory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();

        let symlink_metadata = session
            .sftp()
            .symlink_metadata(dir.path().to_path_buf())
            .await
            .expect("Failed to metadata for dir");

        // Verify that file metadata makes sense
        assert!(symlink_metadata.is_dir(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_metadata_should_return_metadata_about_symlink_pointing_to_a_file(
    #[future] session: SessionWithSshd,
) {
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

        let metadata = session
            .sftp()
            .symlink_metadata(link.path().to_path_buf())
            .await
            .expect("Failed to get metadata for symlink");

        // Verify that file metadata makes sense
        assert!(!metadata.is_file(), "Invalid file metadata returned");
        assert!(!metadata.ty.is_file(), "Invalid file metadata returned");
        assert!(metadata.ty.is_symlink(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_metadata_should_return_metadata_about_symlink_pointing_to_a_directory(
    #[future] session: SessionWithSshd,
) {
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

        let metadata = session
            .sftp()
            .symlink_metadata(link.path().to_path_buf())
            .await
            .expect("Failed to get metadata for symlink");

        // Verify that file metadata makes sense
        assert!(!metadata.is_dir(), "Invalid file metadata returned");
        assert!(!metadata.ty.is_dir(), "Invalid file metadata returned");
        assert!(metadata.ty.is_symlink(), "Invalid file metadata returned");
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn symlink_metadata_should_fail_if_path_missing(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        let result = session
            .sftp()
            .symlink_metadata(temp.child("missing").path().to_path_buf())
            .await;
        assert!(
            result.is_err(),
            "symlink_metadata unexpectedly succeeded: {:?}",
            result
        );
    })
}
