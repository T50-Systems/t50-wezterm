#!/usr/bin/env python3
# pyright: reportAttributeAccessIssue=false, reportCallIssue=false, reportOptionalIterable=false
import os
import glob
from copy import deepcopy

# The build from this target will be pushed to the gemfury APT repo
GEMFURY_TARGET = "ubuntu:22.04"
# The build from this target will be baked into the AppImage
APPIMAGE_TARGET = "ubuntu:24.04"

TRIGGER_PATHS = [
    "**/*.rs",
    "**/Cargo.lock",
    "**/Cargo.toml",
    ".cargo/config.toml",
    "assets/fonts/**/*",
    "assets/icon/*",
    "ci/deploy.sh",
]

TRIGGER_PATHS_APPIMAGE = [
    "ci/appimage.sh",
    "ci/appstreamcli",
    "ci/source-archive.sh",
]

TRIGGER_PATHS_UNIX = [
    "assets/open-wezterm-here",
    "assets/shell-completion/**/*",
    "assets/shell-integration/**/*",
    "assets/wezterm-nautilus.py",
    "assets/wezterm.appdata.xml",
    "assets/wezterm.desktop",
    "get-deps",
    "ci/tag-name.sh",
    "termwiz/data/wezterm.terminfo",
]

TRIGGER_PATHS_MAC = [
    "assets/macos/**/*",
    "ci/macos-entitlement.plist",
    "get-deps",
    "ci/tag-name.sh",
]

TRIGGER_PATHS_WIN = [
    "assets/windows/**/*",
    "ci/windows-installer.iss",
]


def yv(v, depth=0):
    if isinstance(v, bool):
        return "true" if v else "false"
    if v is None:
        return "nil"

    if isinstance(v, str):
        if "\n" in v:
            indent = "  " * depth
            result = ""
            for line in v.splitlines():
                result = result + "\n" + (f"{indent}{line}" if line else "")
            return "|" + result
        # This is hideous
        if '"' in v:
            return "'" + v + "'"
        return '"' + v + '"'

    return v


class Step:
    def render(self, f, depth=0):
        raise NotImplementedError(repr(self))


class RunStep(Step):
    def __init__(self, name, run, shell="bash", env=None, condition=None):
        self.name = name
        self.run = run
        self.shell = shell
        self.env = env
        self.condition = condition

    def render(self, f, depth=0):
        indent = "  " * depth
        f.write(f"{indent}- name: {yv(self.name)}\n")
        if self.condition:
            f.write(f"{indent}  if: {self.condition}\n")
        if self.env:
            f.write(f"{indent}  env:\n")
            keys = list(self.env.keys())
            keys.sort()
            for k in keys:
                v = self.env[k]
                f.write(f"{indent}    {k}: {v}\n")
        if self.shell:
            f.write(f"{indent}  shell: {self.shell}\n")

        run = self.run

        f.write(f"{indent}  run: {yv(run, depth + 2)}\n")


class ActionStep(Step):
    def __init__(self, name, action, params=None, env=None, condition=None, id=None):
        self.name = name
        self.action = action
        self.params = params
        self.env = env
        self.condition = condition
        self.id = id

    def render(self, f, depth=0):
        indent = "  " * depth
        f.write(f"{indent}- name: {yv(self.name)}\n")
        f.write(f"{indent}  uses: {self.action}\n")
        if self.id:
            f.write(f"{indent}  id: {self.id}\n")
        if self.condition:
            f.write(f"{indent}  if: {self.condition}\n")
        if self.params:
            f.write(f"{indent}  with:\n")
            for k, v in self.params.items():
                f.write(f"{indent}    {k}: {yv(v, depth + 3)}\n")
        if self.env:
            f.write(f"{indent}  env:\n")
            for k, v in self.env.items():
                f.write(f"{indent}    {k}: {yv(v, depth + 3)}\n")


class CacheStep(ActionStep):
    def __init__(self, name, path, key, id=None):
        super().__init__(
            name, action="actions/cache@v5.0.5", params={"path": path, "key": key}, id=id
        )


class SccacheStep(ActionStep):
    def __init__(self, name):
        super().__init__(name, action="mozilla-actions/sccache-action@v0.0.10")


class CheckoutStep(ActionStep):
    def __init__(self, name="checkout repo", submodules=True, container=None):
        params = {}
        if submodules:
            params["submodules"] = "recursive"
        super().__init__(name, action="actions/checkout@v7", params=params)


class InstallCrateStep(ActionStep):
    def __init__(self, crate: str, key: str, version=None):
        params = {"crate": crate, "cache-key": key}
        if version is not None:
            params["version"] = version
        super().__init__(
            f"Install {crate} from Cargo",
            action="baptiste0928/cargo-install@v3",
            params=params,
        )


class Job:
    def __init__(self, runs_on, container=None, steps=None, env=None):
        self.runs_on = runs_on
        self.container = container
        self.steps = steps
        self.env = env

    def render(self, f, depth=0):
        f.write("\n    steps:\n")
        for s in self.steps:
            s.render(f, depth)




