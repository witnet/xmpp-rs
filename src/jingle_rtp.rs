// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_element!(
    /// Wrapper element describing an RTP session.
    Description, "description", JINGLE_RTP,
    attributes: [
        /// Namespace of the encryption scheme used.
        media: Required<String> = "media",

        /// User-friendly name for the encryption scheme, should be `None` for OTR,
        /// legacy OpenPGP and OX.
        // XXX: is this a String or an u32?!  Refer to RFC 3550.
        ssrc: Option<String> = "ssrc",
    ],
    children: [
        /// List of encodings that can be used for this RTP stream.
        payload_types: Vec<PayloadType> = ("payload-type", JINGLE_RTP) => PayloadType

        // TODO: Add support for <encryption/> and <bandwidth/>.
    ]
);

generate_attribute!(
    /// The number of channels.
    Channels, "channels", u8, Default = 1
);

generate_element!(
    /// An encoding that can be used for an RTP stream.
    PayloadType, "payload-type", JINGLE_RTP,
    attributes: [
        /// The number of channels.
        channels: Default<Channels> = "channels",

        /// The sampling frequency in Hertz.
        clockrate: Option<u32> = "clockrate",

        /// The payload identifier.
        id: Required<u8> = "id",

        /// Maximum packet time as specified in RFC 4566.
        maxptime: Option<u32> = "maxptime",

        /// The appropriate subtype of the MIME type.
        name: Option<String> = "name",

        /// Packet time as specified in RFC 4566.
        ptime: Option<u32> = "ptime",
    ],
    children: [
        /// List of parameters specifying this payload-type.
        ///
        /// Their order MUST be ignored.
        parameters: Vec<Parameter> = ("parameter", JINGLE_RTP) => Parameter
    ]
);

generate_element!(
    /// Parameter related to a payload.
    Parameter, "parameter", JINGLE_RTP,
    attributes: [
        /// The name of the parameter, from the list at
        /// https://www.iana.org/assignments/sdp-parameters/sdp-parameters.xhtml
        name: Required<String> = "name",

        /// The value of this parameter.
        value: Required<String> = "value",
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Description, 36);
        assert_size!(Channels, 1);
        assert_size!(PayloadType, 52);
        assert_size!(Parameter, 24);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Description, 72);
        assert_size!(Channels, 1);
        assert_size!(PayloadType, 80);
        assert_size!(Parameter, 48);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "
<description xmlns='urn:xmpp:jingle:apps:rtp:1' media='audio'>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' channels='2' clockrate='48000' id='96' name='OPUS'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' channels='1' clockrate='32000' id='105' name='SPEEX'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' channels='1' clockrate='8000' id='9' name='G722'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' channels='1' clockrate='16000' id='106' name='SPEEX'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' clockrate='8000' id='8' name='PCMA'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' clockrate='8000' id='0' name='PCMU'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' channels='1' clockrate='8000' id='107' name='SPEEX'/>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' channels='1' clockrate='8000' id='99' name='AMR'>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='octet-align' value='1'/>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='crc' value='0'/>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='robust-sorting' value='0'/>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='interleaving' value='0'/>
    </payload-type>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' clockrate='48000' id='100' name='telephone-event'>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='events' value='0-15'/>
    </payload-type>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' clockrate='16000' id='101' name='telephone-event'>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='events' value='0-15'/>
    </payload-type>
    <payload-type xmlns='urn:xmpp:jingle:apps:rtp:1' clockrate='8000' id='102' name='telephone-event'>
        <parameter xmlns='urn:xmpp:jingle:apps:rtp:1' name='events' value='0-15'/>
    </payload-type>
</description>"
                .parse()
                .unwrap();
        let desc = Description::try_from(elem).unwrap();
        assert_eq!(desc.media, "audio");
        assert_eq!(desc.ssrc, None);
    }
}
