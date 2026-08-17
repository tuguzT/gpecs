set default-list

[windows]
set shell := ["powershell.exe", "-NoLogo", "-Command"]

PROJECT := "gpecs"
PROFILE_DEV := "dev"
PROFILE_OPT := "release"
PROFILE_GPU := "gpu_capture"
TOOLCHAIN_NIGHTLY := "nightly"
MIRIFLAGS_FULL := """
    -Zmiri-many-seeds=0..16 \
    -Zmiri-strict-provenance \
    -Zmiri-symbolic-alignment-check \
    -Zrandomize-layout \
    -Zmiri-tree-borrows \
"""

[private]
cargo base package binary profile toolchain args $MIRIFLAGS:
    cargo\
        {{ if toolchain == "" { "" } else { " +" + toolchain } }}\
        {{ " " + base }}\
        {{ if package == "" { "" } else { " --package " + quote(PROJECT + if package == PROJECT { "" } else { "_" + package }) } }}\
        {{ if binary == "" { "" } else { " --bin " + quote(PROJECT + if binary == PROJECT { "" } else { "_" + binary }) } }}\
        {{ if profile == "" { "" } else { " --profile " + quote(profile) } }}\
        {{ if args == "" { "" } else { " " + args } }}

[group("doc")]
doc package="": (cargo "doc" package "" "" TOOLCHAIN_NIGHTLY "" "")

[arg("profile", long)]
[arg("toolchain", long)]
[group("lint")]
lint package="" profile="" toolchain="": (cargo "clippy" package "" profile toolchain "" "")

[arg("profile", long)]
[arg("toolchain", long)]
[group("build")]
build package="" profile="" toolchain="": (cargo "build" package "" profile toolchain "" "")

[arg("profile", long)]
[arg("toolchain", long)]
[group("test")]
test package="" profile="" toolchain="": (cargo "test" package "" profile toolchain "" "")

[arg("profile", long)]
[arg("toolchain", long)]
[group("run")]
run binary="" profile="" toolchain="": (cargo "run" "" binary profile toolchain "" "")

[arg("profile", long)]
[arg("toolchain", long)]
[group("bench")]
bench package="" profile="" toolchain="": (cargo "bench" package "" profile toolchain "" "")

[arg("toolchain", long)]
[group("build")]
[group("dev")]
build-dev package="" toolchain="": (build package PROFILE_DEV toolchain)

[arg("toolchain", long)]
[group("dev")]
[group("test")]
test-dev package="" toolchain="": (test package PROFILE_DEV toolchain)

[arg("toolchain", long)]
[group("dev")]
[group("run")]
run-dev binary="" toolchain="": (run binary PROFILE_DEV toolchain)

[arg("toolchain", long)]
[group("build")]
[group("opt")]
build-opt package="" toolchain="": (build package PROFILE_OPT toolchain)

[arg("toolchain", long)]
[group("opt")]
[group("test")]
test-opt package="" toolchain="": (test package PROFILE_OPT toolchain)

[arg("toolchain", long)]
[group("opt")]
[group("run")]
run-opt binary="" toolchain="": (run binary PROFILE_OPT toolchain)

[arg("toolchain", long)]
[group("build")]
[group("gpu")]
build-gpu package="" toolchain="": (build package PROFILE_GPU toolchain)

[arg("toolchain", long)]
[group("gpu")]
[group("test")]
test-gpu package="" toolchain="": (test package PROFILE_GPU toolchain)

[arg("toolchain", long)]
[group("gpu")]
[group("run")]
run-gpu binary="" toolchain="": (run binary PROFILE_GPU toolchain)

[arg("toolchain", long)]
[group("bench")]
[group("gpu")]
bench-gpu package="" toolchain="": (bench package PROFILE_GPU toolchain)

[arg("flags", long)]
[group("test")]
[group("ub")]
test-ub package="" flags="": (cargo "miri test" package "" "" TOOLCHAIN_NIGHTLY "" flags)

[group("test")]
[group("ub")]
test-ub-full package="": (test-ub package MIRIFLAGS_FULL)
