/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub const JABBER_CLIENT: &'static str = "jabber:client";

/// XEP-0004: Data Forms
pub const DATA_FORMS: &'static str = "jabber:x:data";

/// XEP-0030: Service Discovery
pub const DISCO_INFO: &'static str = "http://jabber.org/protocol/disco#info";

/// XEP-0047: In-Band Bytestreams
pub const IBB: &'static str = "http://jabber.org/protocol/ibb";

/// XEP-0085: Chat State Notifications
pub const CHATSTATES: &'static str = "http://jabber.org/protocol/chatstates";

/// XEP-0166: Jingle
pub const JINGLE: &'static str = "urn:xmpp:jingle:1";

/// XEP-0184: Message Delivery Receipts
pub const RECEIPTS: &'static str = "urn:xmpp:receipts";

/// XEP-0199: XMPP Ping
pub const PING: &'static str = "urn:xmpp:ping";

/// XEP-0203: Delayed Delivery
pub const DELAY: &'static str = "urn:xmpp:delay";

/// XEP-0221: Data Forms Media Element
pub const MEDIA_ELEMENT: &'static str = "urn:xmpp:media-element";

/// XEP-0224: Attention
pub const ATTENTION: &'static str = "urn:xmpp:attention:0";

/// XEP-0234: Jingle File Transfer
pub const JINGLE_FT: &'static str = "urn:xmpp:jingle:apps:file-transfer:5";
/// XEP-0234: Jingle File Transfer
pub const JINGLE_FT_ERROR: &'static str = "urn:xmpp:jingle:apps:file-transfer:errors:0";

/// XEP-0261: Jingle In-Band Bytestreams Transport Method
pub const JINGLE_IBB: &'static str = "urn:xmpp:jingle:transports:ibb:1";

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASHES: &'static str = "urn:xmpp:hashes:2";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA_256: &'static str = "urn:xmpp:hash-function-text-name:sha-256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA_512: &'static str = "urn:xmpp:hash-function-text-name:sha-512";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA3_256: &'static str = "urn:xmpp:hash-function-text-name:sha3-256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_SHA3_512: &'static str = "urn:xmpp:hash-function-text-name:sha3-512";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_BLAKE2B_256: &'static str = "urn:xmpp:hash-function-text-name:id-blake2b256";
/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub const HASH_ALGO_BLAKE2B_512: &'static str = "urn:xmpp:hash-function-text-name:id-blake2b512";

/// XEP-0308: Last Message Correction
pub const MESSAGE_CORRECT: &'static str = "urn:xmpp:message-correct:0";

/// XEP-0380: Explicit Message Encryption
pub const EME: &'static str = "urn:xmpp:eme:0";

/// XEP-0390: Entity Capabilities 2.0
pub const ECAPS2: &'static str = "urn:xmpp:caps";
/// XEP-0390: Entity Capabilities 2.0
pub const ECAPS2_OPTIMIZE: &'static str = "urn:xmpp:caps:optimize";
