// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;
use std::net::IpAddr;

use minidom::Element;
use jid::Jid;

use error::Error;

use ns;

generate_attribute!(Type, "type", {
    Assisted => "assisted",
    Direct => "direct",
    Proxy => "proxy",
    Tunnel => "tunnel",
}, Default = Direct);

generate_attribute!(Mode, "mode", {
    Tcp => "tcp",
    Udp => "udp",
}, Default = Tcp);

generate_id!(CandidateId);

generate_id!(StreamId);

generate_element_with_only_attributes!(Candidate, "candidate", ns::JINGLE_S5B, [
    cid: CandidateId = "cid" => required,
    host: IpAddr = "host" => required,
    jid: Jid = "jid" => required,
    port: Option<u16> = "port" => optional,
    priority: u32 = "priority" => required,
    type_: Type = "type" => default,
]);

impl Candidate {
    pub fn new(cid: CandidateId, host: IpAddr, jid: Jid, priority: u32) -> Candidate {
        Candidate {
            cid,
            host,
            jid,
            priority,
            port: Default::default(),
            type_: Default::default(),
        }
    }

    pub fn with_port(mut self, port: u16) -> Candidate {
        self.port = Some(port);
        self
    }

    pub fn with_type(mut self, type_: Type) -> Candidate {
        self.type_ = type_;
        self
    }
}

#[derive(Debug, Clone)]
pub enum TransportPayload {
    Activated(String),
    Candidates(Vec<Candidate>),
    CandidateError,
    CandidateUsed(String),
    ProxyError,
    None,
}

#[derive(Debug, Clone)]
pub struct Transport {
    pub sid: StreamId,
    pub dstaddr: Option<String>,
    pub mode: Mode,
    pub payload: TransportPayload,
}

impl Transport {
    pub fn new(sid: StreamId) -> Transport {
        Transport {
            sid,
            dstaddr: None,
            mode: Default::default(),
            payload: TransportPayload::None,
        }
    }

    pub fn with_dstaddr(mut self, dstaddr: String) -> Transport {
        self.dstaddr = Some(dstaddr);
        self
    }

    pub fn with_mode(mut self, mode: Mode) -> Transport {
        self.mode = mode;
        self
    }

    pub fn with_payload(mut self, payload: TransportPayload) -> Transport {
        self.payload = payload;
        self
    }
}

impl TryFrom<Element> for Transport {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Transport, Error> {
        check_self!(elem, "transport", ns::JINGLE_S5B);
        let sid = get_attr!(elem, "sid", required);
        let dstaddr = get_attr!(elem, "dstaddr", optional);
        let mode = get_attr!(elem, "mode", default);

        let mut payload = None;
        for child in elem.children() {
            payload = Some(if child.is("candidate", ns::JINGLE_S5B) {
                let mut candidates = match payload {
                    Some(TransportPayload::Candidates(candidates)) => candidates,
                    Some(_) => return Err(Error::ParseError("Non-candidate child already present in JingleS5B transport element.")),
                    None => vec!(),
                };
                candidates.push(Candidate::try_from(child.clone())?);
                TransportPayload::Candidates(candidates)
            } else if child.is("activated", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError("Non-activated child already present in JingleS5B transport element."));
                }
                let cid = get_attr!(child, "cid", required);
                TransportPayload::Activated(cid)
            } else if child.is("candidate-error", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError("Non-candidate-error child already present in JingleS5B transport element."));
                }
                TransportPayload::CandidateError
            } else if child.is("candidate-used", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError("Non-candidate-used child already present in JingleS5B transport element."));
                }
                let cid = get_attr!(child, "cid", required);
                TransportPayload::CandidateUsed(cid)
            } else if child.is("proxy-error", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError("Non-proxy-error child already present in JingleS5B transport element."));
                }
                TransportPayload::ProxyError
            } else {
                return Err(Error::ParseError("Unknown child in JingleS5B transport element."));
            });
        }
        let payload = payload.unwrap_or(TransportPayload::None);
        Ok(Transport {
            sid: sid,
            dstaddr: dstaddr,
            mode: mode,
            payload: payload,
        })
    }
}

impl From<Transport> for Element {
    fn from(transport: Transport) -> Element {
        Element::builder("transport")
                .ns(ns::JINGLE_S5B)
                .attr("sid", transport.sid)
                .attr("dstaddr", transport.dstaddr)
                .attr("mode", transport.mode)
                .append(match transport.payload {
                     TransportPayload::Candidates(candidates) => {
                         candidates.into_iter()
                                   .map(Element::from)
                                   .collect::<Vec<_>>()
                     },
                     TransportPayload::Activated(cid) => {
                         vec!(Element::builder("activated")
                                      .ns(ns::JINGLE_S5B)
                                      .attr("cid", cid)
                                      .build())
                     },
                     TransportPayload::CandidateError => {
                         vec!(Element::builder("candidate-error")
                                      .ns(ns::JINGLE_S5B)
                                      .build())
                     },
                     TransportPayload::CandidateUsed(cid) => {
                         vec!(Element::builder("candidate-used")
                                      .ns(ns::JINGLE_S5B)
                                      .attr("cid", cid)
                                      .build())
                     },
                     TransportPayload::ProxyError => {
                         vec!(Element::builder("proxy-error")
                                      .ns(ns::JINGLE_S5B)
                                      .build())
                     },
                     TransportPayload::None => vec!(),
                 })
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use compare_elements::NamespaceAwareCompare;

    #[test]
    fn test_simple() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:s5b:1' sid='coucou'/>".parse().unwrap();
        let transport = Transport::try_from(elem).unwrap();
        assert_eq!(transport.sid, StreamId(String::from("coucou")));
        assert_eq!(transport.dstaddr, None);
        assert_eq!(transport.mode, Mode::Tcp);
        match transport.payload {
            TransportPayload::None => (),
            _ => panic!("Wrong element inside transport!"),
        }
    }

    #[test]
    fn test_serialise_activated() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:s5b:1' sid='coucou'><activated cid='coucou'/></transport>".parse().unwrap();
        let transport = Transport {
            sid: StreamId(String::from("coucou")),
            dstaddr: None,
            mode: Mode::Tcp,
            payload: TransportPayload::Activated(String::from("coucou")),
        };
        let elem2: Element = transport.into();
        assert!(elem.compare_to(&elem2));
    }

    #[test]
    fn test_serialise_candidate() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:s5b:1' sid='coucou'><candidate cid='coucou' host='127.0.0.1' jid='coucou@coucou' priority='0'/></transport>".parse().unwrap();
        let transport = Transport {
            sid: StreamId(String::from("coucou")),
            dstaddr: None,
            mode: Mode::Tcp,
            payload: TransportPayload::Candidates(vec!(Candidate {
                cid: CandidateId(String::from("coucou")),
                host: IpAddr::from_str("127.0.0.1").unwrap(),
                jid: Jid::from_str("coucou@coucou").unwrap(),
                port: None,
                priority: 0u32,
                type_: Type::Direct,
            })),
        };
        let elem2: Element = transport.into();
        assert!(elem.compare_to(&elem2));
    }
}
