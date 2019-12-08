// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod jingle_to_sdp;
pub use jingle_to_sdp::{jingle_to_jsep, jingle_to_sdp};
mod sdp_to_jingle;
pub use sdp_to_jingle::sdp_to_jingle;
