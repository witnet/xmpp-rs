Version 0.6.0, released 2019-06-19:
  * Updates
    - Jid is now an enum, with two variants, Bare(BareJid) and Full(FullJid)
    - BareJid and FullJid are two specialised variants of a JID.

Version 0.5.3, released 2019-01-16:
  * Updates
    - Link Mauve bumped the minidom dependency version.
    - Use Edition 2018, putting the baseline rustc version to 1.31.
    - Run cargo-fmt on the code, to lower the barrier of entry.

Version 0.5.2, released 2018-07-31:
  * Updates
    - Astro bumped the minidom dependency version.
    - Updated the changelog to reflect that 0.5.1 was never actually released.

Version 0.5.1, "released" 2018-03-01:
  * Updates
    - Link Mauve implemented failure::Fail on JidParseError.
    - Link Mauve simplified the code a bit.

Version 0.5.0, released 2018-02-18:
  * Updates
    - Link Mauve has updated the optional `minidom` dependency.
    - Link Mauve has added tests for invalid JIDs, which adds more error cases.

Version 0.4.0, released 2017-12-27:
  * Updates
    - Maxime Buquet has updated the optional `minidom` dependency.
    - The repository has been transferred to xmpp-rs/jid-rs.

Version 0.3.1, released 2017-10-31:
  * Additions
    - Link Mauve added a minidom::IntoElements implementation on Jid behind the "minidom" feature. ( https://gitlab.com/lumi/jid-rs/merge_requests/9 )
