#![deny(missing_docs)]

//! A minimal DOM crate built on top of quick-xml.
//!
//! This library exports an `Element` struct which represents a DOM tree.
//!
//! # Example
//!
//! Run with `cargo run --example articles`. Located in `examples/articles.rs`.
//!
//! ```rust,ignore
//! extern crate minidom;
//!
//! use minidom::Element;
//!
//! const DATA: &'static str = r#"<articles xmlns="article">
//!     <article>
//!         <title>10 Terrible Bugs You Would NEVER Believe Happened</title>
//!         <body>
//!             Rust fixed them all. &lt;3
//!         </body>
//!     </article>
//!     <article>
//!         <title>BREAKING NEWS: Physical Bug Jumps Out Of Programmer's Screen</title>
//!         <body>
//!             Just kidding!
//!         </body>
//!     </article>
//! </articles>"#;
//!
//! const ARTICLE_NS: &'static str = "article";
//!
//! #[derive(Debug)]
//! pub struct Article {
//!     title: String,
//!     body: String,
//! }
//!
//! fn main() {
//!     let root: Element = DATA.parse().unwrap();
//!
//!     let mut articles: Vec<Article> = Vec::new();
//!
//!     for child in root.children() {
//!         if child.is("article", ARTICLE_NS) {
//!             let title = child.get_child("title", ARTICLE_NS).unwrap().text();
//!             let body = child.get_child("body", ARTICLE_NS).unwrap().text();
//!             articles.push(Article {
//!                 title: title,
//!                 body: body.trim().to_owned(),
//!             });
//!         }
//!     }
//!
//!     println!("{:?}", articles);
//! }
//! ```
//!
//! # Usage
//!
//! To use `minidom`, add this to your `Cargo.toml` under `dependencies`:
//!
//! ```toml,ignore
//! minidom = "*"
//! ```

pub use quick_xml;

pub mod error;
pub mod element;
pub mod convert;
pub mod node;
mod namespace_set;

#[cfg(test)] mod tests;

pub use error::{Error, Result};
pub use element::{Element, Children, ChildrenMut, ElementBuilder};
pub use node::Node;
pub use convert::IntoAttributeValue;
