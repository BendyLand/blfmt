use tree_sitter::{InputEdit, Parser, Language, Point};
use crate::{utils, c_ast};

pub fn parse_cpp_file(path: String) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading C++ parser.");
    let contents = std::fs::read_to_string(path).unwrap_or("".to_string());
    let tree = parser.parse(contents.clone(), None).unwrap();
    return tree;
}

pub fn parse_existing_cpp_file(text: String) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).expect("Error loading C++ parser.");
    let tree = parser.parse(text.clone(), None).unwrap();
    return tree;
}

pub fn print_tree(node: tree_sitter::Node, source: &str, indent_level: usize) {
    for _ in 0..indent_level {
        print!("  ");
    }
    println!(
        "{} [{} - {}]: {:?}",
        node.kind(),
        node.start_position(),
        node.end_position(),
        node.utf8_text(source.as_bytes()).unwrap()
    );
    
    for child in node.children(&mut node.walk()) {
        print_tree(child, source, indent_level + 1);
    }
}

