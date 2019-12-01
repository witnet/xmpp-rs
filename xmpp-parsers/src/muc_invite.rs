// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;
use jid::BareJid;

generate_attribute!(
    /// Whether this invitation continues a one-to-one chat.
    Continue,
    "continue",
    bool
);

generate_element!(
    /// Notes when and by whom a message got stored for later delivery.
    Invite, "x", MUC_INVITATION,
    attributes: [
        /// Whether this invitation continues a one-to-one chat.
        continue_: Default<Continue> = "continue",

        /// The address of the groupchat room to be joined.
        jid: Required<BareJid> = "jid",

        /// Password needed for entry into a password-protected room.
        password: Option<String> = "password",

        /// Human-readable purpose for the invitation.
        reason: Option<String> = "reason",

        /// When continuing a one-to-one chat, the thread to continue from.
        // TODO: unify that with message’s Thread struct.
        thread: Option<String> = "thread"
    ]
);

impl MessagePayload for Invite {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Continue, 1);
        assert_size!(Invite, 64);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Continue, 1);
        assert_size!(Invite, 128);
    }

    #[test]
    fn simple() {
        let elem: Element = "<x xmlns='jabber:x:conference'
                jid='darkcave@macbeth.shakespeare.lit'/>"
            .parse()
            .unwrap();
        let invite = Invite::try_from(elem).unwrap();
        assert_eq!(
            invite.jid,
            BareJid::new("darkcave", "macbeth.shakespeare.lit")
        );
        assert_eq!(invite.password, None);
        assert_eq!(invite.reason, None);
        assert_eq!(invite.continue_, Continue::False);
        assert_eq!(invite.thread, None);
    }

    #[test]
    fn example_1() {
        let elem: Element = "<x xmlns='jabber:x:conference'
                jid='darkcave@macbeth.shakespeare.lit'
                password='cauldronburn'
                reason='Hey Hecate, this is the place for all good witches!'/>"
            .parse()
            .unwrap();
        let invite = Invite::try_from(elem).unwrap();
        assert_eq!(
            invite.jid,
            BareJid::new("darkcave", "macbeth.shakespeare.lit")
        );
        assert_eq!(invite.password, Some(String::from("cauldronburn")));
        assert_eq!(
            invite.reason,
            Some(String::from(
                "Hey Hecate, this is the place for all good witches!"
            ))
        );
        assert_eq!(invite.continue_, Continue::False);
        assert_eq!(invite.thread, None);
    }

    #[test]
    fn example_2() {
        let elem: Element = "<x xmlns='jabber:x:conference'
                continue='true'
                jid='darkcave@macbeth.shakespeare.lit'
                password='cauldronburn'
                reason='Hey Hecate, this is the place for all good witches!'
                thread='e0ffe42b28561960c6b12b944a092794b9683a38'/>"
            .parse()
            .unwrap();
        let invite = Invite::try_from(elem).unwrap();
        assert_eq!(
            invite.jid,
            BareJid::new("darkcave", "macbeth.shakespeare.lit")
        );
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>"
            .parse()
            .unwrap();
        let error = Invite::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a x element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<x xmlns='jabber:x:conference'><coucou/></x>"
            .parse()
            .unwrap();
        let error = Invite::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element =
            "<x xmlns='jabber:x:conference' jid='darkcave@macbeth.shakespeare.lit'/>"
                .parse()
                .unwrap();
        let invite = Invite {
            jid: BareJid::new("darkcave", "macbeth.shakespeare.lit"),
            password: None,
            reason: None,
            continue_: Continue::False,
            thread: None,
        };
        let elem2 = invite.into();
        assert_eq!(elem, elem2);
    }
}
