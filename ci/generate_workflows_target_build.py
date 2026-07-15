def install_system_deps(self):
    if "win" in self.name:
        return []
    sudo = "sudo -n " if self.needs_sudo() else ""
    return [
        RunStep(
            name="Install System Deps",
            run=f"{sudo}env CI=yes PATH=$PATH ./get-deps",
        )
    ]

def fixup_windows_path(self, cmd):
    if "win" in self.name:
        return "PATH C:\\Strawberry\\perl\\bin;%PATH%\n" + cmd
    return cmd

def build_all_release(self):
    bin_crates = [
        "wezterm",
        "wezterm-gui",
        "wezterm-mux-server",
        "strip-ansi-escapes",
    ]
    steps = []
    if "win" in self.name:
        steps.append(
            RunStep(
                name="Reset release sccache statistics",
                shell="cmd",
                run="sccache --zero-stats",
            )
        )
    for bin in bin_crates:
        if "win" in self.name:
            steps += [
                RunStep(
                    name=f"Build {bin} (Release mode)",
                    shell="cmd",
                    run=self.fixup_windows_path(f"cargo build -p {bin} --release"),
                )
            ]
        elif "macos" in self.name:
            steps += [
                RunStep(
                    name=f"Build {bin} (Release mode Intel)",
                    run=f"cargo build --target x86_64-apple-darwin -p {bin} --release",
                ),
                RunStep(
                    name=f"Build {bin} (Release mode ARM)",
                    run=f"cargo build --target aarch64-apple-darwin -p {bin} --release",
                ),
            ]
        else:
            if self.name == "centos7":
                enable = "source /opt/rh/devtoolset-9/enable && "
            else:
                enable = ""
            steps += [
                RunStep(
                    name=f"Build {bin} (Release mode)",
                    run=enable + f"cargo build -p {bin} --release",
                )
            ]
    if "win" in self.name:
        steps.append(
            RunStep(
                name="Report release sccache statistics",
                shell="cmd",
                run="sccache --show-stats",
            )
        )
    return steps

def test_all(self):
    run = "cargo nextest run --all --no-fail-fast"
    if "macos" in self.name:
        run += " --target=x86_64-apple-darwin"
    if self.name == "centos7":
        run = "source /opt/rh/devtoolset-9/enable\n" + run
    return [
        # Install cargo-nextest
        InstallCrateStep("cargo-nextest", key=self.name),
        # Run tests
        RunStep(name="Test", run=self.fixup_windows_path(run), shell="cmd")
        if "win" in self.name
        else RunStep(name="Test", run=run),
    ]

def package(self, trusted=False):
    steps = []
    deploy_env = None
    if trusted and ("mac" in self.name):
        deploy_env = {
            "MACOS_CERT": "${{ secrets.MACOS_CERT }}",
            "MACOS_CERT_PW": "${{ secrets.MACOS_CERT_PW }}",
            "MACOS_TEAM_ID": "${{ secrets.MACOS_TEAM_ID }}",
            "MACOS_APPLEID": "${{ secrets.MACOS_APPLEID }}",
            "MACOS_APP_PW": "${{ secrets.MACOS_APP_PW }}",
        }
    steps = [RunStep("Package", "bash ci/deploy.sh", env=deploy_env)]
    if self.app_image:
        # AppImage needs fuse and the file command
        steps += self.install_system_package("libfuse2")
        steps += self.install_system_package("file")
        steps.append(RunStep("Source Tarball", "bash ci/source-archive.sh"))
        steps.append(RunStep("Build AppImage", "bash ci/appimage.sh"))
    return steps

def upload_artifact(self):
    steps = []

    if self.uses_yum():
        steps.append(
            RunStep(
                "Move RPM",
                f"mv ~/rpmbuild/RPMS/*/*.rpm .",
            )
        )
    elif self.uses_apk():
        steps += [
            # Add the distro name/version into the filename
            RunStep(
                "Rename APKs",
                f"mv ~/packages/wezterm/x86_64/*.apk $(echo ~/packages/wezterm/x86_64/*.apk | sed -e 's/wezterm-/wezterm-{self.name}-/')",
            ),
            # Move it to the repo dir
            RunStep(
                "Move APKs",
                f"mv ~/packages/wezterm/x86_64/*.apk .",
            ),
            # Move and rename the keys
            RunStep(
                "Move APK keys",
                f"mv ~/.abuild/*.pub wezterm-{self.name}.pub",
            ),
        ]
    elif self.uses_zypper():
        steps.append(
            RunStep(
                "Move RPM",
                f"mv /usr/src/packages/RPMS/*/*.rpm .",
            )
        )

    patterns = self.asset_patterns()
    glob = " ".join(patterns)
    paths = "\n".join(patterns)

    return steps + [
        ActionStep(
            "Upload artifact",
            action="actions/upload-artifact@v7",
            params={"name": self.name, "path": paths},
        ),
    ]

