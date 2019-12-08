// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use xmpp_parsers::jingle::{Action, Content, ContentId, Creator, Jingle, SessionId};
use xmpp_parsers::jingle_dtls_srtp::{Fingerprint, Setup};
use xmpp_parsers::jingle_grouping::{Content as GroupContent, Group as Grouping};
use xmpp_parsers::jingle_ice_udp::{Candidate, Transport, Type};
use xmpp_parsers::jingle_rtcp_fb::RtcpFb;
use xmpp_parsers::jingle_rtp::{Description, Parameter as RtpParameter, PayloadType, RtcpMux};
use xmpp_parsers::jingle_rtp_hdrext::RtpHdrext;
use xmpp_parsers::jingle_ssma::{Group, Parameter, Source};

lazy_static::lazy_static! {
    /// List of static payload types.
    ///
    /// From https://tools.ietf.org/html/rfc3551#section-6
    static ref STATIC_PAYLOADS: [PayloadType; 35] = [
        PayloadType::new(0, String::from("PCMU"), 8000, 1),
        PayloadType::without_clockrate(1, String::from("reserved")),
        PayloadType::without_clockrate(2, String::from("reserved")),
        PayloadType::new(3, String::from("GSM"), 8000, 1),
        PayloadType::new(4, String::from("G723"), 8000, 1),
        PayloadType::new(5, String::from("DVI4"), 8000, 1),
        PayloadType::new(6, String::from("DVI4"), 16000, 1),
        PayloadType::new(7, String::from("LPC"), 8000, 1),
        PayloadType::new(8, String::from("PCMA"), 8000, 1),
        PayloadType::new(9, String::from("G722"), 8000, 1),
        PayloadType::new(10, String::from("L16"), 44100, 2),
        PayloadType::new(11, String::from("L16"), 44100, 1),
        PayloadType::new(12, String::from("QCELP"), 8000, 1),
        PayloadType::new(13, String::from("CN"), 8000, 1),
        PayloadType::new(14, String::from("MPA"), 90000, 0), // XXX: (see text)
        PayloadType::new(15, String::from("G728"), 8000, 1),
        PayloadType::new(16, String::from("DVI4"), 11025, 1),
        PayloadType::new(17, String::from("DVI4"), 22050, 1),
        PayloadType::new(18, String::from("G729"), 8000, 1),
        PayloadType::without_clockrate(19, String::from("reserved")),
        PayloadType::without_clockrate(20, String::from("unassigned")),
        PayloadType::without_clockrate(21, String::from("unassigned")),
        PayloadType::without_clockrate(22, String::from("unassigned")),
        PayloadType::without_clockrate(23, String::from("unassigned")),
        PayloadType::without_clockrate(24, String::from("unassigned")),
        PayloadType::new(25, String::from("CelB"), 90000, 1),
        PayloadType::new(26, String::from("JPEG"), 90000, 1),
        PayloadType::without_clockrate(27, String::from("unassigned")),
        PayloadType::new(28, String::from("nv"), 90000, 1),
        PayloadType::without_clockrate(29, String::from("unassigned")),
        PayloadType::without_clockrate(30, String::from("unassigned")),
        PayloadType::new(31, String::from("H261"), 90000, 1),
        PayloadType::new(32, String::from("MPV"), 90000, 1),
        PayloadType::new(33, String::from("MP2T"), 90000, 1),
        PayloadType::new(34, String::from("H263"), 90000, 1),
    ];
}

#[derive(Debug)]
pub enum Error {
    WrongNumberOfArguments,
    TooManyFingerprints,
}

#[derive(Debug)]
enum Sdp {
    V(u8),
    O(SessionId, u8),
    S(String),
    T(u32, u32),
    M(String, Vec<u8>),
    C,
    A(String, Option<String>),
}

impl FromStr for Sdp {
    type Err = ();

