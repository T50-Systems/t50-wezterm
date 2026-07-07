# ruff: noqa: F821
# pyright: reportUndefinedVariable=false
def create_flathub_pr(self):
    if not self.app_image:
        return []
    return [
        ActionStep(
            "Checkout flathub/org.wezfurlong.wezterm",
            action="actions/checkout@v7",
            params={
                "repository": "flathub/org.wezfurlong.wezterm",
                "path": "flathub",
                "token": "${{ secrets.GH_PAT }}",
            },
        ),
        RunStep(
            "Create flathub commit and push",
            "bash ci/make-flathub-pr.sh",
        ),
        RunStep(
            "Submit PR",
            'cd flathub && gh pr create --fill --body "PR automatically created by release automation in the wezterm repo"',
            env={
                "GITHUB_TOKEN": "${{ secrets.GH_PAT }}",
            },
        ),
    ]

def create_winget_pr(self):
    steps = []
    if "windows" in self.name:
        upstream_only = "github.repository == 'wezterm/wezterm'"
        steps += [
            ActionStep(
                "Checkout winget-pkgs",
                action="actions/checkout@v7",
                params={
                    "repository": "wez/winget-pkgs",
                    "path": "winget-pkgs",
                    "token": "${{ secrets.GH_PAT }}",
                },
                condition=upstream_only,
            ),
            RunStep(
                "Setup email for winget repo",
                "cd winget-pkgs && git config user.email wez@wezfurlong.org",
                condition=upstream_only,
            ),
            RunStep(
                "Setup name for winget repo",
                "cd winget-pkgs && git config user.name 'Wez Furlong'",
                condition=upstream_only,
            ),
            RunStep(
                "Create winget manifest and push to fork",
                "bash ci/make-winget-pr.sh winget-pkgs WezTerm-*.exe",
                condition=upstream_only,
            ),
            RunStep(
                "Submit PR",
                'cd winget-pkgs && gh pr create --fill --body "PR automatically created by release automation in the wezterm repo"',
                env={
                    "GITHUB_TOKEN": "${{ secrets.GH_PAT }}",
                },
                condition=upstream_only,
            ),
        ]

    return steps

def update_homebrew_tap(self):
    steps = []
    if "macos" in self.name:
        steps += [
            ActionStep(
                "Checkout homebrew tap",
                action="actions/checkout@v7",
                params={
                    "repository": "wez/homebrew-wezterm",
                    "path": "homebrew-wezterm",
                    "token": "${{ secrets.GH_PAT }}",
                },
            ),
            RunStep(
                "Update homebrew tap formula",
                "cp wezterm.rb homebrew-wezterm/Casks/wezterm.rb",
            ),
            ActionStep(
                "Commit homebrew tap changes",
                action="stefanzweifel/git-auto-commit-action@v5",
                params={
                    "commit_message": "Automated update to match latest tag",
                    "repository": "homebrew-wezterm",
                },
            ),
        ]
    elif self.app_image:
        steps += [
            ActionStep(
                "Checkout linuxbrew tap",
                action="actions/checkout@v7",
                params={
                    "repository": "wez/homebrew-wezterm-linuxbrew",
                    "path": "linuxbrew-wezterm",
                    "token": "${{ secrets.GH_PAT }}",
                },
            ),
            RunStep(
                "Update linuxbrew tap formula",
                "cp wezterm-linuxbrew.rb linuxbrew-wezterm/Formula/wezterm.rb",
            ),
            ActionStep(
                "Commit linuxbrew tap changes",
                action="stefanzweifel/git-auto-commit-action@v5",
                params={
                    "commit_message": "Automated update to match latest tag",
                    "repository": "linuxbrew-wezterm",
                },
            ),
        ]

    return steps

def global_env(self):
    self.env["CARGO_INCREMENTAL"] = "0"
    self.env["SCCACHE_GHA_ENABLED"] = "true"
    self.env["RUSTC_WRAPPER"] = "sccache"
    if "macos" in self.name:
        self.env["MACOSX_DEPLOYMENT_TARGET"] = "10.12"
    if "alpine" in self.name:
        self.env["RUSTFLAGS"] = "-C target-feature=-crt-static"
    if "win" in self.name:
        self.env["RUSTUP_WINDOWS_PATH_ADD_BIN"] = "1"
    return

