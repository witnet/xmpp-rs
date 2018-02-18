Version 0.8.0, released 2018-02-18:
  * Additions
    - Link Mauve replaced error\_chain with failure ( https://gitlab.com/lumi/minidom-rs/merge_requests/27 )
Version 0.6.2, released 2017-08-27:
  * Additions
    - Link Mauve added an implementation of IntoElements for all Into<Element> ( https://gitlab.com/lumi/minidom-rs/merge_requests/19 )
Version 0.6.1, released 2017-08-20:
  * Additions
    - Astro added Element::has_ns, which checks whether an element's namespace matches the passed argument. ( https://gitlab.com/lumi/minidom-rs/merge_requests/16 )
    - Link Mauve updated the quick-xml dependency to the latest version.
  * Fixes
    - Because break value is now stable, Link Mauve rewrote some code marked FIXME to use it.
Version 0.6.0, released 2017-08-13:
  * Big changes
    - Astro added proper support for namespace prefixes. ( https://gitlab.com/lumi/minidom-rs/merge_requests/14 )
  * Fixes
    - Astro fixed a regression that caused the writer not to escape its xml output properly. ( https://gitlab.com/lumi/minidom-rs/merge_requests/15 )
Version 0.5.0, released 2017-06-10:
  * Big changes
    - Eijebong made parsing a lot faster by switching the crate from xml-rs to quick_xml. ( https://gitlab.com/lumi/minidom-rs/merge_requests/11 )
