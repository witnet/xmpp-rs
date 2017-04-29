// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct URI {
    pub type_: String,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct MediaElement {
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub uris: Vec<URI>,
}

pub fn parse_media_element(root: &Element) -> Result<MediaElement, Error> {
    if !root.is("media", ns::MEDIA_ELEMENT) {
        return Err(Error::ParseError("This is not a media element."));
    }

    let width = root.attr("width").and_then(|width| width.parse().ok());
    let height = root.attr("height").and_then(|height| height.parse().ok());
    let mut uris = vec!();
    for uri in root.children() {
        if uri.is("uri", ns::MEDIA_ELEMENT) {
            let type_ = uri.attr("type").ok_or(Error::ParseError("Attribute type on uri is mandatory."))?;
            let text = uri.text().trim().to_owned();
            if text == "" {
                return Err(Error::ParseError("URI missing in uri."));
            }
            uris.push(URI { type_: type_.to_owned(), uri: text });
        } else {
            return Err(Error::ParseError("Unknown child in media element."));
        }
    }
    Ok(MediaElement { width: width, height: height, uris: uris })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use media_element;
    use data_forms;

    #[test]
    fn test_simple() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'/>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert!(media.width.is_none());
        assert!(media.height.is_none());
        assert!(media.uris.is_empty());
    }

    #[test]
    fn test_width_height() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element' width='32' height='32'/>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert_eq!(media.width.unwrap(), 32);
        assert_eq!(media.height.unwrap(), 32);
    }

    #[test]
    fn test_uri() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'><uri type='text/html'>https://example.org/</uri></media>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert_eq!(media.uris.len(), 1);
        assert_eq!(media.uris[0].type_, "text/html");
        assert_eq!(media.uris[0].uri, "https://example.org/");
    }

    #[test]
    fn test_invalid_width_height() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element' width=''/>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert!(media.width.is_none());

        let elem: Element = "<media xmlns='urn:xmpp:media-element' width='coucou'/>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert!(media.width.is_none());

        let elem: Element = "<media xmlns='urn:xmpp:media-element' height=''/>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert!(media.height.is_none());

        let elem: Element = "<media xmlns='urn:xmpp:media-element' height='-10'/>".parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert!(media.height.is_none());
    }

    #[test]
    fn test_unknown_child() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'><coucou/></media>".parse().unwrap();
        let error = media_element::parse_media_element(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in media element.");
    }

    #[test]
    fn test_bad_uri() {
        let elem: Element = "<media xmlns='urn:xmpp:media-element'><uri>https://example.org/</uri></media>".parse().unwrap();
        let error = media_element::parse_media_element(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Attribute type on uri is mandatory.");

        let elem: Element = "<media xmlns='urn:xmpp:media-element'><uri type='text/html'/></media>".parse().unwrap();
        let error = media_element::parse_media_element(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "URI missing in uri.");
    }

    #[test]
    fn test_xep_ex1() {
        let elem: Element = r#"
<media xmlns='urn:xmpp:media-element'>
  <uri type='audio/x-wav'>
    http://victim.example.com/challenges/speech.wav?F3A6292C
  </uri>
  <uri type='audio/ogg; codecs=speex'>
    cid:sha1+a15a505e360702b79c75a5f67773072ed392f52a@bob.xmpp.org
  </uri>
  <uri type='audio/mpeg'>
    http://victim.example.com/challenges/speech.mp3?F3A6292C
  </uri>
</media>"#.parse().unwrap();
        let media = media_element::parse_media_element(&elem).unwrap();
        assert!(media.width.is_none());
        assert!(media.height.is_none());
        assert_eq!(media.uris.len(), 3);
        assert_eq!(media.uris[0].type_, "audio/x-wav");
        assert_eq!(media.uris[0].uri, "http://victim.example.com/challenges/speech.wav?F3A6292C");
        assert_eq!(media.uris[1].type_, "audio/ogg; codecs=speex");
        assert_eq!(media.uris[1].uri, "cid:sha1+a15a505e360702b79c75a5f67773072ed392f52a@bob.xmpp.org");
        assert_eq!(media.uris[2].type_, "audio/mpeg");
        assert_eq!(media.uris[2].uri, "http://victim.example.com/challenges/speech.mp3?F3A6292C");
    }

    #[test]
    fn test_xep_ex2() {
        let elem: Element = r#"
<x xmlns='jabber:x:data' type='form'>
  [ ... ]
  <field var='ocr'>
    <media xmlns='urn:xmpp:media-element'
           height='80'
           width='290'>
      <uri type='image/jpeg'>
        http://www.victim.com/challenges/ocr.jpeg?F3A6292C
      </uri>
      <uri type='image/jpeg'>
        cid:sha1+f24030b8d91d233bac14777be5ab531ca3b9f102@bob.xmpp.org
      </uri>
    </media>
  </field>
  [ ... ]
</x>"#.parse().unwrap();
        let form = data_forms::parse_data_form(&elem).unwrap();
        assert_eq!(form.fields.len(), 1);
        assert_eq!(form.fields[0].var, "ocr");
        assert_eq!(form.fields[0].media[0].width, Some(290));
        assert_eq!(form.fields[0].media[0].height, Some(80));
        assert_eq!(form.fields[0].media[0].uris[0].type_, "image/jpeg");
        assert_eq!(form.fields[0].media[0].uris[0].uri, "http://www.victim.com/challenges/ocr.jpeg?F3A6292C");
        assert_eq!(form.fields[0].media[0].uris[1].type_, "image/jpeg");
        assert_eq!(form.fields[0].media[0].uris[1].uri, "cid:sha1+f24030b8d91d233bac14777be5ab531ca3b9f102@bob.xmpp.org");
    }
}
