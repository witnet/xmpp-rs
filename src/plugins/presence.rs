use error::Error;
use plugin::{Plugin, PluginProxy};

use minidom::Element;

use ns;

use std::fmt;

use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Show {
    Available,
    Away,
    ExtendedAway,
    DoNotDisturb,
    Chat,
    Unavailable,
}

impl fmt::Display for Show {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Show::Away => write!(fmt, "away"),
            Show::ExtendedAway => write!(fmt, "xa"),
            Show::DoNotDisturb => write!(fmt, "dnd"),
            Show::Chat => write!(fmt, "chat"),

            // will never be seen inside a <show>, maybe should crash?
            Show::Available => write!(fmt, "available"),
            Show::Unavailable => write!(fmt, "unavailable"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidShow;

impl FromStr for Show {
    type Err = InvalidShow;

    fn from_str(s: &str) -> Result<Show, InvalidShow> {
        Ok(match s {
            "away" => Show::Away,
            "xa" => Show::ExtendedAway,
            "dnd" => Show::DoNotDisturb,
            "chat" => Show::Chat,

            _ => { return Err(InvalidShow); }
        })
    }
}

pub struct PresencePlugin {
    proxy: PluginProxy,
}

impl PresencePlugin {
    pub fn new() -> PresencePlugin {
        PresencePlugin {
            proxy: PluginProxy::new(),
        }
    }

    pub fn set_presence(&self, show: Show, status: Option<String>) -> Result<(), Error> {
        if show == Show::Unavailable {
            self.proxy.send(Element::builder("presence")
                                    .ns(ns::CLIENT)
                                    .attr("type", "unavailable")
                                    .build());
        }
        else {
            let mut stanza = Element::builder("presence")
                                     .ns(ns::CLIENT)
                                     .build();
            if let Some(stat) = status {
                let mut elem = Element::builder("status")
                                       .ns(ns::CLIENT)
                                       .build();
                elem.append_text_node(stat);
                stanza.append_child(elem);
            }
            let mut elem = Element::builder("show")
                                   .ns(ns::CLIENT)
                                   .build();
            if show != Show::Available {
                elem.append_text_node(show.to_string());
            }
            stanza.append_child(elem);
            self.proxy.send(stanza);
        }
        Ok(())
    }
}

impl Plugin for PresencePlugin {
    fn get_proxy(&mut self) -> &mut PluginProxy {
        &mut self.proxy
    }
}