def prep_environment(self, cache=True):
    steps = []
    sudo = "sudo -n " if self.needs_sudo() else ""
    if self.uses_apt():
        if self.container:
            steps += [
                RunStep(
                    "set APT to non-interactive",
                    "echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections",
                ),
            ]
        steps += [
            RunStep("Update APT", f"{sudo}apt update"),
        ]

    if self.uses_zypper() and self.container:
        steps += [
            RunStep(
                "Seed GITHUB_PATH to work around possible @action/core bug",
                'echo "$PATH:/bin:/usr/bin" >> $GITHUB_PATH',
            ),
            RunStep(
                "Install util-linux",
                "zypper install -y util-linux",
            ),
        ]
    if self.container:
        if ("fedora" in self.container) or (
            ("centos" in self.container) and ("centos7" not in self.container)
        ):
            steps += [
                RunStep(
                    "Install config manager",
                    "dnf install -y 'dnf-command(config-manager)'",
                ),
            ]
        if "centos:stream8" in self.container:
            steps += [
                RunStep(
                    "Enable PowerTools",
                    "dnf config-manager --set-enabled powertools",
                ),
            ]
        if "centos:stream9" in self.container:
            steps += [
                # This holds the xcb bits
                RunStep(
                    "Enable CRB repo for X bits",
                    "dnf config-manager --set-enabled crb",
                ),
            ]
        if "alpine" in self.container:
            steps += [
                RunStep(
                    "Upgrade system",
                    "apk upgrade --update-cache",
                    shell="sh",
                ),
                RunStep(
                    "Install CI dependencies",
                    "apk add nodejs zstd wget bash coreutils tar findutils",
                    shell="sh",
                ),
                RunStep(
                    "Allow root login",
                    "sed 's/root:!/root:*/g' -i /etc/shadow",
                ),
            ]
        if "opensuse" in self.container:
            steps += [
                # This holds the xcb bits
                RunStep(
                    "Install tar",
                    "zypper install -yl tar gzip",
                ),
            ]

    steps += self.install_newer_compiler()
    steps += self.install_git()
    steps += self.install_curl()

    if self.uses_apt() and self.container:
        steps += [
            RunStep("Update APT", f"{sudo}apt update"),
        ]

    steps += self.install_openssh_server()
    steps += self.checkout()
    # We should be able to cache mac builds now?
    steps += self.install_rust()  # cache="mac" not in self.name)
    steps += self.install_system_deps()
    return steps

def pull_request(self):
    steps = self.prep_environment()
    steps += self.build_all_release()
    steps += self.test_all()
    steps += self.package()
    steps += self.upload_artifact()

    return (
        Job(
            runs_on=self.os,
            container=self.container,
            steps=steps,
            env=self.env,
        ),
        None,
    )

def checkout(self, submodules=True):
    steps = []
    if self.container:
        steps += [
            RunStep(
                "Workaround git permissions issue",
                "git config --global --add safe.directory /__w/wezterm/wezterm",
            )
        ]
    steps += [CheckoutStep(submodules=submodules, container=self.container)]
    return steps

def continuous(self):
    steps = self.prep_environment()
    steps += self.build_all_release()
    steps += self.test_all()
    steps += self.package(trusted=True)
    steps += self.upload_artifact_nightly()

    self.env["BUILD_REASON"] = "Schedule"

    uploader = Job(
        runs_on="ubuntu-latest",
        steps=self.checkout(submodules=False) + self.upload_asset_nightly(),
    )

    return (
        Job(
            runs_on=self.os,
            container=self.container,
            steps=steps,
            env=self.env,
        ),
        uploader,
    )

def tag(self):
    steps = self.prep_environment()
    steps += self.build_all_release()
    steps += self.test_all()
    steps += self.package(trusted=True)
    steps += self.upload_artifact()

    uploader = Job(
        runs_on="ubuntu-latest",
        steps=self.checkout(submodules=False)
        + self.update_homebrew_tap()
        + self.upload_asset_tag()
        + self.create_winget_pr()
        + self.create_flathub_pr(),
    )

    return (
        Job(
            runs_on=self.os,
            container=self.container,
            steps=steps,
            env=self.env,
        ),
        uploader,
    )
