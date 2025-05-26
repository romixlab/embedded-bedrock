use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash as _, Hasher as _},
};

use proc_macro::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub(crate) fn crate_local_disambiguator() -> u64 {
    // We want a deterministic, but unique-per-macro-invocation identifier. For that we
    // hash the call site `Span`'s debug representation, which contains a counter that
    // should disambiguate macro invocations within a crate.
    hash(&format!("{:?}", Span::call_site()))
}

/// work around restrictions on length and allowed characters imposed by macos linker
/// returns (note the comma character for macos):
///   under macos: ".acc," + 16 character hex digest of symbol's hash
///   otherwise:   ".acc." + prefix + symbol
pub(crate) fn linker_section(for_macos: bool, prefix: Option<&str>, symbol: &str) -> String {
    let mut sub_section = if let Some(prefix) = prefix {
        format!(".{prefix}.{symbol}")
    } else {
        format!(".{symbol}")
    };

    if for_macos {
        sub_section = format!(",{:x}", hash(&sub_section));
    }

    format!(".cnt{sub_section}")
}

#[derive(Copy, Clone)]
pub enum CounterKind {
    Error,
    Warning,
    Info,
}

// impl Into<&str> for CounterKind {
//     fn into(self) -> &'static str {
//         match self {
//             CounterKind::Error => "error",
//             CounterKind::Warning => "warning",
//             CounterKind::Info => "info",
//         }
//     }
// }

impl CounterKind {
    fn tag(&self) -> &'static str {
        match self {
            CounterKind::Error => "cnt_err",
            CounterKind::Warning => "cnt_warn",
            CounterKind::Info => "cnt_info",
        }
    }
}

pub(crate) fn static_variable(counter_kind: CounterKind, data: &str) -> TokenStream2 {
    let sym_name = crate::symbol::mangled(counter_kind.tag(), data);
    let section = linker_section(false, None, &sym_name);
    let section_for_macos = linker_section(true, None, &sym_name);

    quote!({
        #[cfg_attr(target_os = "macos", link_section = #section_for_macos)]
        #[cfg_attr(not(target_os = "macos"), link_section = #section)]
        #[export_name = #sym_name]
        static CNT: u8 = 0;
        &CNT as *const u8 as usize
    })
}

fn hash(string: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    string.hash(&mut hasher);
    hasher.finish()
}