def _target_chunk(name):
    try:
        with open(os.path.join(os.path.dirname(__file__), name), encoding="utf-8") as f:
            return f.read()
    except OSError as exc:
        raise RuntimeError(f"failed to read workflow target chunk {name}") from exc


class Target:
    exec(_target_chunk("generate_workflows_target_core.py"))
    exec(_target_chunk("generate_workflows_target_build.py"))
    exec(_target_chunk("generate_workflows_target_publish.py"))


TARGETS = [
    Target(container="ubuntu:22.04", continuous_only=True),
    Target(container="ubuntu:24.04", continuous_only=True),
    Target(container="debian:12", continuous_only=True),
    Target(name="centos9", container="quay.io/centos/centos:stream9"),
    Target(name="macos", os="macos-latest"),
    # https://fedoraproject.org/wiki/End_of_life?rd=LifeCycle/EOL
    Target(container="fedora:41"),
    # Target(container="alpine:3.15"),

    Target(name="windows", os="windows-2025", rust_target="x86_64-pc-windows-msvc"),
]


def generate_actions(namer, jobber, trigger, is_continuous, is_tag=False):
    have_gemfury = False
    have_appimage = False
    for t in TARGETS:
        # Clone the definition, as some Target methods called
        # in the body below have side effects that we don't
        # want to bleed across into different schedule types
        t = deepcopy(t)

        if t.app_image:
            have_appimage = True
        if t.container == GEMFURY_TARGET:
            have_gemfury = True

        t.is_tag = is_tag
        # if t.continuous_only and not is_continuous:
        #    continue
        name = namer(t).replace(":", "")
        print(name)
        job, uploader = jobber(t)

        file_name = f".github/workflows/gen_{name}.yml"
        if job.container:
            if t.app_image:
                container = f"container:\n      image: {yv(job.container)}\n      options: --privileged"
            else:
                container = f"container: {yv(job.container)}"

        else:
            container = ""

        trigger_paths = [file_name]
        trigger_paths += TRIGGER_PATHS
        if "win" in name:
            trigger_paths += TRIGGER_PATHS_WIN
        elif "macos" in name:
            trigger_paths += TRIGGER_PATHS_MAC
        else:
            trigger_paths += TRIGGER_PATHS_UNIX
        if t.app_image:
            trigger_paths += TRIGGER_PATHS_APPIMAGE

        trigger_paths = "- " + "\n      - ".join(yv(p) for p in sorted(trigger_paths))
        trigger_with_paths = trigger.replace("@PATHS@", trigger_paths)

        try:
            with open(file_name, "w") as f:
                f.write(
                    f"""name: {name}
{trigger_with_paths}
jobs:
  build:
    runs-on: {yv(job.runs_on)}
    {container}
"""
                )

                t.render_env(f)

                job.render(f, 3)

                # We upload using a native runner as github API access
                # inside a container is really unreliable and can result
                # in broken releases that can't automatically be repaired
                # <https://github.com/cli/cli/issues/4863>
                if uploader:
                    f.write(
                        """
  upload:
    runs-on: ubuntu-latest
    needs: build
    if: github.repository == 'wezterm/wezterm' || github.repository == 'T50-Systems/t50-wezterm'
    permissions:
      contents: write
      pages: write
      id-token: write
"""
                    )
                    uploader.render(f, 3)
        except OSError as exc:
            raise RuntimeError(f"failed to write workflow file {file_name}") from exc
        # Sanity check the yaml, if pyyaml is available
        try:
            import yaml

            with open(file_name) as f:
                yaml.safe_load(f)
        except ImportError:
            pass
    if not have_appimage:
        raise NotImplementedError("no appimage target is present")
    if not have_gemfury:
        raise NotImplementedError("no gemfury target is present")


def generate_pr_actions():
    generate_actions(
        lambda t: f"{t.name}",
        lambda t: t.pull_request(),
        trigger="""
on:
  pull_request:
    branches:
      - main
      - dev
    paths:
      @PATHS@
""",
        is_continuous=False,
    )


def continuous_actions():
    generate_actions(
        lambda t: f"{t.name}_continuous",
        lambda t: t.continuous(),
        trigger="""
on:
  schedule:
    - cron: "10 3 * * *"
  push:
    branches:
      - main
    paths:
      @PATHS@
""",
        is_continuous=True,
    )


def tag_actions():
    generate_actions(
        lambda t: f"{t.name}_tag",
        lambda t: t.tag(),
        trigger="""
on:
  push:
    tags:
      - "20*"
""",
        is_continuous=True,
        is_tag=True,
    )


def remove_gen_actions():
    for name in glob.glob(".github/workflows/gen_*.yml"):
        try:
            os.remove(name)
        except OSError as exc:
            raise RuntimeError(f"failed to remove generated workflow {name}") from exc


remove_gen_actions()
generate_pr_actions()
continuous_actions()
tag_actions()
