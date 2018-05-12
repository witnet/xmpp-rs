// Copyright (c) 2017-2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

macro_rules! get_attr {
    ($elem:ident, $attr:tt, $type:tt) => (
        get_attr!($elem, $attr, $type, value, value.parse()?)
    );
    ($elem:ident, $attr:tt, optional_empty, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some("") => None,
            Some($value) => Some($func),
            None => None,
        }
    );
    ($elem:ident, $attr:tt, optional, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some($value) => Some($func),
            None => None,
        }
    );
    ($elem:ident, $attr:tt, required, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some($value) => $func,
            None => return Err(Error::ParseError(concat!("Required attribute '", $attr, "' missing."))),
        }
    );
    ($elem:ident, $attr:tt, default, $value:ident, $func:expr) => (
        match $elem.attr($attr) {
            Some($value) => $func,
            None => Default::default(),
        }
    );
}

macro_rules! generate_attribute {
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+,}) => (
        generate_attribute!($elem, $name, {$($a => $b),+});
    );
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+,}, Default = $default:ident) => (
        generate_attribute!($elem, $name, {$($a => $b),+}, Default = $default);
    );
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+}) => (
        #[derive(Debug, Clone, PartialEq)]
        pub enum $elem {
            $(
                #[doc=$b]
                #[doc="value for this attribute."]
                $a
            ),+
        }
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                Ok(match s {
                    $($b => $elem::$a),+,
                    _ => return Err(Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
                })
            }
        }
        impl IntoAttributeValue for $elem {
            fn into_attribute_value(self) -> Option<String> {
                Some(String::from(match self {
                    $($elem::$a => $b),+
                }))
            }
        }
    );
    ($elem:ident, $name:tt, {$($a:ident => $b:tt),+}, Default = $default:ident) => (
        #[derive(Debug, Clone, PartialEq)]
        pub enum $elem {
            $(
                #[doc=$b]
                #[doc="value for this attribute."]
                $a
            ),+
        }
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                Ok(match s {
                    $($b => $elem::$a),+,
                    _ => return Err(Error::ParseError(concat!("Unknown value for '", $name, "' attribute."))),
                })
            }
        }
        impl IntoAttributeValue for $elem {
            #[allow(unreachable_patterns)]
            fn into_attribute_value(self) -> Option<String> {
                Some(String::from(match self {
                    $elem::$default => return None,
                    $($elem::$a => $b),+
                }))
            }
        }
        impl Default for $elem {
            fn default() -> $elem {
                $elem::$default
            }
        }
    );
}

macro_rules! generate_element_enum {
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+,}) => (
        generate_element_enum!($(#[$meta])* $elem, $name, $ns, {$($(#[$enum_meta])* $enum => $enum_name),+});
    );
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+}) => (
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub enum $elem {
            $(
            $(#[$enum_meta])*
            $enum
            ),+
        }
        impl TryFrom<Element> for $elem {
            type Err = Error;
            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_ns_only!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_attributes!(elem, $name);
                Ok(match elem.name() {
                    $($enum_name => $elem::$enum,)+
                    _ => return Err(Error::ParseError(concat!("This is not a ", $name, " element."))),
                })
            }
        }
        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder(match elem {
                    $($elem::$enum => $enum_name,)+
                }).ns($ns)
                  .build()
            }
        }
    );
}

macro_rules! generate_attribute_enum {
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, $attr:tt, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+,}) => (
        generate_attribute_enum!($(#[$meta])* $elem, $name, $ns, $attr, {$($(#[$enum_meta])* $enum => $enum_name),+});
    );
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, $attr:tt, {$($(#[$enum_meta:meta])* $enum:ident => $enum_name:tt),+}) => (
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq)]
        pub enum $elem {
            $(
            $(#[$enum_meta])*
            $enum
            ),+
        }
        impl TryFrom<Element> for $elem {
            type Err = Error;
            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_ns_only!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_unknown_attributes!(elem, $name, [$attr]);
                Ok(match get_attr!(elem, $attr, required) {
                    $($enum_name => $elem::$enum,)+
                    _ => return Err(Error::ParseError(concat!("Invalid ", $name, " ", $attr, " value."))),
                })
            }
        }
        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        .attr($attr, match elem {
                             $($elem::$enum => $enum_name,)+
                         })
                         .build()
            }
        }
    );
}

macro_rules! check_self {
    ($elem:ident, $name:tt, $ns:expr) => (
        check_self!($elem, $name, $ns, $name);
    );
    ($elem:ident, $name:tt, $ns:expr, $pretty_name:tt) => (
        if !$elem.is($name, $ns) {
            return Err(Error::ParseError(concat!("This is not a ", $pretty_name, " element.")));
        }
    );
}

macro_rules! check_ns_only {
    ($elem:ident, $name:tt, $ns:expr) => (
        if !$elem.has_ns($ns) {
            return Err(Error::ParseError(concat!("This is not a ", $name, " element.")));
        }
    );
}

macro_rules! check_no_children {
    ($elem:ident, $name:tt) => (
        for _ in $elem.children() {
            return Err(Error::ParseError(concat!("Unknown child in ", $name, " element.")));
        }
    );
}

macro_rules! check_no_attributes {
    ($elem:ident, $name:tt) => (
        for _ in $elem.attrs() {
            return Err(Error::ParseError(concat!("Unknown attribute in ", $name, " element.")));
        }
    );
}

