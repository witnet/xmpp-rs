// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::net::IpAddr;

generate_element!(
    /// Wrapper element for an ICE-UDP transport.
    Transport, "transport", JINGLE_ICE_UDP,
    attributes: [
        /// A Password as defined in ICE-CORE.
        pwd: Option<String> = "pwd",

        /// A User Fragment as defined in ICE-CORE.
        ufrag: Option<String> = "ufrag",
    ],
    children: [
	/// List of candidates for this ICE-UDP session.
        candidates: Vec<Candidate> = ("candidate", JINGLE_ICE_UDP) => Candidate
    ]
);

generate_attribute!(
    /// A Candidate Type as defined in ICE-CORE.
    Type, "type", {
        /// Host candidate.
        Host => "host",

        /// Peer reflexive candidate.
        Prflx => "prflx",

        /// Relayed candidate.
        Relay => "relay",

        /// Server reflexive candidate.
        Srflx => "srflx",
    }
);

generate_element!(
    /// A candidate for an ICE-UDP session.
    Candidate, "candidate", JINGLE_ICE_UDP,
    attributes: [
        /// A Component ID as defined in ICE-CORE.
        component: Required<u8> = "component",

        /// A Foundation as defined in ICE-CORE.
        foundation: Required<u8> = "foundation",

        /// An index, starting at 0, that enables the parties to keep track of updates to the
        /// candidate throughout the life of the session.
        generation: Required<u8> = "generation",

        /// A unique identifier for the candidate.
        id: Required<String> = "id",

        /// The Internet Protocol (IP) address for the candidate transport mechanism; this can be
        /// either an IPv4 address or an IPv6 address.
        ip: Required<IpAddr> = "ip",

        /// An index, starting at 0, referencing which network this candidate is on for a given
        /// peer.
        network: Required<u8> = "network",

        /// The port at the candidate IP address.
        port: Required<u16> = "port",

        /// A Priority as defined in ICE-CORE.
        priority: Required<u32> = "priority",

        /// The protocol to be used. The only value defined by this specification is "udp".
        protocol: Required<String> = "protocol",

        /// A related address as defined in ICE-CORE.
        rel_addr: Option<String> = "rel-addr",

        /// A related port as defined in ICE-CORE.
        rel_port: Option<u16> = "rel-port",

        /// A Candidate Type as defined in ICE-CORE.
        type_: Required<Type> = "type",
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use minidom::Element;
    use try_from::TryFrom;

    #[test]
    fn test_simple() {
        let elem: Element = "
<transport xmlns='urn:xmpp:jingle:transports:ice-udp:1' pwd='wakMJ8Ydd5rqnPaFerws5o' ufrag='aeXX'>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='11b72719-6a1b-4c51-8ae6-9f1538047568' ip='192.168.0.12' network='0' port='56715' priority='1010828030' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='7e07b22d-db50-4e17-9ed9-eafeb96f4f63' ip='192.168.0.12' network='0' port='0' priority='1015022334' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='431de362-c45f-40a8-bf10-9ed898a71d86' ip='192.168.0.12' network='0' port='36480' priority='2013266428' protocol='udp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='b1197df3-abca-413b-99ee-3660d91bcfa7' ip='192.168.0.12' network='0' port='50387' priority='1010828031' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='adaf3a85-3a57-4df0-a2d8-0c7d28d3ca01' ip='192.168.0.12' network='0' port='0' priority='1015022335' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='ef4e0a62-81f2-4fe3-87ae-46cb5d1d1e1d' ip='192.168.0.12' network='0' port='43132' priority='2013266429' protocol='udp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='51891e8a-4c1e-4540-b173-8637aeb0143c' ip='fe80::24eb:646f:7d78:cb6' network='0' port='38881' priority='2013266431' protocol='udp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='73f82655-eb84-4fa1-b05c-1ea76f695d32' ip='fe80::24eb:646f:7d78:cb6' network='0' port='0' priority='1015023103' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='a2a8fa62-6f2e-41e8-b218-ba095540d60f' ip='fe80::24eb:646f:7d78:cb6' network='0' port='55819' priority='1010828799' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='23e66735-9515-414c-81ad-2455569a57f8' ip='2a01:e35:2e2f:fbb0:43aa:33b5:5535:8905' network='0' port='39967' priority='2013266430' protocol='udp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='9a8dff18-e138-4fb2-a956-89d71216da84' ip='2a01:e35:2e2f:fbb0:43aa:33b5:5535:8905' network='0' port='0' priority='1015022079' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='1' foundation='1' generation='0' id='f0c73ac3-9b7d-4032-abe3-6dd9a57d0f03' ip='2a01:e35:2e2f:fbb0:43aa:33b5:5535:8905' network='0' port='37487' priority='1010827775' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='a6199a00-34df-46f5-a608-847b75c5250e' ip='fe80::24eb:646f:7d78:cb6' network='0' port='43521' priority='2013266430' protocol='udp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='83bc2600-39ce-4c9e-8b0b-cc7aa7e6a293' ip='fe80::24eb:646f:7d78:cb6' network='0' port='0' priority='1015023102' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='7e3606ca-46de-4de8-8802-068dd69ef01a' ip='fe80::24eb:646f:7d78:cb6' network='0' port='52279' priority='1010828798' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='a7c2472a-8462-412c-a64c-d3528f0abfa4' ip='2a01:e35:2e2f:fbb0:43aa:33b5:5535:8905' network='0' port='34088' priority='2013266429' protocol='udp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='5a12c345-9643-4d2c-b770-695ec6affcaf' ip='2a01:e35:2e2f:fbb0:43aa:33b5:5535:8905' network='0' port='0' priority='1015022078' protocol='tcp' type='host'/>
    <candidate xmlns='urn:xmpp:jingle:transports:ice-udp:1' component='2' foundation='1' generation='0' id='67f65b0b-8cee-421a-9f37-1f2ca2211c87' ip='2a01:e35:2e2f:fbb0:43aa:33b5:5535:8905' network='0' port='39431' priority='1010827774' protocol='tcp' type='host'/>
</transport>"
                .parse()
                .unwrap();
        let transport = Transport::try_from(elem).unwrap();
        assert_eq!(transport.pwd.unwrap(), "wakMJ8Ydd5rqnPaFerws5o");
        assert_eq!(transport.ufrag.unwrap(), "aeXX");
    }
}
