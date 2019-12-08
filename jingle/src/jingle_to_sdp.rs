// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::net::IpAddr;
use xmpp_parsers::hashes::{Algo, Hash};
use xmpp_parsers::jingle::{self, Action, Jingle};
use xmpp_parsers::jingle_ice_udp;

const DISCARD_PORT: u16 = 9;

#[derive(Debug)]
pub enum Error {
    TooManyContents,
    UnsupportedFingerprintHash(Algo),
    MissingDescription,
    UnknownDescription(String),
    InvalidDescription,
    MissingTransport,
    UnknownTransport(String),
    InvalidTransport,
}

struct SdpLine {
    key: char,
    value: String,
}

impl SdpLine {
    fn new<V: Into<String>>(key: char, value: V) -> SdpLine {
        let value = value.into();
        SdpLine { key, value }
    }

    fn media(value: String) -> SdpLine {
        SdpLine { key: 'm', value }
    }

    fn candidate(ip: IpAddr) -> SdpLine {
        let inet = match ip {
            IpAddr::V4(_) => "IP4",
            IpAddr::V6(_) => "IP6",
        };
        SdpLine {
            key: 'c',
            value: format!("IN {} {}", inet, ip),
        }
    }

    fn attribute<V: Into<Option<String>>>(key: &str, value: V) -> SdpLine {
        let value = value.into();
        SdpLine {
            key: 'a',
            value: match value {
                Some(value) => format!("{}:{}", key, value),
                None => String::from(key),
            },
        }
    }
}

impl fmt::Display for SdpLine {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}={}", self.key, self.value)
    }
}

fn jingle_description_to_sdp(
    content_name: String,
    description: Option<jingle::Description>,
    transport_sdp: Vec<SdpLine>,
) -> Result<Vec<SdpLine>, Error> {
    match description {
        Some(jingle::Description::Rtp(desc)) => {
            let mut sdp = Vec::new();
            let ids = desc
                .payload_types
                .iter()
                .map(|pl| format!("{}", pl.id))
                .collect::<Vec<String>>()
                .join(" ");
            sdp.push(SdpLine::media(format!(
                "{} {} {} {}",
                desc.media, DISCARD_PORT, "UDP/TLS/RTP/SAVPF", ids
            )));
            sdp.push(SdpLine::candidate("0.0.0.0".parse().unwrap()));
            sdp.push(SdpLine::attribute(
                "rtcp",
                format!("{} IN IP4 0.0.0.0", DISCARD_PORT),
            ));
            sdp.extend(transport_sdp);
            sdp.push(SdpLine::attribute("mid", content_name));
            for hdrext in desc.hdrexts {
                sdp.push(SdpLine::attribute(
                    "extmap",
                    format!("{} {}", hdrext.id, hdrext.uri),
                ));
            }
            if desc.rtcp_mux.is_some() {
                sdp.push(SdpLine::attribute("rtcp-mux", None));
            }
            for pl in desc.payload_types {
                if pl.id >= 96 && pl.id <= 127 {
                    if let (Some(name), Some(clockrate)) = (pl.name, pl.clockrate) {
                        if pl.channels.0 != 1 {
                            sdp.push(SdpLine::attribute(
                                "rtpmap",
                                format!("{} {}/{}/{}", pl.id, name, clockrate, pl.channels.0),
                            ));
                        } else {
                            sdp.push(SdpLine::attribute(
                                "rtpmap",
                                format!("{} {}/{}", pl.id, name, clockrate),
                            ));
                        }
                        for rtcp_fb in pl.rtcp_fbs {
                            sdp.push(SdpLine::attribute(
                                "rtcp-fb",
                                format!(
                                    "{} {}",
                                    pl.id,
                                    if let Some(subtype) = rtcp_fb.subtype {
                                        format!("{} {}", rtcp_fb.type_, subtype)
                                    } else {
                                        rtcp_fb.type_.clone()
                                    }
                                ),
                            ));
                        }
                        if !pl.parameters.is_empty() {
                            let params = pl
                                .parameters
                                .iter()
                                .map(|param| format!("{}={}", param.name, param.value))
                                .collect::<Vec<String>>()
                                .join("; ");
                            sdp.push(SdpLine::attribute("fmtp", format!("{} {}", pl.id, params)));
                        }
                    }
                }
            }
            for group in desc.ssrc_groups {
                let sources = group
                    .sources
                    .iter()
                    .map(|source| source.id.as_ref())
                    .collect::<Vec<_>>()
                    .join(" ");
                sdp.push(SdpLine::attribute("ssrc-group", format!("FID {}", sources)));
            }
            for ssrc in desc.ssrcs {
                for param in ssrc.parameters {
                    sdp.push(SdpLine::attribute(
                        "ssrc",
                        format!(
                            "{} {}",
                            ssrc.id,
                            if let Some(value) = param.value {
                                format!("{}:{}", param.name, value)
                            } else {
                                param.name.clone()
                            }
                        ),
                    ));
                }
            }
            Ok(sdp)
        }
        Some(jingle::Description::Unknown(elem)) => Err(Error::UnknownDescription(elem.ns())),
        None => Err(Error::MissingDescription),
    }
}

fn jingle_candidate_to_jsep(candidate: &jingle_ice_udp::Candidate) -> String {
    format!(
        "candidate:{} {} {} {} {} {} typ {}{}",
        candidate.foundation,
        candidate.component,
        candidate.protocol,
        candidate.priority,
        candidate.ip,
        candidate.port,
        candidate.type_,
        if let (Some(rel_addr), Some(rel_port)) = (candidate.rel_addr, candidate.rel_port) {
            format!(" raddr {} rport {}", rel_addr, rel_port)
        } else {
            String::new()
        }
    )
}

