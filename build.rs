pub fn main() {
    let rust_version = rustversion_detect::detect_version().unwrap();
    let autocfg = autocfg::AutoCfg::new().unwrap();

    // version detection
    emit_check_cfg("has_cfg_panic", None);
    if rust_version.is_since_minor_version(1, 60) {
        println!("cargo:rustc-cfg=has_cfg_panic");
    }
    emit_check_cfg("has_doc_cfg", None);
    if rust_version.is_nightly() {
        println!("cargo:rustc-cfg=has_doc_cfg");
    }
    emit_check_cfg("is_cabi_unwind_guaranteed_abort", None);
    if rust_version.is_since_minor_version(1, 81) {
        // As of Rust 1.81, unwinding past an `extern "C"` function
        // is guaranteed to unwind
        //
        // Before this release, it caused undefined behavior.
        println!("cargo:rustc-cfg=is_cabi_unwind_guaranteed_abort");
    }
    let target_arch = {
        let mut values = load_cargo_cfg_var("target_arch");
        assert_eq!(
            values.len(),
            1,
            "Must have one and only one `cfg!(target_arch)`"
        );
        values.remove(0)
    };
    // can't use matches! due to MSRV
    #[allow(clippy::match_like_matches_macro)]
    let supported_arch = match &*target_arch {
        "x86_64" | "x86" | "arm" | "aarch64" => true,
        _ => false,
    };
    let trap_impl_name = if target_arch == "wasm32" && rust_version.is_since_minor_version(1, 37) {
        // Use core::arch::wasm32::unreachable() intrinsic
        //
        // Requires `simd_wasm32` feature for the module (stable 1.33),
        // and the `unreachable_wasm32` feature for the function (stable 1.37)
        "wasm32-intrinsic"
    } else if target_arch == "wasm64"
        && rust_version.is_nightly()
        && WASM64_UNREACHABLE.is_present(&autocfg)
    {
        // Use core::arch::wasm64::unreachable() intrinsic
        //
        // Requires `simd_wasm64` feature for the module (unstable, issue #90599)
        emit_warning(&"The `wasm64` architecture is currently untested (issue #3)");
        "wasm64-intrinsic"
    } else if rust_version.is_nightly() && CORE_INTRINSICS_ABORT.is_present(&autocfg) {
        // The `core::intrinsics` module requires nightly.
        // It is an "internal" feature that will never be directly stabilized.
        "core-intrinsics"
    } else if supported_arch && rust_version.is_since_minor_version(1, 59) {
        "assembly"
    } else {
        "fallback"
    };
    emit_check_cfg(
        "trap_impl",
        Some(vec![
            "core-intrinsics",
            "assembly",
            "wasm32-intrinsic",
            "wasm64-intrinsic",
            "fallback",
        ]),
    );
    println!("cargo:rustc-cfg=trap_impl=\"{}\"", trap_impl_name);
    let abort_impl_name = if has_cargo_feature("std") {
        "std"
    } else if has_cargo_feature("libc") {
        "libc"
    } else if has_cargo_feature("abort-via-trap") && trap_impl_name != "fallback" {
        "trap"
    } else {
        "fallback"
    };
    emit_check_cfg("abort_impl", Some(vec!["std", "libc", "trap", "fallback"]));
    println!("cargo:rustc-cfg=abort_impl=\"{}\"", abort_impl_name);
    // never need to be re-run
    println!("cargo:rerun-if-changed=build.rs");
}

const CORE_INTRINSICS_ABORT: NightlyAbortFunc = NightlyAbortFunc {
    path: "core::intrinsics::abort",
    feature: "core_intrinsics",
    is_unsafe: false,
};
const WASM64_UNREACHABLE: NightlyAbortFunc = NightlyAbortFunc {
    path: "core::arch::wasm64::unreachable",
    feature: "simd_wasm64",
    is_unsafe: false,
};
struct NightlyAbortFunc {
    path: &'static str,
    feature: &'static str,
    is_unsafe: bool,
}
impl NightlyAbortFunc {
    fn is_present(&self, cfg: &autocfg::AutoCfg) -> bool {
        let probe = format!(
            r##"
            #![feature({feature})]
            #![no_std]

            fn do_abort() -> ! {{
                {block} {{
                    {func}();
                }}
            }}
            "##,
            block = if self.is_unsafe { "unsafe" } else { "" },
            feature = self.feature,
            func = self.path,
        );
        let res = cfg.probe_raw(&probe).is_ok();
        if !res {
            println!(
                "cargo:warning=The `{}` function is missing on current nightly",
                self.path
            );
        }
        res
    }
}

fn load_cargo_cfg_var(name: &'static str) -> Vec<String> {
    let env_var = format!("CARGO_CFG_{}", name.to_uppercase());
    match std::env::var(&env_var) {
        Ok(val) => val.split(",").map(String::from).collect(),
        Err(std::env::VarError::NotUnicode(_)) => panic!("Var not unicode: {:?}", env_var),
        Err(std::env::VarError::NotPresent) => vec![],
    }
}

fn has_cargo_feature(name: &str) -> bool {
    let name = name.replace('-', "_").to_uppercase();
    std::env::var_os(format!("CARGO_FEATURE_{}", name)).is_some()
}

fn emit_check_cfg(name: &'static str, values: Option<Vec<&str>>) {
    let mut values_spec = String::new();
    if let Some(values) = values {
        values_spec.push_str(", values(");
        for (index, value) in values.into_iter().enumerate() {
            if index > 0 {
                values_spec.push_str(", ");
            }
            values_spec.push('"');
            values_spec.push_str(value);
            values_spec.push('"');
        }
        values_spec.push(')');
    }
    println!("cargo:rustc-check-cfg=cfg({}{})", name, values_spec);
}

fn emit_warning(msg: &dyn std::fmt::Display) {
    println!("cargo:warning={}", msg);
}