    fn from_str(s: &str) -> Result<Sdp, ()> {
        let b = s.as_bytes();
        assert_eq!(b[1], b'=');
        Ok(match b[0] {
            b'v' => {
                let version = s[2..].parse().unwrap();
                assert_eq!(version, 0);
                Sdp::V(version)
            }
            b'o' => {
                if let [hyphen, sid, nb_contents, localhost] =
                    s[2..].splitn(4, ' ').collect::<Vec<_>>().as_slice()
                {
                    assert_eq!(hyphen, &"-");
                    assert_eq!(localhost, &"IN IP4 127.0.0.1");
                    let nb_contents = nb_contents.parse().unwrap();
                    Sdp::O(SessionId(sid.to_string()), nb_contents)
                } else {
                    //return Err(Error::WrongNumberOfArguments);
                    todo!();
                }
            }
            b's' => Sdp::S(s[2..].to_string()),
            b't' => Sdp::T(0, 0),
            b'm' => {
                let mut iter = s[2..].split(' ');
                let type_ = iter.next().unwrap().to_string();
                let port: u16 = iter.next().unwrap().parse().unwrap();
                let kind = iter.next().unwrap();
                assert_eq!(port, 9u16);
                assert_eq!(kind, "UDP/TLS/RTP/SAVPF");
                let codecs = iter.map(|x| x.parse().unwrap()).collect();
                Sdp::M(type_, codecs)
            }
            b'c' => Sdp::C,
            b'a' => {
                if let [key, value] = s[2..].splitn(2, ':').collect::<Vec<_>>().as_slice() {
                    let key = key.to_string();
                    let value = value.to_string();
                    Sdp::A(key, Some(value))
                } else {
                    let key = s[2..].to_string();
                    Sdp::A(key, None)
                }
            }
            _ => todo!(),
        })
    }
}

impl fmt::Display for Sdp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Sdp::V(version) => write!(fmt, "v={}", version),
            Sdp::O(sid, contents) => write!(fmt, "o=- {} {} IN IP4 127.0.0.1", sid.0, contents),
            Sdp::S(s) => write!(fmt, "s={}", s),
            Sdp::T(begin, end) => write!(fmt, "t={} {}", begin, end),
            Sdp::M(content, codecs) => write!(
                fmt,
                "m={} 9 UDP/TLS/RTP/SAVPF {}",
                content,
                codecs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Sdp::C => write!(fmt, "c=IN IP4 0.0.0.0"),
            Sdp::A(key, value) => write!(
                fmt,
                "a={}{}",
                key,
                if let Some(value) = value {
                    format!(":{}", value)
                } else {
                    String::new()
                }
            ),
        }
    }
}

