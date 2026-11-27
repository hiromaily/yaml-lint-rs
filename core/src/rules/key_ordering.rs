//! Ordering of scalar keys in mappings.

use super::{
    LintContext, Rule,
    tokens::{Patterns, is_property, problem, scan},
};
use crate::yaml_rust2::scanner::{Token, TokenType::*};
use crate::{LintError, Result, problem::LintProblem};
use serde::Deserialize;
use std::cmp::Ordering;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod locale {
    use super::*;
    use std::{ffi::CString, ptr::NonNull, sync::Arc};

    unsafe extern "C" {
        fn wcscoll_l(
            left: *const libc::wchar_t,
            right: *const libc::wchar_t,
            locale: libc::locale_t,
        ) -> libc::c_int;
    }

    #[derive(Debug, Clone)]
    pub(super) struct Collation {
        name: String,
        handle: Arc<Locale>,
    }

    #[derive(Debug)]
    struct Locale(NonNull<libc::c_void>);

    // The handle is immutable after creation and is freed only when its last
    // owner is dropped. POSIX locale-specific comparisons can run concurrently.
    unsafe impl Send for Locale {}
    unsafe impl Sync for Locale {}

    impl Drop for Locale {
        fn drop(&mut self) {
            // SAFETY: The handle is owned by this value and came from newlocale.
            unsafe { libc::freelocale(self.0.as_ptr().cast()) };
        }
    }

    impl Collation {
        pub(super) fn new(name: &str) -> Result<Self> {
            let c_name = CString::new(name)
                .map_err(|_| LintError::ConfigError("locale cannot contain NUL".into()))?;
            // SAFETY: The locale name is NUL-terminated and no base locale is modified.
            let handle = unsafe {
                libc::newlocale(
                    libc::LC_COLLATE_MASK | libc::LC_CTYPE_MASK,
                    c_name.as_ptr(),
                    std::ptr::null_mut(),
                )
            };
            let handle = NonNull::new(handle.cast())
                .ok_or_else(|| LintError::ConfigError(format!("unsupported locale {name:?}")))?;
            Ok(Self {
                name: name.into(),
                handle: Arc::new(Locale(handle)),
            })
        }

        pub(super) fn compare(&self, left: &str, right: &str) -> Ordering {
            if left.contains('\0') || right.contains('\0') {
                return left.cmp(right);
            }
            let wide = |key: &str| -> Vec<libc::wchar_t> {
                key.chars()
                    .map(|ch| ch as libc::wchar_t)
                    .chain([0])
                    .collect()
            };
            let left = wide(left);
            let right = wide(right);
            // SAFETY: Both wide strings are NUL-terminated and the locale is live.
            unsafe { wcscoll_l(left.as_ptr(), right.as_ptr(), self.handle.0.as_ptr().cast()) }
                .cmp(&0)
        }
    }

    impl PartialEq for Collation {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }
}

/// Enforces ordering of scalar mapping keys, using Unicode code point order by default.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct KeyOrderingRule {
    ignored_keys: Patterns,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[serde(skip)]
    collation: Option<locale::Collation>,
}

impl KeyOrderingRule {
    pub(crate) fn with_locale(self, name: &str) -> Result<Self> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            Ok(Self {
                collation: Some(locale::Collation::new(name)?),
                ..self
            })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(LintError::ConfigError(format!(
                "locale {name:?} is unsupported on this platform"
            )))
        }
    }

    fn compare(&self, left: &str, right: &str) -> Ordering {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if let Some(collation) = &self.collation {
            return collation.compare(left, right);
        }
        left.cmp(right)
    }
}

impl Rule for KeyOrderingRule {
    fn name(&self) -> &'static str {
        "key-ordering"
    }

    fn check(&self, context: &LintContext) -> Vec<LintProblem> {
        let tokens = scan(&context.content);
        let mut collections = Vec::new();
        let mut problems = Vec::new();
        for (index, Token(_, token)) in tokens.iter().enumerate() {
            match token {
                BlockMappingStart | FlowMappingStart => collections.push(Some(None)),
                BlockSequenceStart | FlowSequenceStart => collections.push(None),
                BlockEnd | FlowMappingEnd | FlowSequenceEnd => {
                    collections.pop();
                }
                Key => {
                    let Some(Some(previous)) = collections.last_mut() else {
                        continue;
                    };
                    let Some(Token(mark, Scalar(_, key))) =
                        tokens[index + 1..].iter().find(|token| !is_property(token))
                    else {
                        continue;
                    };
                    if self.ignored_keys.matches(key) {
                        continue;
                    }
                    if previous.is_some_and(|previous: &str| {
                        self.compare(previous, key) == Ordering::Greater
                    }) {
                        problems.push(problem(
                            *mark,
                            self.name(),
                            format!("wrong ordering of key \"{key}\" in mapping"),
                        ));
                    } else {
                        *previous = Some(key.as_str());
                    }
                }
                _ => {}
            }
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_scalar_keys_with_node_properties() {
        let content = "z: 1\n&anchor a: 2\n!!str b: 3\n";
        let problems = KeyOrderingRule::default().check(&LintContext::new(content.into()));
        let locations: Vec<_> = problems.iter().map(|p| (p.line, p.column)).collect();
        assert_eq!(locations, [(2, 9), (3, 7)]);
    }
}
