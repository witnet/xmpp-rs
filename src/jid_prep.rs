// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqGetPayload, IqResultPayload};
use crate::util::helpers::{PlainText, JidCodec};
use jid::Jid;

generate_element!(
    /// TODO
    JidPrepQuery, "jid", JID_PREP,
    text: (
        /// TODO
        data: PlainText<Option<String>>
    )
);

impl IqGetPayload for JidPrepQuery {}

generate_element!(
    /// TODO
    JidPrepResponse, "jid", JID_PREP,
    text: (
        /// TODO
        jid: JidCodec<Jid>
    )
);

impl IqResultPayload for JidPrepResponse {}

#[cfg(test)]
mod tests {
    use super::*;
    use minidom::Element;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_size() {
        assert_size!(JidPrepQuery, 24);
        assert_size!(JidPrepResponse, 80);
    }

    #[test]
    fn simple() {
        let elem: Element = "<jid xmlns='urn:xmpp:jidprep:0'>ROMeo@montague.lit/orchard</jid>".parse().unwrap();
        let query = JidPrepQuery::try_from(elem).unwrap();
        assert_eq!(query.data.unwrap(), "ROMeo@montague.lit/orchard");

        let elem: Element = "<jid xmlns='urn:xmpp:jidprep:0'>romeo@montague.lit/orchard</jid>".parse().unwrap();
        let response = JidPrepResponse::try_from(elem).unwrap();
        assert_eq!(response.jid, Jid::from_str("romeo@montague.lit/orchard").unwrap());
    }
}