pub fn sdp_to_jingle(sdp: &str) -> Result<Jingle, Error> {
    let mut session_id = None;
    let mut attributes = Vec::new();
    let mut media = Vec::new();

    // This loop is used to associate attributes with their media line, and extract some
    // information we need later.
    {
        let mut current_media = None;
        for line in sdp.lines() {
            let sdp: Sdp = line.parse().unwrap();
            assert_eq!(line, format!("{}", sdp));
            match sdp {
                Sdp::V(version) => assert_eq!(version, 0),
                Sdp::O(sid, _) => {
                    if session_id.is_some() {
                        todo!("Too many o=");
                    }
                    session_id = Some(sid);
                }
                Sdp::S(_) => (),
                Sdp::T(_, _) => (),
                Sdp::M(content, codecs) => {
                    if let Some(cur_media) = current_media {
                        media.push(cur_media);
                    }
                    current_media = Some((content, codecs, Vec::new()));
                    //media.push(current_media);
                }
                Sdp::C => (),
                Sdp::A(key, value) => {
                    if let Some(ref mut media) = current_media {
                        media.2.push((key, value));
                    } else {
                        attributes.push((key, value));
                    }
                }
            }
        }
        if let Some(cur_media) = current_media {
            media.push(cur_media);
        }
    }

    let mut jingle = Jingle::new(Action::SessionInitiate, session_id.unwrap());
    for attribute in attributes {
        match attribute {
            (key, Some(value)) if key == "group" => {
                let mut iter = value.split(' ');
                let semantics = iter.next().unwrap().parse().unwrap();
                let contents = iter.map(GroupContent::new).collect::<Vec<_>>();
                if contents.len() < 2 {
                    println!("Warning: useless grouping of size {}.", contents.len());
                }
                jingle.group = Some(Grouping {
                    semantics,
                    contents,
                });
            }
            (key, Some(value)) => println!("Unknown global attribute {}:{:?}", key, value),
            (key, None) => println!("Unknown global attribute {}", key),
        }
    }

    for media in media {
        let mut content = Content::new(Creator::Initiator, ContentId(media.0.clone()));
        let mut description = Description::new(media.0.clone());
        let mut transport = Transport::new();
        let mut rtp_map: HashMap<u8, PayloadType> = HashMap::new();
        let mut ssrc_map = HashMap::new();
        let mut ssrc_order = Vec::new();
        let mut setup = None;
        for attribute in media.2.iter() {
            let value = attribute.1.clone();
            match attribute.0.as_str() {
                "candidate" => {
                    let value = value.unwrap();
                    let mut values = value.split(' ').collect::<Vec<_>>();
                    let candidate = if let [foundation, component, protocol, priority, ip, port] =
                        values.drain(..6).collect::<Vec<_>>().as_slice()
                    {
                        assert_eq!(values.len() % 2, 0);
                        let mut typ = None;
                        let mut generation = None;
                        // TODO: Support XEP-0371.
                        //let mut tcptype = None;
                        let mut rel_addr = None;
                        let mut rel_port = None;
                        for i in 0..values.len() / 2 {
                            let key = values[i * 2];
                            let value = values[i * 2 + 1];
                            match key {
                                "typ" => typ = Some(value.parse().unwrap()),
                                "generation" => generation = Some(value.parse().unwrap()),
                                "tcptype" => (), // XXX
                                //"tcptype" => tcptype = Some(value.parse().unwrap()),
                                "raddr" => rel_addr = Some(value.parse().unwrap()),
                                "rport" => rel_port = Some(value.parse().unwrap()),
                                _ => unimplemented!("TODO"),
                            }
                        }
                        Candidate {
                            component: component.parse().unwrap(),
                            foundation: foundation.parse().unwrap(),
                            generation: generation.unwrap_or(0), // XXX
                            id: "id".to_string(),                // XXX
                            ip: ip.parse().unwrap(),
                            network: 0, // XXX
                            port: port.parse().unwrap(),
                            priority: priority.parse().unwrap(),
                            protocol: protocol.parse().unwrap(),
                            rel_addr,
                            rel_port,
                            type_: typ.unwrap_or(Type::Host), // XXX
                        }
                    } else {
                        return Err(Error::WrongNumberOfArguments);
                    };
                    transport.candidates.push(candidate);
                }
                "fingerprint" => {
                    if transport.fingerprint.is_some() {
                        return Err(Error::TooManyFingerprints);
                    }
                    let value = value.unwrap();
                    let fingerprint = if let [algo, hash] =
                        value.splitn(2, ' ').collect::<Vec<_>>().as_slice()
                    {
                        Fingerprint::from_colon_separated_hex(Setup::Actpass, algo, hash).unwrap()
                    } else {
                        return Err(Error::WrongNumberOfArguments);
                    };
                    transport.fingerprint = Some(fingerprint);
                }
                "ice-pwd" => {
                    let value = value.unwrap();
                    transport.pwd = Some(value.to_string());
                }
                "ice-ufrag" => {
                    let value = value.unwrap();
                    transport.ufrag = Some(value.to_string());
                }
                "mid" => {
                    let value = value.unwrap();
                    assert_eq!(value, media.0);
                }
                "rtcp-fb" => {
                    let value = value.unwrap();
                    if let [id, type_, subtype] =
                        value.splitn(3, ' ').collect::<Vec<_>>().as_slice()
                    {
                        let id = id.parse().unwrap();
                        let type_ = type_.to_string();
                        let subtype = Some(subtype.to_string());
                        let payload_type = rtp_map.get_mut(&id).unwrap();
                        payload_type.rtcp_fbs.push(RtcpFb { type_, subtype });
                    } else if let [id, type_] = value.splitn(2, ' ').collect::<Vec<_>>().as_slice()
                    {
                        let id = id.parse().unwrap();
                        let type_ = type_.to_string();
                        let payload_type = rtp_map.get_mut(&id).unwrap();
                        payload_type.rtcp_fbs.push(RtcpFb {
                            type_,
                            subtype: None,
                        });
                    } else {
                        panic!("Invalid rtcp-fb value: {}", value);
                    }
                }
                "rtpmap" => {
                    let value = value.unwrap();
                    if let [id, codec] = value.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
                        let id = id.parse().unwrap();
                        let payload_type = if let [codec, clockrate, channels] =
                            codec.splitn(3, '/').collect::<Vec<_>>().as_slice()
                        {
                            let codec = codec.to_string();
                            let clockrate = clockrate.parse().unwrap();
                            let channels = channels.parse().unwrap();
                            PayloadType::new(id, codec, clockrate, channels)
                        } else if let [codec, clockrate] =
                            codec.splitn(2, '/').collect::<Vec<_>>().as_slice()
                        {
                            let codec = codec.to_string();
                            let clockrate = clockrate.parse().unwrap();
                            PayloadType::new(id, codec, clockrate, 1)
                        } else {
                            PayloadType::without_clockrate(id, codec.to_string())
                        };
                        if rtp_map.contains_key(&id) {
                            todo!("Return a proper error since this rtpmap has already been set.");
                        }
                        rtp_map.insert(id, payload_type);
                    }
                }
                "fmtp" => {
                    let value = value.unwrap();
                    if let [id, params] = value.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
                        let id = id.parse().unwrap();
                        for param in params.split(';').map(|x| x.trim()) {
                            if let [name, value] =
                                param.splitn(2, '=').collect::<Vec<_>>().as_slice()
                            {
                                let name = name.to_string();
                                let value = value.to_string();
                                let param = RtpParameter { name, value };
                                let payload_type = rtp_map.get_mut(&id).unwrap();
                                payload_type.parameters.push(param);
                            } else {
                                todo!();
                            }
                        }
                    } else {
                        todo!();
                    }
                }
                "setup" => {
                    let value = value.unwrap();
                    setup = Some(Setup::from_str(&value).unwrap());
                }
                "ssrc-group" => {
                    let value = value.unwrap();
                    let mut values = value.split(' ').collect::<Vec<_>>();
                    let semantics = values.remove(0).to_string();
                    let mut sources = Vec::new();
                    for id in values {
                        let source = Source::new(id.to_string());
                        sources.push(source);
                    }
                    let group = Group { semantics, sources };
                    description.ssrc_groups.push(group);
                }
                "ssrc" => {
                    let value = value.unwrap();
                    if let [id, param] = value.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
                        let id = id.to_string();
                        let parameter = if let [name, value] =
                            param.splitn(2, ':').collect::<Vec<_>>().as_slice()
                        {
                            let name = name.to_string();
                            let value = Some(value.to_string());
                            Parameter { name, value }
                        } else {
                            let name = param.to_string();
                            let value = None;
                            Parameter { name, value }
                        };
                        let source = ssrc_map.entry(id.clone()).or_insert_with(|| {
                            ssrc_order.push(id.clone());
                            Source::new(id)
                        });
                        source.parameters.push(parameter);
                    } else {
                        panic!("No param in ssrc attribute.");
                    }
                }
                "rtcp-mux" => {
                    assert!(value.is_none());
                    description.rtcp_mux = Some(RtcpMux);
                }
                "extmap" => {
                    let value = value.unwrap();
                    if let [id, uri] = value.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
                        let id = id.parse().unwrap();
                        let uri = uri.to_string();
                        description.hdrexts.push(RtpHdrext::new(id, uri));
                    } else {
                        panic!("No param in ssrc attribute.");
                    }
                }
                // Legacy stuff, unused in Jingle.
                "rtcp" => {
                    let value = value.unwrap();
                    assert_eq!(value, "9 IN IP4 0.0.0.0");
                }
                /*
                // Unused so far by Conversations.
                "sendrecv" => {
                    assert!(value.is_none());
                    println!("TODO: sendrecv");
                },
                // Unused so far by Conversations.
                "maxptime" => {
                    let value = value.unwrap();
                    println!("TODO: maxptime:{}", value);
                },
                */
                _ => todo!("unsupported attribute: {:?}", attribute),
            }
        }
        for id in ssrc_order {
            description.ssrcs.push(ssrc_map[&id].clone());
        }
        for id in media.1 {
            if id >= 96 && id <= 127 {
                description.payload_types.push(rtp_map[&id].clone());
            } else {
                description
                    .payload_types
                    .push(STATIC_PAYLOADS[id as usize].clone());
            }
        }
        if let Some(setup) = setup {
            if let Some(ref mut fingerprint) = transport.fingerprint {
                fingerprint.setup = setup;
            } else {
                panic!("No fingerprint despite setup.");
            }
        }
        content.description = Some(description.into());
        content.transport = Some(transport.into());
        jingle.contents.push(content);
    }
    Ok(jingle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use xmpp_parsers::Element;

    #[test]
    fn conversations_audio_to_jingle() {
        let sdp = include_str!("../tests/conversations-audio.sdp");
        let jingle = sdp_to_jingle(sdp).unwrap();
        let elem: Element = include_str!("../tests/conversations-audio.xml")
            .parse()
            .unwrap();
        let jingle2 = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle, jingle2);
    }

    #[test]
    fn conversations_audio_video_to_jingle() {
        let sdp = include_str!("../tests/conversations-audio-video.sdp");
        let jingle = sdp_to_jingle(sdp).unwrap();
        let elem: Element = include_str!("../tests/conversations-audio-video.xml")
            .parse()
            .unwrap();
        let jingle2 = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle, jingle2);
    }
}
