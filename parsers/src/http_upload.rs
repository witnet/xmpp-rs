// Copyright (c) 2021 Maxime “pep” Buquet <pep@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqGetPayload, IqResultPayload};
use crate::util::helpers::Text;

generate_element!(
    /// Requesting a slot
    SlotRequest, "request", HTTP_UPLOAD,
    attributes: [
        /// The filename to be uploaded.
        filename: Required<String> = "filename",

        /// Size of the file to be uploaded.
        size: Required<u64> = "size",

        /// Content-Type of the file.
        content_type: Option<String> = "content-type",
    ]
);

impl IqGetPayload for SlotRequest {}

generate_element!(
    /// Slot header
    Header, "header", HTTP_UPLOAD,
    attributes: [
        /// Name of the header
        name: Required<String> = "name"
    ],
    text: (
        /// Content of the header
        data: Text<String>
    )
);

generate_element!(
    /// Put URL
    Put, "put", HTTP_UPLOAD,
    attributes: [
        /// URL
        url: Required<String> = "url",
    ],
    children: [
        /// Header list
        headers: Vec<Header> = ("header", HTTP_UPLOAD) => Header
    ]
);

generate_element!(
    /// Get URL
    Get, "get", HTTP_UPLOAD,
    attributes: [
        /// URL
        url: Required<String> = "url",
    ]
);

generate_element!(
    /// Requesting a slot
    SlotResult, "slot", HTTP_UPLOAD,
    children: [
        /// Put URL and headers
        put: Required<Put> = ("put", HTTP_UPLOAD) => Put,
        /// Get URL
        get: Required<Get> = ("get", HTTP_UPLOAD) => Get
    ]
);

impl IqResultPayload for SlotResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[test]
    fn test_slot_request() {
        let elem: Element = "<request xmlns='urn:xmpp:http:upload:0'
            filename='très cool.jpg'
            size='23456'
            content-type='image/jpeg' />"
            .parse()
            .unwrap();
        let slot = SlotRequest::try_from(elem).unwrap();
        assert_eq!(slot.filename, String::from("très cool.jpg"));
        assert_eq!(slot.size, 23456);
        assert_eq!(slot.content_type, Some(String::from("image/jpeg")));
    }

    #[test]
    fn test_slot_result() {
        let elem: Element = "<slot xmlns='urn:xmpp:http:upload:0'>
            <put url='https://upload.montague.tld/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/tr%C3%A8s%20cool.jpg'>
              <header name='Authorization'>Basic Base64String==</header>
              <header name='Cookie'>foo=bar; user=romeo</header>
            </put>
            <get url='https://download.montague.tld/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/tr%C3%A8s%20cool.jpg' />
          </slot>"
            .parse()
            .unwrap();
        let slot = SlotResult::try_from(elem).unwrap();
        assert_eq!(slot.put.url, String::from("https://upload.montague.tld/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/tr%C3%A8s%20cool.jpg"));
        assert_eq!(slot.put.headers[0].name, String::from("Authorization"));
        assert_eq!(
            slot.put.headers[0].data,
            String::from("Basic Base64String==")
        );
        assert_eq!(slot.put.headers[1].name, String::from("Cookie"));
        assert_eq!(
            slot.put.headers[1].data,
            String::from("foo=bar; user=romeo")
        );
        assert_eq!(slot.get.url, String::from("https://download.montague.tld/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/tr%C3%A8s%20cool.jpg"));
    }
}
