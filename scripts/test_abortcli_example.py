# Helper for Github Actions to run the `abortcli` example
# and verify it exits with a SIGABORT
#
# Requires POSIX to detect sigabort

import os
import re
import signal
import subprocess
import sys
from pathlib import Path

RUST_VERSION_PATTERN = re.compile(r"(\d+).(\d+).(\d+)")


def parse_rust_version(version):
    if version in ("stable", "beta", "nightly"):
        return version
    else:
        mtch = RUST_VERSION_PATTERN.fullmatch(version)
        if mtch is None:
            fatal(f"Invalid rust version: {version!r}")
        return tuple(map(int, mtch.groups()))


def fatal(msg):
    print(msg, file=sys.stderr)
    sys.exit(1)


def main(args):
    if len(args) < 2 or args[0] != "--features":
        fatal("Expected `--features` as first argument")
    else:
        features = args[1].split(",")
        if features == [""]:
            features = []
    if len(args) < 4 or args[2] != "--rust-version":
        fatal("Expected `--rust-version` as third argument")
    else:
        rust_version = parse_rust_version(args[3])
    if os.name != "posix":
        fatal(f"Need POSIX to check signal for example, but have {os.name!r}")
    example_binary = Path("target/release/examples/abortcli")
    if not example_binary.is_file():
        raise FileNotFoundError(f"Missing example binary: {example_binary}")
    print("Running `abortcli` example:")
    result = subprocess.run(
        [example_binary],
    )
    # After Rust 1.59, we support using inline assembly to issue trap instructions
    supports_trap_instruction = isinstance(rust_version, str) or rust_version[1] >= 59
    # For some reason Rust 1.59 and 1.31 trigger SIGILL on double panics
    double_panic_triggers_trap = rust_version[1] in range(31, 60)
    # If our only feature is `abort-via-trap
    if (
        features == ["abort-via-trap"]
        and (supports_trap_instruction or double_panic_triggers_trap)
    ) or (features == [] and double_panic_triggers_trap):
        expected_signal = "SIGILL"
    else:
        expected_signal = "SIGABRT"
    expected_signal_code = getattr(signal, expected_signal)
    if result.returncode == -expected_signal_code:
        print(f"Detected a {expected_signal}, as expected")
    else:
        fatal(
            f"Expected a `{expected_signal}` exit code (-{expected_signal_code}), but got {result.returncode}"
        )


if __name__ == "__main__":
    main(sys.argv[1:])
