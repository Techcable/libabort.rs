# Changelog

Notable changes to this project should be documented in this file.
Make sure it is up to date before performing a release.

This project follows the [Keep a Changelog](https://keepachangelog.com/en/2.0.0/) format wherever that is reasonable.

The "title" of each release should be its first line.
A title is required for publishing a github release, so all versions should have one.

Most changes include the relevant [jj](https://jj-vcs.dev) change ids in parens. An example of a change id is wuoxvnsw.

## Unreleased

### Fixes
- Use `autocfg` to check if nightly features work before using them (wulnptwt)
  - Avoids build failures caused by the removal of nightly functions.
- Document use of unwinding past extern "C" on 1.81 (qmvsysnu)
- On nightly, use `core::process::abort_immediately()` instead of `core::intrinsics::abort()` (mxmspryo)
  - The latter has been removed on recent nightly versions.

## v0.1.9 - 2024-09-05
On Rust 1.81, abort by unwinding past `extern "C"`

In Rust 1.81, unwinding past an `extern "C"` is guaranteed to produce an abort.
This is preferable to the previous fallback behavior of triggering a double panic
because it doesn't produce a backtrace.

On previous versions, this functionality is disabled because it triggers undefined behavior.

This only affects the "fallback" behavior when there is no other implementation of abort.

Decrease minimum supported rust version to 1.31.
Use Github Actions for automated testing.

## v0.1.8 - 2024-06-19
Implement a `trap()` function.

The trap() function issues an illegal instruction on supported architectures,
falling back to calling abort() elsewhere.
It is semantically equivalent to the `llvm.trap` and `__builtin_abort()` intrinsics.

On some architectures, issuing an illegal instruction is cheaper than calling an abort function.

The `trap` function can be used to implement the `abort()` function by enabling the `abort-via-trap` feature.
This feature does nothing if the architecture is unsupported.

Currently supported architectures for `trap`:
- x86_64
- i686 (x86)
- aarch64
- arm
- wasm32
- wasm64

On nightly, the trap function wraps the `core::intrisnics::abort()` function.

Additional fixes:
- Use `doc(cfg(...))` to document feature requirements
- Properly configure feature flags for docs.rs
- Drop rustversion dependency in favor of detection in build.rs script.

## v0.1.7 - 2024-06-10
Document clear distinction between immediate & fallback aborts.

Could be important in some safety scenarios, and explains why fallback implementation prints to stderr.

The README.md file could use some cleanup after this.

Forgot to number a version v0.1.5 and made a silly mistake in v0.1.6 which I fixed in the next commit.
I yanked v0.1.6 from crates.io because it has no changes except a missing feature-declaration.
I also combined the release notes for v0.1.6 and v0.1.7.

## v0.1.4 - 2024-06-10
Fix compilation errors with bad config flags.

Before this release, the crate didn't really work very well.
As a result, previous versions have been yanked because the unreliability could violate safety guarantees.

### Changes
- Support rust versions since 1.54
  - Document MSRV
- Add abortcli example which just prints & aborts

## v0.1.3 - 2024-05-20 (*YANKED*)
Add constructor for [`AbortGuard`].

Implement `Clone` and `Default` for [`AbortGuard`].

[`AbortGuard`]: https://docs.rs/libabort/0.1/libabort/struct.AbortGuard.html

## v0.1.2 - 2024-05-20 (*YANKED*)
Finish describing fallback impl in README.

Before this the sentence was unfinished.

## v0.1.1 - 2024-05-20 (*YANKED*)
Stop using `cfg_if!` macro.

At the time, I considered this an unnecessary dependency.

## v0.1.0 - 2024-05-20 (*YANKED*)
Initial Release

An implementation of the `abort` function that works without the standard library.
Depending on feature flags, this relies on double-panics or `libc::abort()`.
