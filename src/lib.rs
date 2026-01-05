use pulldown_cmark::{Parser, Options, Event, Tag};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref TAG_RE: Regex = Regex::new(r#"^<([a-zA-Z0-9-]+)([^>]*?)(/?)>$"#).unwrap();
    static ref ATTR_RE: Regex = Regex::new(r#"([a-zA-Z0-9-]+)(?:=(?:"([^"]*)"|'([^']*)'|([^>\s]+)))?"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "element")]
    Element {
        tag: String,
        props: HashMap<String, serde_json::Value>,
        children: Vec<Node>,
    },
    #[serde(rename = "text")]
    Text {
        content: String,
    },
}

pub struct TranspileOptions {
    pub allowed_tags: Vec<String>,
}

fn parse_html_tag(html: &str) -> Option<(String, HashMap<String, serde_json::Value>, bool)> {
    let html = html.trim();
    if let Some(caps) = TAG_RE.captures(html) {
        let tag_name = caps.get(1).unwrap().as_str().to_string();
        let attrs_str = caps.get(2).unwrap().as_str();
        let is_self_closing = !caps.get(3).unwrap().as_str().is_empty();
        
        let mut props = HashMap::new();
        for attr_caps in ATTR_RE.captures_iter(attrs_str) {
            let key = attr_caps.get(1).unwrap().as_str().to_string();
            let value = attr_caps.get(2)
                .or_else(|| attr_caps.get(3))
                .or_else(|| attr_caps.get(4))
                .map(|m| serde_json::Value::String(m.as_str().to_string()))
                .unwrap_or(serde_json::Value::Bool(true));
            props.insert(key, value);
        }
        
        return Some((tag_name, props, is_self_closing));
    }
    
    // Handle closing tags
    if html.starts_with("</") && html.ends_with(">") {
        let tag_name = html[2..html.len()-1].trim().to_string();
        return Some((tag_name, HashMap::new(), false));
    }
    
    None
}

pub fn parse(markdown: &str, options: &TranspileOptions) -> Vec<Node> {
    let mut p_options = Options::empty();
    p_options.insert(Options::ENABLE_TABLES);
    p_options.insert(Options::ENABLE_STRIKETHROUGH);
    p_options.insert(Options::ENABLE_TASKLISTS);
    p_options.insert(Options::ENABLE_FOOTNOTES);
    p_options.insert(Options::ENABLE_SMART_PUNCTUATION);
    
    let parser = Parser::new_ext(markdown, p_options);
    let mut stack: Vec<Node> = Vec::new();
    let mut root: Vec<Node> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => {
                let node = match tag {
                    Tag::Heading { level, .. } => Node::Element {
                        tag: format!("h{}", level as u32),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Paragraph => Node::Element {
                        tag: "p".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Emphasis => Node::Element {
                        tag: "em".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Strong => Node::Element {
                        tag: "strong".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Link { dest_url, .. } => {
                        let mut props = HashMap::new();
                        props.insert("href".to_string(), serde_json::Value::String(dest_url.to_string()));
                        Node::Element {
                            tag: "a".to_string(),
                            props,
                            children: Vec::new(),
                        }
                    },
                    Tag::List(first) => Node::Element {
                        tag: if first.is_some() { "ol".to_string() } else { "ul".to_string() },
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Item => Node::Element {
                        tag: "li".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Table(_) => Node::Element {
                        tag: "table".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::TableHead => Node::Element {
                        tag: "thead".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::TableRow => Node::Element {
                        tag: "tr".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::TableCell => Node::Element {
                        tag: "td".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::Strikethrough => Node::Element {
                        tag: "del".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                    Tag::FootnoteDefinition(label) => {
                        let mut props = HashMap::new();
                        props.insert("id".to_string(), serde_json::Value::String(format!("fn-{}", label)));
                        props.insert("className".to_string(), serde_json::Value::String("footnote-definition".to_string()));
                        Node::Element {
                            tag: "div".to_string(),
                            props,
                            children: Vec::new(),
                        }
                    },
                    _ => Node::Element {
                        tag: "div".to_string(),
                        props: HashMap::new(),
                        children: Vec::new(),
                    },
                };
                stack.push(node);
            }
            Event::End(_) => {
                if let Some(node) = stack.pop() {
                    if stack.is_empty() {
                        root.push(node);
                    } else {
                        let parent = stack.last_mut().unwrap();
                        if let Node::Element { children, .. } = parent {
                            children.push(node);
                        }
                    }
                }
            }
            Event::Text(text) => {
                let node = Node::Text { content: text.to_string() };
                if stack.is_empty() {
                    root.push(node);
                } else {
                    let parent = stack.last_mut().unwrap();
                    if let Node::Element { children, .. } = parent {
                        children.push(node);
                    }
                }
            }
            Event::Code(code) => {
                let node = Node::Element {
                    tag: "code".to_string(),
                    props: HashMap::new(),
                    children: vec![Node::Text { content: code.to_string() }],
                };
                if stack.is_empty() {
                    root.push(node);
                } else {
                    let parent = stack.last_mut().unwrap();
                    if let Node::Element { children, .. } = parent {
                        children.push(node);
                    }
                }
            }
            Event::FootnoteReference(label) => {
                let mut props = HashMap::new();
                props.insert("href".to_string(), serde_json::Value::String(format!("#fn-{}", label)));
                props.insert("className".to_string(), serde_json::Value::String("footnote-ref".to_string()));
                let node = Node::Element {
                    tag: "sup".to_string(),
                    props: HashMap::new(),
                    children: vec![Node::Element {
                        tag: "a".to_string(),
                        props,
                        children: vec![Node::Text { content: label.to_string() }],
                    }],
                };
                if stack.is_empty() {
                    root.push(node);
                } else {
                    let parent = stack.last_mut().unwrap();
                    if let Node::Element { children, .. } = parent {
                        children.push(node);
                    }
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                if let Some((tag_name, props, is_self_closing)) = parse_html_tag(&html) {
                    if options.allowed_tags.contains(&tag_name) {
                        if html.starts_with("</") {
                            // Closing tag
                            if let Some(node) = stack.pop() {
                                if stack.is_empty() {
                                    root.push(node);
                                } else {
                                    let parent = stack.last_mut().unwrap();
                                    if let Node::Element { children, .. } = parent {
                                        children.push(node);
                                    }
                                }
                            }
                        } else {
                            // Opening tag
                            let node = Node::Element {
                                tag: tag_name,
                                props,
                                children: Vec::new(),
                            };
                            if is_self_closing {
                                if stack.is_empty() {
                                    root.push(node);
                                } else {
                                    let parent = stack.last_mut().unwrap();
                                    if let Node::Element { children, .. } = parent {
                                        children.push(node);
                                    }
                                }
                            } else {
                                stack.push(node);
                            }
                        }
                    } else {
                        // Tag not allowed, treat as text
                        let node = Node::Text { content: html.to_string() };
                        if stack.is_empty() {
                            root.push(node);
                        } else {
                            let parent = stack.last_mut().unwrap();
                            if let Node::Element { children, .. } = parent {
                                children.push(node);
                            }
                        }
                    }
                } else {
                    // Treat unknown HTML as text
                    let node = Node::Text { content: html.to_string() };
                    if stack.is_empty() {
                        root.push(node);
                    } else {
                        let parent = stack.last_mut().unwrap();
                        if let Node::Element { children, .. } = parent {
                            children.push(node);
                        }
                    }
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                let node = Node::Text { content: "\n".to_string() };
                if !stack.is_empty() {
                    let parent = stack.last_mut().unwrap();
                    if let Node::Element { children, .. } = parent {
                        children.push(node);
                    }
                }
            }
            _ => {}
        }
    }
    
    root
}

#[cfg(feature = "wasm")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn transpile(markdown: &str, allowed_tags: Vec<String>) -> Result<JsValue, JsValue> {
        let options = TranspileOptions { allowed_tags };
        let ast = parse(markdown, &options);
        serde_wasm_bindgen::to_value(&ast).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[cfg(feature = "android")]
mod android {
    use super::*;
    use jni::JNIEnv;
    use jni::objects::{JClass, JString};
    use jni::sys::jstring;

    #[no_mangle]
    pub extern "system" fn Java_com_clevertree_md2jsx_MarkdownParser_nativeParse(
        mut env: JNIEnv,
        _class: JClass,
        input: JString,
        allowed_tags_json: JString,
    ) -> jstring {
        let input: String = env.get_string(&input).expect("Couldn't get java string!").into();
        let allowed_tags_json: String = env.get_string(&allowed_tags_json).expect("Couldn't get java string!").into();
        let allowed_tags: Vec<String> = serde_json::from_str(&allowed_tags_json).unwrap_or_default();
        
        let options = TranspileOptions { allowed_tags };
        let ast = parse(&input, &options);
        let result_json = serde_json::to_string(&ast).unwrap();
        
        env.new_string(result_json).expect("Couldn't create java string!").into_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_node<'a>(nodes: &'a [Node], tag_name: &str) -> Option<&'a Node> {
        for node in nodes {
            match node {
                Node::Element { tag, children, .. } => {
                    if tag == tag_name {
                        return Some(node);
                    }
                    if let Some(found) = find_node(children, tag_name) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    #[test]
    fn test_gfm_footnotes() {
        let markdown = "Here is a footnote[^1]\n\n[^1]: This is the footnote content.";
        let options = TranspileOptions { allowed_tags: vec![] };
        let ast = parse(markdown, &options);
        println!("AST: {}", serde_json::to_string_pretty(&ast).unwrap());
        
        // Footnotes are rendered as <sup><a href=\"#fn-1\" className=\"footnote-ref\">1</a></sup>
        // and a <div class=\"footnote-definition\" id=\"fn-1\">...</div>
        
        let sup = find_node(&ast, "sup").expect("Should find sup for footnote ref");
        if let Node::Element { children, .. } = sup {
            let a = children.first().expect("Should have link child");
            if let Node::Element { tag, props, .. } = a {
                assert_eq!(tag, "a");
                let href = props.get("href").unwrap().as_str().unwrap();
                assert!(href.contains("#fn-1"));
            }
        }

        let div = find_node(&ast, "div").expect("Should find footnote definition");
        if let Node::Element { props, .. } = div {
            assert_eq!(props.get("className").unwrap().as_str().unwrap(), "footnote-definition");
        }
    }

    #[test]
    fn test_basic_markdown() {
        let markdown = "# Hello\nThis is **bold**";
        let options = TranspileOptions { allowed_tags: vec![] };
        let ast = parse(markdown, &options);
        
        assert_eq!(ast.len(), 2);
        if let Node::Element { tag, children, .. } = &ast[0] {
            assert_eq!(tag, "h1");
            assert_eq!(children[0], Node::Text { content: "Hello".to_string() });
        } else {
            panic!("Expected h1 element");
        }
    }

    #[test]
    fn test_html_tags() {
        let markdown = "Hello <VideoPlayer src=\"test.mp4\" /> world";
        let options = TranspileOptions { allowed_tags: vec!["VideoPlayer".to_string()] };
        let ast = parse(markdown, &options);
        
        let node = find_node(&ast, "VideoPlayer").expect("Should find VideoPlayer node");
        if let Node::Element { props, .. } = node {
            assert_eq!(props.get("src").unwrap(), "test.mp4");
        }
    }
    
    #[test]
    fn test_nested_html() {
        let markdown = "<div>\n\n# Inside\n\n</div>";
        let options = TranspileOptions { allowed_tags: vec!["div".to_string()] };
        let ast = parse(markdown, &options);
        
        assert!(find_node(&ast, "div").is_some());
    }

    #[test]
    fn test_allowed_tags_filtering() {
        let markdown = "<Allowed>Keep</Allowed><Forbidden>Drop</Forbidden>";
        let options = TranspileOptions { allowed_tags: vec!["Allowed".to_string()] };
        let ast = parse(markdown, &options);
        
        assert!(find_node(&ast, "Allowed").is_some());
        assert!(find_node(&ast, "Forbidden").is_none());
    }

    #[test]
    fn test_gfm_table() {
        let markdown = "| Header |\n| --- |\n| Cell |";
        let options = TranspileOptions { allowed_tags: vec![] };
        let ast = parse(markdown, &options);
        
        assert!(find_node(&ast, "table").is_some());
        assert!(find_node(&ast, "thead").is_some());
        assert!(find_node(&ast, "td").is_some());
    }

    #[test]
    fn test_strikethrough() {
        let markdown = "~~deleted~~";
        let options = TranspileOptions { allowed_tags: vec![] };
        let ast = parse(markdown, &options);
        
        assert!(find_node(&ast, "del").is_some());
    }
}
