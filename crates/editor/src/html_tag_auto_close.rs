use language::{BufferSnapshot, Node};
use text::{Point, ToOffset};

const HTML_VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub(super) fn closing_tag_for_input(buffer: &BufferSnapshot, point: Point) -> Option<String> {
    let offset = point.to_offset(&buffer.text);
    let less_than_offset = offset.checked_sub(1)?;
    if !buffer
        .text
        .text_for_range(less_than_offset..offset)
        .equals_str("<")
    {
        return None;
    }

    let layer = buffer.smallest_syntax_layer_containing(less_than_offset..offset)?;
    let root = layer.node();
    let less_than = root.descendant_for_byte_range(less_than_offset, offset)?;
    let mut ancestor = Some(less_than);
    while let Some(node) = ancestor {
        if matches!(
            node.kind(),
            "comment" | "doctype" | "quoted_attribute_value" | "attribute_value" | "raw_text"
        ) {
            return None;
        }
        ancestor = node.parent();
    }

    let mut tags = Vec::new();
    let mut cursor = root.walk();
    let mut nodes = vec![root];
    while let Some(node) = nodes.pop() {
        if node.end_byte() <= less_than_offset
            && matches!(node.kind(), "start_tag" | "end_tag" | "self_closing_tag")
        {
            tags.push(node);
        }
        if node.start_byte() < less_than_offset {
            nodes.extend(node.children(&mut cursor));
        }
    }
    tags.sort_by_key(Node::start_byte);

    let mut open_tags = Vec::new();
    for tag in tags {
        let Some(name_node) = tag.named_child(0) else {
            continue;
        };
        let tag_name = buffer
            .text
            .text_for_range(name_node.byte_range())
            .collect::<String>();

        match tag.kind() {
            "start_tag" if !is_void_element(&tag_name) => open_tags.push(tag_name),
            "end_tag" => {
                if let Some(index) = open_tags
                    .iter()
                    .rposition(|open_tag| open_tag.eq_ignore_ascii_case(&tag_name))
                {
                    open_tags.truncate(index);
                }
            }
            _ => {}
        }
    }

    open_tags.pop()
}

fn is_void_element(tag_name: &str) -> bool {
    HTML_VOID_ELEMENTS
        .iter()
        .any(|void_element| tag_name.eq_ignore_ascii_case(void_element))
}

#[cfg(test)]
mod tests {
    use crate::{
        editor_tests::init_test,
        test::{editor_test_context::EditorTestContext, TestAppContext},
    };

    use gpui::AppContext as _;
    use languages::language;

    async fn test_setup(cx: &mut TestAppContext) -> EditorTestContext {
        init_test(cx, |_| {});

        let mut cx = EditorTestContext::new(cx).await;
        cx.update_buffer(|buffer, cx| {
            buffer.set_language(
                Some(language("HTML", tree_sitter_html::LANGUAGE.into())),
                cx,
            );
        });
        cx
    }

    macro_rules! check {
        ($name:ident, $initial:literal + $input:literal => $expected:expr) => {
            #[gpui::test]
            async fn $name(cx: &mut TestAppContext) {
                let mut cx = test_setup(cx).await;
                cx.set_state($initial);
                cx.run_until_parked();

                cx.update_editor(|editor, window, cx| {
                    editor.handle_input($input, window, cx);
                });
                cx.run_until_parked();
                cx.assert_editor_state($expected);
            }
        };
    }

    check!(
        inserts_closing_tag_for_unclosed_element,
        "<p>text<ˇ" + "/" => "<p>text</p>ˇ"
    );

    check!(
        inserts_closing_tag_for_innermost_element,
        "<main><p>text<ˇ" + "/" => "<main><p>text</p>ˇ"
    );

    check!(
        does_not_insert_a_closing_tag_without_an_open_element,
        "<img><ˇ" + "/" => "<img></ˇ"
    );

    check!(
        does_not_insert_a_closing_tag_for_void_elements,
        "<img src=\"image.png\"><ˇ" + "/" => "<img src=\"image.png\"></ˇ"
    );

    check!(
        does_not_insert_a_closing_tag_inside_a_comment,
        "<p><!-- <ˇ" + "/" => "<p><!-- </ˇ"
    );

    check!(
        does_not_insert_a_closing_tag_inside_an_attribute,
        "<div title=\"<ˇ\">" + "/" => "<div title=\"</ˇ\">"
    );

    check!(
        does_not_insert_a_duplicate_closing_tag,
        "<p>text</p><ˇ" + "/" => "<p>text</p></ˇ"
    );
}