pub fn jingle_to_jsep(jingle: Jingle) -> Result<String, Error> {
    if jingle.contents.len() != 1 {
        return Err(Error::TooManyContents);
    }
    match jingle.action {
        Action::TransportInfo => {
            let transport = jingle.contents[0].transport.clone();
            match transport {
                Some(jingle::Transport::IceUdp(transport)) => {
                    if transport.candidates.len() != 1 {
                        return Err(Error::InvalidTransport);
                    }
                    let candidate = &transport.candidates[0];
                    Ok(jingle_candidate_to_jsep(candidate))
                }
                Some(jingle::Transport::Ibb(_)) => {
                    Err(Error::UnknownTransport(String::from("ibb")))
                }
                Some(jingle::Transport::Socks5(_)) => {
                    Err(Error::UnknownTransport(String::from("socks5")))
                }
                Some(jingle::Transport::Unknown(elem)) => Err(Error::UnknownTransport(elem.ns())),
                None => Err(Error::MissingTransport),
            }
        }
        _ => todo!("don’t panic here"),
    }
}

fn jingle_transport_to_sdp(transport: Option<jingle::Transport>) -> Result<Vec<SdpLine>, Error> {
    match transport {
        Some(jingle::Transport::IceUdp(transport)) => {
            let mut sdp = Vec::new();
            sdp.push(SdpLine::attribute("ice-pwd", transport.pwd.clone()));
            sdp.push(SdpLine::attribute("ice-ufrag", transport.ufrag.clone()));
            if let Some(fingerprint) = transport.fingerprint {
                let algo = match fingerprint.hash {
                    Algo::Sha_1 => "sha-1",
                    Algo::Sha_256 => "sha-256",
                    algo => return Err(Error::UnsupportedFingerprintHash(algo)),
                };
                let hash = Hash::new(fingerprint.hash, fingerprint.value);
                sdp.push(SdpLine::attribute(
                    "fingerprint",
                    format!("{} {}", algo, hash.to_colon_separated_hex()),
                ));
                sdp.push(SdpLine::attribute(
                    "setup",
                    format!("{}", fingerprint.setup),
                ));
            }
            for candidate in transport.candidates {
                sdp.push(SdpLine::candidate(candidate.ip));
                sdp.push(SdpLine::attribute(
                    "candidate",
                    format!(
                        "{} {} {} {} {} {} typ {}",
                        candidate.foundation,
                        candidate.component,
                        candidate.protocol,
                        candidate.priority,
                        candidate.ip,
                        candidate.port,
                        candidate.type_
                    ),
                ));
            }
            Ok(sdp)
        }
        Some(jingle::Transport::Ibb(_)) => Err(Error::UnknownTransport(String::from("ibb"))),
        Some(jingle::Transport::Socks5(_)) => Err(Error::UnknownTransport(String::from("socks5"))),
        Some(jingle::Transport::Unknown(elem)) => Err(Error::UnknownTransport(elem.ns())),
        None => Err(Error::MissingTransport),
    }
}

pub fn jingle_to_sdp(jingle: Jingle) -> Result<String, Error> {
    let mut sdp = Vec::new();
    sdp.push(SdpLine::new('v', "0"));
    sdp.push(SdpLine::new(
        'o',
        format!("- {} 2 IN IP4 127.0.0.1", jingle.sid.0),
    ));
    sdp.push(SdpLine::new('s', "-"));
    sdp.push(SdpLine::new('t', "0 0"));
    if let Some(group) = jingle.group {
        let contents = group
            .contents
            .iter()
            .map(|content| content.name.0.as_ref())
            .collect::<Vec<_>>()
            .join(" ");
        sdp.push(SdpLine::attribute(
            "group",
            format!("{} {}", group.semantics, contents),
        ));
    }
    for content in jingle.contents {
        match jingle.action {
            Action::SessionInitiate => {
                let transport_sdp = jingle_transport_to_sdp(content.transport)?;
                sdp.extend(jingle_description_to_sdp(
                    content.name.0,
                    content.description,
                    transport_sdp,
                )?);
            }
            _ => {
                // TODO: implement support for the other actions.
                todo!("add support for action={}", jingle.action);
            }
        }
    }
    Ok(sdp
        .iter()
        .map(|line| format!("{}", line))
        .collect::<Vec<String>>()
        .join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use xmpp_parsers::Element;

    #[test]
    fn conversations_initiate_audio() {
        let elem: Element = include_str!("../tests/conversations-audio.xml")
            .parse()
            .unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let sdp = jingle_to_sdp(jingle).unwrap();
        assert_eq!(sdp, include_str!("../tests/conversations-audio.sdp").trim());
    }

    #[test]
    fn conversations_initiate_video() {
        let elem: Element = include_str!("../tests/conversations-audio-video.xml")
            .parse()
            .unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let sdp = jingle_to_sdp(jingle).unwrap();
        assert_eq!(
            sdp,
            include_str!("../tests/conversations-audio-video.sdp").trim()
        );
    }

    #[test]
    fn conversations_candidate() {
        let elem: Element = include_str!("../tests/conversations-ice-candidate.xml")
            .parse()
            .unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let jsep = jingle_to_jsep(jingle).unwrap();
        assert_eq!(
            jsep,
            include_str!("../tests/conversations-ice-candidate.jsep").trim()
        );
    }
}
