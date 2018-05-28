// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use stanza_error::DefinedCondition;

generate_element!(
    A, "a", SM,
    attributes: [
        h: u32 = "h" => required,
    ]
);

impl A {
    pub fn new(h: u32) -> A {
        A { h }
    }
}

generate_attribute!(ResumeAttr, "resume", bool);

generate_element!(
    Enable, "enable", SM,
    attributes: [
        // TODO: should be the infinite integer set ≥ 1.
        max: Option<u32> = "max" => optional,
        resume: ResumeAttr = "resume" => default,
    ]
);

impl Enable {
    pub fn new() -> Self {
        Enable {
            max: None,
            resume: ResumeAttr::False,
        }
    }

    pub fn with_max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn with_resume(mut self) -> Self {
        self.resume = ResumeAttr::True;
        self
    }
}

generate_element!(
    Enabled, "enabled", SM,
    attributes: [
        id: Option<String> = "id" => optional,
        location: Option<String> = "location" => optional,
        // TODO: should be the infinite integer set ≥ 1.
        max: Option<u32> = "max" => optional,
        resume: ResumeAttr = "resume" => default,
    ]
);

generate_element!(
    Failed, "failed", SM,
    attributes: [
        h: Option<u32> = "h" => optional,
    ],
    children: [
        // XXX: implement the * handling.
        error: Option<DefinedCondition> = ("*", XMPP_STANZAS) => DefinedCondition
    ]
);

generate_empty_element!(
    R, "r", SM
);

generate_element!(
    Resume, "resume", SM,
    attributes: [
        h: u32 = "h" => required,
        previd: String = "previd" => required,
    ]
);

generate_element!(
    Resumed, "resumed", SM,
    attributes: [
        h: u32 = "h" => required,
        previd: String = "previd" => required,
    ]
);

// TODO: add support for optional and required.
generate_empty_element!(
    /// Represents availability of Stream Management in `<stream:features/>`.
    StreamManagement, "sm", SM
);

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;

    #[test]
    fn a() {
        let elem: Element = "<a xmlns='urn:xmpp:sm:3' h='5'".parse().unwrap();
        let a = A::try_from(elem).unwrap();
        assert_eq!(a.h, 5);
    }

    #[test]
    fn stream_feature() {
        let elem: Element = "<sm xmlns='urn:xmpp:sm:3'/>".parse().unwrap();
        StreamManagement::try_from(elem).unwrap();
    }
}
