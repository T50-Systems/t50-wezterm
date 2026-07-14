# ruff: noqa: F821
# pyright: reportUndefinedVariable=false
def __init__(
    self,
    name=None,
    os="ubuntu-latest",
    container=None,
    bootstrap_git=False,
    rust_target=None,
    continuous_only=False,
    is_tag=False,
):
    if not name:
        if container:
            name = container
        else:
            name = os
    self.name = name.replace(":", "")
    self.os = os
    self.container = container
    self.bootstrap_git = bootstrap_git
    self.rust_target = rust_target
    self.continuous_only = continuous_only
    self.app_image = container == APPIMAGE_TARGET
    self.env = {}
    self.is_tag = is_tag

def render_env(self, f, depth=0):
    self.global_env()
    if self.env:
        indent = "    "
        f.write(f"{indent}env:\n")
        for k, v in self.env.items():
            f.write(f"{indent}  {k}: {yv(v, depth + 3)}\n")

def uses_yum(self):
    return "fedora" in self.name or "centos" in self.name

def uses_apt(self):
    return "ubuntu" in self.name or "debian" in self.name

def uses_apk(self):
    return "alpine" in self.name

def uses_zypper(self):
    return "suse" in self.name

def needs_sudo(self):
    return not self.container and self.uses_apt()

def install_system_package(self, name):
    installer = None
    if self.uses_yum():
        installer = "yum"
    elif self.uses_apt():
        installer = "apt-get"
    elif self.uses_apk():
        installer = "apk"
    elif self.uses_zypper():
        installer = "zypper"
    else:
        return []
    if self.needs_sudo():
        installer = f"sudo -n {installer}"
    if self.uses_apk():
        return [RunStep(f"Install {name}", f"{installer} add {name}")]
    else:
        return [RunStep(f"Install {name}", f"{installer} install -y {name}")]

def install_curl(self):
    if (
        self.uses_yum()
        or self.uses_apk()
        or self.uses_zypper()
        or (self.uses_apt() and self.container)
    ):
        if "centos:stream9" in self.container:
            return self.install_system_package("curl-minimal")
        else:
            return self.install_system_package("curl")
    return []

def install_openssh_server(self):
    steps = []
    if (
        self.uses_yum()
        or self.uses_zypper()
        or (self.uses_apt() and self.container)
    ):
        steps += [
            RunStep("Ensure /run/sshd exists", "mkdir -p /run/sshd")
        ] + self.install_system_package("openssh-server")
    if self.uses_apk():
        steps += self.install_system_package("openssh")
    return steps

def install_newer_compiler(self):
    steps = []
    if self.name == "centos7":
        steps.append(
            RunStep(
                "Install SCL",
                "yum install -y centos-release-scl-rh",
            )
        )
        steps.append(
            RunStep(
                "Update compiler",
                "yum install -y devtoolset-9-gcc devtoolset-9-gcc-c++",
            )
        )
    return steps

def install_git(self):
    steps = []
    if self.bootstrap_git:
        GIT_VERS = "2.26.2"
        steps.append(
            CacheStep(
                "Cache Git installation",
                path="/usr/local/git",
                key=f"{self.name}-git-{GIT_VERS}",
            )
        )

        pre_reqs = ""
        if self.uses_yum():
            pre_reqs = "yum install -y wget curl-devel expat-devel gettext-devel openssl-devel zlib-devel gcc perl-ExtUtils-MakeMaker make"
        elif self.uses_apt():
            pre_reqs = "apt-get install -y wget libcurl4-openssl-dev libexpat-dev gettext libssl-dev libz-dev gcc libextutils-autoinstall-perl make"
        elif self.uses_zypper():
            pre_reqs = "zypper install -y wget libcurl-devel libexpat-devel gettext-tools libopenssl-devel zlib-devel gcc perl-ExtUtils-MakeMaker make"

        steps.append(
            RunStep(
                name="Install Git from source",
                shell="bash",
                run=f"""{pre_reqs}
if test ! -x /usr/local/git/bin/git ; then
cd /tmp
wget https://github.com/git/git/archive/v{GIT_VERS}.tar.gz
tar xzf v{GIT_VERS}.tar.gz
cd git-{GIT_VERS}
make prefix=/usr/local/git install
fi
ln -s /usr/local/git/bin/git /usr/local/bin/git""",
            )
        )

    else:
        if "tumbleweed" in self.name:
            # git-core requires /usr/bin/which and that gets satisfied
            # by busybox-which by default, which blocks installing
            # rpmbuild, which depends on the which rpm directly,
            # but that is blocked by the conflicting busybox-which rpm.
            # So we explicitly install which here now
            steps += self.install_system_package("which")

        steps += self.install_system_package("git")

    return steps

def install_rust(self, cache=True, toolchain="stable"):
    params = {}
    if self.rust_target:
        params["target"] = self.rust_target
    steps = []
    # Manually setup rust toolchain in CentOS7 curl is too old for the action
    if "centos7" in self.name:
        steps += [
            RunStep(
                name="Install Rustup",
                run="""
if ! command -v rustup &>/dev/null; then
  curl --proto '=https' --tlsv1.2 --retry 10 -fsSL "https://sh.rustup.rs" | sh -s -- --default-toolchain none -y
  echo "${CARGO_HOME:-$HOME/.cargo}/bin" >> $GITHUB_PATH
fi
""",
            ),
            RunStep(
                name="Setup Toolchain",
                run=f"""
rustup toolchain install {toolchain} --profile minimal --no-self-update
rustup default {toolchain}
""",
            ),
        ]
    elif "macos" in self.name:
        steps += [
            RunStep(
                name="Install Rust (ARM)",
                run="rustup target add aarch64-apple-darwin",
            ),
            RunStep(
                name="Install Rust (Intel)",
                run="rustup target add x86_64-apple-darwin",
            )
        ]
    else:
        steps += [
            ActionStep(
                name="Install Rust",
                action=f"dtolnay/rust-toolchain@{toolchain}",
                params=params,
            ),
        ]
    if cache:
        steps += [
            SccacheStep(name="Compile with sccache"),
            # Cache vendored dependecies
            CacheStep(
                name="Cache Rust Dependencies",
                path="vendor\n.cargo/config.toml",
                key="cargo-deps-${{ hashFiles('**/Cargo.lock', '.cargo/config.toml') }}",
                id="cache-cargo-vendor",
            ),
            # Vendor dependencies
            RunStep(
                name="Vendor dependecies",
                condition="steps.cache-cargo-vendor.outputs.cache-hit != 'true'",
                run="cargo vendor --locked --versioned-dirs >> .cargo/config.toml",
            ),
        ]
        if "win" in self.name:
            cache_key = "${{ steps.openssl-cache-context.outputs.key }}"
            cache_prefix = f"windows-openssl-msvc-static-v3-{cache_key}"
            steps += [
                RunStep(
                    name="Fingerprint static OpenSSL cache inputs",
                    shell="pwsh",
                    run="./ci/windows-openssl-cache.ps1 key",
                    id="openssl-cache-context",
                ),
                CacheStep(
                    name="Restore static OpenSSL",
                    path="target/ci-cache/openssl/x86_64-pc-windows-msvc",
                    key=f"{cache_prefix}-restore",
                    restore_keys=f"{cache_prefix}-",
                    restore_only=True,
                    id="cache-openssl",
                ),
                RunStep(
                    name="Activate cached static OpenSSL",
                    shell="pwsh",
                    run="./ci/windows-openssl-cache.ps1 activate",
                    env={"OPENSSL_CACHE_KEY": cache_key},
                    id="openssl-cache-activate",
                ),
            ]
    return steps
