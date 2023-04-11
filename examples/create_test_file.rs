use rust_xlearn::{diff_text, highlight_text};
use serde_json::json;
use std::fs;

fn main() {
    let text1 = "foo\nbar";
    let text2 = "foo\nbaz";
    let content1 = diff_text(text1, text2).unwrap();
    fs::write("fixtures/diff_test_txt1.txt", content1).unwrap();

    let jsonv = json!({
        "foo": "bar",
        "baz":"qux"
    });
    let text3 = serde_json::to_string_pretty(&jsonv).unwrap();
    let content2 = highlight_text(&text3, "json", None).unwrap();
    fs::write("fixtures/diff_test_txt2.txt", content2).unwrap();
}
