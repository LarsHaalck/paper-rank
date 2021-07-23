use comrak::nodes::{AstNode, NodeValue};
use comrak::{format_html, parse_document, Arena, ComrakOptions};

pub fn markdown_to_html(md: &str) -> String {
    return markdown_to_html_filtered(md);
}

fn markdown_to_html_filtered(md: &str) -> String
{
    let arena = Arena::new();
    let root = parse_document(&arena, md, &ComrakOptions::default());

    fn filter_images<'a>(node: &'a AstNode<'a>) {
        let is_image = |node: &AstNode| -> bool {
            match node.data.borrow().value {
                NodeValue::Image(_) => true,
                _ => false,
            }
        };

        let mut children = node.children().peekable();
        if is_image(node) || (children.peek().is_some() && children.all(|n| is_image(n))) {
            node.detach();
        }

        for c in node.children() {
            filter_images(c);
        }
    }

    filter_images(root);

    let mut html = vec![];
    format_html(root, &ComrakOptions::default(), &mut html).unwrap();
    return String::from_utf8(html).unwrap();
}