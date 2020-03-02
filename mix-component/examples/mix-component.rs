// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use async_std::task;
use mix_component::{Component, RecvError};

async fn spawn_component() -> Result<(), RecvError> {
    // Let’s init some constants we need to know where to connect and how.
    let jid = "chat.inkscape.org".parse().unwrap();
    let server = "[::1]:5347".parse().unwrap();
    let password = "password";

    // Now do the connection and authentication dance.
    let mut component = Component::connect(jid, server, password).await?;

    // And we’re connected!
    component.accept_loop().await
}

fn main() {
    task::block_on(spawn_component()).unwrap();
}