macro_rules! check_no_unknown_attributes {
    ($elem:ident, $name:tt, [$($attr:tt),*]) => (
        for (_attr, _) in $elem.attrs() {
            $(
            if _attr == $attr {
                continue;
            }
            )*
            return Err(Error::ParseError(concat!("Unknown attribute in ", $name, " element.")));
        }
    );
}

macro_rules! generate_empty_element {
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr) => (
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $elem;

        impl TryFrom<Element> for $elem {
            type Err = Error;

            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_attributes!(elem, $name);
                Ok($elem)
            }
        }

        impl From<$elem> for Element {
            fn from(_: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        .build()
            }
        }
    );
}

macro_rules! generate_element_with_only_attributes {
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, [$($(#[$attr_meta:meta])* $attr:ident: $attr_type:ty = $attr_name:tt => $attr_action:tt),+,]) => (
        generate_element_with_only_attributes!($(#[$meta])* $elem, $name, $ns, [$($(#[$attr_meta])* $attr: $attr_type = $attr_name => $attr_action),*]);
    );
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, [$($(#[$attr_meta:meta])* $attr:ident: $attr_type:ty = $attr_name:tt => $attr_action:tt),+]) => (
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $elem {
            $(
            $(#[$attr_meta])*
            pub $attr: $attr_type,
            )*
        }

        impl TryFrom<Element> for $elem {
            type Err = Error;

            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_unknown_attributes!(elem, $name, [$($attr_name),*]);
                Ok($elem {
                    $(
                    $attr: get_attr!(elem, $attr_name, $attr_action),
                    )*
                })
            }
        }

        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        $(
                        .attr($attr_name, elem.$attr)
                        )*
                        .build()
            }
        }
    );
}

macro_rules! generate_id {
    ($elem:ident) => (
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $elem(pub String);
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                // TODO: add a way to parse that differently when needed.
                Ok($elem(String::from(s)))
            }
        }
        impl IntoAttributeValue for $elem {
            fn into_attribute_value(self) -> Option<String> {
                Some(self.0)
            }
        }
    );
}

macro_rules! generate_elem_id {
    ($elem:ident, $name:tt, $ns:expr) => (
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $elem(pub String);
        impl FromStr for $elem {
            type Err = Error;
            fn from_str(s: &str) -> Result<$elem, Error> {
                // TODO: add a way to parse that differently when needed.
                Ok($elem(String::from(s)))
            }
        }
        impl TryFrom<Element> for $elem {
            type Err = Error;
            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_attributes!(elem, $name);
                // TODO: add a way to parse that differently when needed.
                Ok($elem(elem.text()))
            }
        }
        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        .append(elem.0)
                        .build()
            }
        }
    );
}

macro_rules! generate_element_with_text {
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, $text_ident:ident: $codec:ident < $text_type:ty >) => (
        generate_element_with_text!($(#[$meta])* $elem, $name, $ns, [], $text_ident: $codec<$text_type>);
    );
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, [$($(#[$attr_meta:meta])* $attr:ident: $attr_type:ty = $attr_name:tt => $attr_action:tt),*], $text_ident:ident: $codec:ident < $text_type:ty >) => (
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $elem {
            $(
            $(#[$attr_meta])*
            pub $attr: $attr_type,
            )*
            pub $text_ident: $text_type,
        }

        impl TryFrom<Element> for $elem {
            type Err = Error;

            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, $ns);
                check_no_children!(elem, $name);
                check_no_unknown_attributes!(elem, $name, [$($attr_name),*]);
                Ok($elem {
                    $(
                    $attr: get_attr!(elem, $attr_name, $attr_action),
                    )*
                    $text_ident: $codec::decode(&elem.text())?,
                })
            }
        }

        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        $(
                        .attr($attr_name, elem.$attr)
                        )*
                        .append($codec::encode(&elem.$text_ident))
                        .build()
            }
        }
    );
}

macro_rules! generate_element_with_children {
    ($(#[$meta:meta])* $elem:ident, $name:tt, $ns:expr, attributes: [$($(#[$attr_meta:meta])* $attr:ident: $attr_type:ty = $attr_name:tt => $attr_action:tt),+], children: [$($(#[$child_meta:meta])* $child_ident:ident: Vec<$child_type:ty> = ($child_name:tt, $child_ns:expr) => $child_constructor:ident),+]) => (
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $elem {
            $(
            $(#[$attr_meta])*
            pub $attr: $attr_type,
            )*
            $(
            $(#[$child_meta])*
            pub $child_ident: Vec<$child_type>,
            )*
        }

        impl TryFrom<Element> for $elem {
            type Err = Error;

            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, $ns);
                check_no_unknown_attributes!(elem, $name, [$($attr_name),*]);
                let mut parsed_children = vec!();
                for child in elem.children() {
                    $(
                    if child.is($child_name, $child_ns) {
                        let parsed_child = $child_constructor::try_from(child.clone())?;
                        parsed_children.push(parsed_child);
                        continue;
                    }
                    )*
                    return Err(Error::ParseError(concat!("Unknown child in ", $name, " element.")));
                }
                Ok($elem {
                    $(
                    $attr: get_attr!(elem, $attr_name, $attr_action),
                    )*
                    $(
                    $child_ident: parsed_children,
                    )*
                })
            }
        }

        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name)
                        .ns($ns)
                        $(
                        .attr($attr_name, elem.$attr)
                        )*
                        $(
                        .append(elem.$child_ident)
                        )*
                        .build()
            }
        }
    );
}
