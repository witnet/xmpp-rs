// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod event;

pub use self::event::PubSubEvent;

generate_id!(NodeName);
generate_id!(ItemId);
generate_id!(SubscriptionId);

generate_attribute!(Subscription, "subscription", {
    None => "none",
    Pending => "pending",
    Subscribed => "subscribed",
    Unconfigured => "unconfigured",
}, Default = None);