def asset_patterns(self):
    patterns = []
    if self.uses_yum() or self.uses_zypper():
        patterns += ["wezterm-*.rpm"]
    elif "win" in self.name:
        patterns += ["WezTerm-*.zip", "WezTerm-*.exe"]
    elif "mac" in self.name:
        patterns += ["WezTerm-*.zip"]
    elif ("ubuntu" in self.name) or ("debian" in self.name):
        patterns += ["wezterm-*.deb", "wezterm-*.xz"]
    elif "alpine" in self.name:
        patterns += ["wezterm-*.apk"]
        if self.is_tag:
            patterns.append("*.pub")

    if self.app_image:
        patterns.append("*src.tar.gz")
        patterns.append("*.AppImage")
        #patterns.append("*.zsync") broken upstream: <https://github.com/linuxdeploy/linuxdeploy/issues/309>
    return patterns

def upload_artifact_nightly(self):
    steps = []

    if self.uses_yum() or self.uses_zypper():

        rpmbuild = "~/rpmbuild/RPMS/*"
        if self.uses_zypper():
            rpmbuild = "/usr/src/packages/RPMS/*"

        script = ""
        # Note that 'wezterm' MUST be last in this list,
        # otherwise the globbing will mess things up
        for pkg in ['wezterm-common', 'wezterm-gui', 'wezterm-mux-server', 'wezterm']:
            script = script + f"mv {rpmbuild}/{pkg}-*.rpm {pkg}-nightly-{self.name}.rpm\n"

        steps.append(
            RunStep(
                "Move RPM",
                script
            )
        )
    elif self.uses_apk():
        steps.append(
            RunStep(
                "Move APKs",
                f"mv ~/packages/wezterm/x86_64/*.apk wezterm-nightly-{self.name}.apk",
            )
        )

    patterns = self.asset_patterns()
    glob = " ".join(patterns)
    paths = "\n".join(patterns)

    return steps + [
        ActionStep(
            "Upload artifact",
            action="actions/upload-artifact@v7",
            params={"name": self.name, "path": paths, "retention-days": 5},
        ),
    ]

def upload_asset_nightly(self):
    steps = []

    patterns = self.asset_patterns()
    checksum = RunStep(
        "Checksum",
        f"for f in {' '.join(patterns)} ; do sha256sum $f > $f.sha256 ; done",
    )

    patterns.append("*.sha256")
    glob = " ".join(patterns)

    if self.container == GEMFURY_TARGET:
        steps += [
            RunStep(
                "Upload to gemfury",
                f"for f in wezterm*.deb ; do curl -i -F package=@$f https://$FURY_TOKEN@push.fury.io/wez/ ; done",
                env={"FURY_TOKEN": "${{ secrets.FURY_TOKEN }}"},
            ),
        ]

    return [
        ActionStep(
            "Download artifact",
            action="actions/download-artifact@v8",
            params={"name": self.name},
        ),
        checksum,
        RunStep(
            "Upload to Nightly Release",
            f"bash ci/retry.sh gh release upload --clobber nightly {glob}",
            env={"GITHUB_TOKEN": "${{ secrets.GITHUB_TOKEN }}"},
        ),
    ] + steps

def upload_asset_tag(self):
    steps = []

    patterns = self.asset_patterns()
    checksum = RunStep(
        "Checksum",
        f"for f in {' '.join(patterns)} ; do sha256sum $f > $f.sha256 ; done",
    )

    patterns.append("*.sha256")
    glob = " ".join(patterns)

    if self.container == GEMFURY_TARGET:
        steps += [
            RunStep(
                "Upload to gemfury",
                f"for f in wezterm*.deb ; do curl -i -F package=@$f https://$FURY_TOKEN@push.fury.io/wez/ ; done",
                env={"FURY_TOKEN": "${{ secrets.FURY_TOKEN }}"},
            ),
        ]

    return steps + [
        ActionStep(
            "Download artifact",
            action="actions/download-artifact@v8",
            params={"name": self.name},
        ),
        checksum,
        RunStep(
            "Create pre-release",
            "bash ci/retry.sh bash ci/create-release.sh $(ci/tag-name.sh)",
            env={
                "GITHUB_TOKEN": "${{ secrets.GITHUB_TOKEN }}",
            },
        ),
        RunStep(
            "Upload to Tagged Release",
            f"bash ci/retry.sh gh release upload --clobber $(ci/tag-name.sh) {glob}",
            env={
                "GITHUB_TOKEN": "${{ secrets.GITHUB_TOKEN }}",
            },
        ),
    ]
