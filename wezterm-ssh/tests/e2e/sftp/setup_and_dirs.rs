use crate::sshd::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use rstest::*;
use std::convert::TryInto;
use wezterm_ssh::{FileType, SftpChannelError, SftpError, Utf8PathBuf};

// Sftp file tests
mod file;

#[inline]
fn file_type_to_str(file_type: FileType) -> &'static str {
    if file_type.is_dir() {
        "dir"
    } else if file_type.is_file() {
        "file"
    } else {
        "symlink"
    }
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn read_dir_should_return_list_of_directories_files_and_symlinks(
    #[future] session: SessionWithSshd,
) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        // $TEMP/dir1/
        // $TEMP/dir2/
        // $TEMP/file1
        // $TEMP/file2
        // $TEMP/dir-link -> $TEMP/dir1/
        // $TEMP/file-link -> $TEMP/file1
        let temp = TempDir::new().unwrap();
        let dir1 = temp.child("dir1");
        dir1.create_dir_all().unwrap();
        let dir2 = temp.child("dir2");
        dir2.create_dir_all().unwrap();
        let file1 = temp.child("file1");
        file1.touch().unwrap();
        let file2 = temp.child("file2");
        file2.touch().unwrap();
        let link_dir = temp.child("link-dir");
        link_dir.symlink_to_dir(dir1.path()).unwrap();
        let link_file = temp.child("link-file");
        link_file.symlink_to_file(file1.path()).unwrap();

        let mut contents = session
            .sftp()
            .read_dir(temp.path().to_path_buf())
            .await
            .expect("Failed to read directory")
            .into_iter()
            .map(|(p, s)| (p, file_type_to_str(s.ty)))
            .collect::<Vec<(Utf8PathBuf, &'static str)>>();
        contents.sort_unstable_by_key(|(p, _)| p.to_path_buf());

        assert_eq!(
            contents,
            vec![
                (dir1.path().to_path_buf().try_into().unwrap(), "dir"),
                (dir2.path().to_path_buf().try_into().unwrap(), "dir"),
                (file1.path().to_path_buf().try_into().unwrap(), "file"),
                (file2.path().to_path_buf().try_into().unwrap(), "file"),
                (link_dir.path().to_path_buf().try_into().unwrap(), "symlink"),
                (
                    link_file.path().to_path_buf().try_into().unwrap(),
                    "symlink"
                ),
            ]
        );
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn create_dir_should_create_a_directory_on_the_remote_filesystem(
    #[future] session: SessionWithSshd,
) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        session
            .sftp()
            .create_dir(temp.child("dir").path().to_path_buf(), 0o644)
            .await
            .expect("Failed to create directory");

        // Verify the path exists and is to a directory
        temp.child("dir").assert(predicate::path::is_dir());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn create_dir_should_return_error_if_unable_to_create_directory(
    #[future] session: SessionWithSshd,
) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        // Attempt to create a nested directory structure, which is not supported
        let result = session
            .sftp()
            .create_dir(temp.child("dir").child("dir").path().to_path_buf(), 0o644)
            .await;
        assert!(
            result.is_err(),
            "Unexpectedly succeeded in creating directory: {:?}",
            result
        );

        // Verify the path is not a directory
        temp.child("dir")
            .child("dir")
            .assert(predicate::path::is_dir().not());
        temp.child("dir").assert(predicate::path::is_dir().not());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_dir_should_remove_a_remote_directory(#[future] session: SessionWithSshd) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        // Removing an empty directory should succeed
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();
        session
            .sftp()
            .remove_dir(dir.path().to_path_buf())
            .await
            .expect("Failed to remove directory");

        // Verify the directory no longer exists
        dir.assert(predicate::path::is_dir().not());
    })
}

#[rstest]
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), ignore)]
fn remove_dir_should_return_an_error_if_failed_to_remove_directory(
    #[future] session: SessionWithSshd,
) {
    if !sshd_available() {
        return;
    }
    smol::block_on(async {
        let session: SessionWithSshd = session.await;

        let temp = TempDir::new().unwrap();

        // Attempt to remove a missing path
        let result = session
            .sftp()
            .remove_dir(temp.child("missing-dir").path().to_path_buf())
            .await;
        assert!(
            result.is_err(),
            "Unexpectedly succeeded in removing missing directory: {:?}",
            result
        );

        // Attempt to remove a non-empty directory
        let dir = temp.child("dir");
        dir.create_dir_all().unwrap();
        dir.child("file").touch().unwrap();

        let result = session.sftp().remove_dir(dir.path().to_path_buf()).await;
        assert!(
            result.is_err(),
            "Unexpectedly succeeded in removing non-empty directory: {:?}",
            result
        );

        // Verify the non-empty directory still exists
        dir.assert(predicate::path::is_dir());

        // Attempt to remove a file (not a directory)
        let file = temp.child("file");
        file.touch().unwrap();
        let result = session.sftp().remove_dir(file.path().to_path_buf()).await;
        assert!(
            result.is_err(),
            "Unexpectedly succeeded in removing file: {:?}",
            result
        );

        // Verify the file still exists
        file.assert(predicate::path::is_file());
    })
}
