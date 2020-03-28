// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2020 Astro <astro@spaceboyz.net>
// Copyright (c) 2020 Maxime “pep” Buquet <pep@bouah.net>
// Copyright (c) 2020 Xidorn Quan <me@upsuper.org>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Prefixes {
    prefixes: BTreeMap<String, Option<String>>,
}

impl Default for Prefixes {
    fn default() -> Self {
        Prefixes {
            prefixes: BTreeMap::new(),
        }
    }
}

impl fmt::Debug for Prefixes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prefixes(")?;
        for (namespace, prefix) in &self.prefixes {
            write!(
                f,
                "xmlns{}={:?} ",
                match prefix {
                    None => String::new(),
                    Some(prefix) => format!(":{}", prefix),
                },
                namespace
            )?;
        }
        write!(f, ")")
    }
}

impl Prefixes {
    pub fn declared_prefixes(&self) -> &BTreeMap<String, Option<String>> {
        &self.prefixes
    }

    pub fn get(&self, namespace: &String) -> Option<Option<String>> {
        match self.prefixes.get(namespace) {
            Some(ns) => Some(ns.clone()),
            None => None,
        }
    }

    pub(crate) fn insert<S: Into<String>>(&mut self, namespace: S, prefix: Option<String>) {
        self.prefixes.insert(namespace.into(), prefix);
    }
}

impl From<BTreeMap<String, Option<String>>> for Prefixes {
    fn from(prefixes: BTreeMap<String, Option<String>>) -> Self {
        Prefixes { prefixes }
    }
}

impl From<Option<String>> for Prefixes {
    fn from(namespace: Option<String>) -> Self {
        match namespace {
            None => Self::default(),
            Some(namespace) => Self::from(namespace),
        }
    }
}

impl From<String> for Prefixes {
    fn from(namespace: String) -> Self {
        let mut prefixes = BTreeMap::new();
        prefixes.insert(namespace, None);

        Prefixes { prefixes }
    }
}

impl From<(Option<String>, String)> for Prefixes {
    fn from(prefix_namespace: (Option<String>, String)) -> Self {
        let (prefix, namespace) = prefix_namespace;
        let mut prefixes = BTreeMap::new();
        prefixes.insert(namespace, prefix);

        Prefixes { prefixes }
    }
}

impl From<(String, String)> for Prefixes {
    fn from(prefix_namespace: (String, String)) -> Self {
        let (prefix, namespace) = prefix_namespace;
        Self::from((Some(prefix), namespace))
    }
}
