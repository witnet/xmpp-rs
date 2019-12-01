// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::env;
use std::io::{self, Read};
use xmpp_parsers::{
    caps::{compute_disco as compute_disco_caps, hash_caps, Caps},
    disco::DiscoInfoResult,
    ecaps2::{compute_disco as compute_disco_ecaps2, hash_ecaps2, ECaps2},
    hashes::Algo,
    Element, Error,
};

fn get_caps(disco: &DiscoInfoResult, node: String) -> Result<Caps, String> {
    let caps_data = compute_disco_caps(&disco);
    let caps_hash = hash_caps(&caps_data, Algo::Sha_1)?;
    Ok(Caps::new(node, caps_hash))
}

fn get_ecaps2(disco: &DiscoInfoResult) -> Result<ECaps2, Error> {
    let ecaps2_data = compute_disco_ecaps2(&disco)?;
    let ecaps2_sha256 = hash_ecaps2(&ecaps2_data, Algo::Sha_256)?;
    let ecaps2_sha3_256 = hash_ecaps2(&ecaps2_data, Algo::Sha3_256)?;
    Ok(ECaps2::new(vec![ecaps2_sha256, ecaps2_sha3_256]))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <node>", args[0]);
        std::process::exit(1);
    }
    let node = args[1].clone();

    eprintln!("Reading a disco#info payload from stdin...");

    // Read from stdin.
    let stdin = io::stdin();
    let mut data = String::new();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut data)?;

    // Parse the payload into a DiscoInfoResult.
    let elem: Element = data.parse()?;
    let disco = DiscoInfoResult::try_from(elem)?;

    // Compute both kinds of caps.
    let caps = get_caps(&disco, node)?;
    let ecaps2 = get_ecaps2(&disco)?;

    // Print them.
    let caps_elem = Element::from(caps);
    let ecaps2_elem = Element::from(ecaps2);
    println!("{}", String::from(&caps_elem));
    println!("{}", String::from(&ecaps2_elem));

    Ok(())
}
