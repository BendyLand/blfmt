use std::{fs, fs::File, io::Write, io::Error};
use regex::Regex;
use crate::options::{self, TxtOpts};
use std::ops::{Bound, RangeBounds};
use std::cmp::Ordering;
use std::collections::HashMap;

pub trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn at(&self, current: usize) -> Option<char>;
}

impl StringUtils for str {
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            }
            else { break; }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len { break; }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            }
            else { break; }
        }
        return self.get(byte_start..byte_end).unwrap();
    }

    fn at(&self, current: usize) -> Option<char> {
        if current < self.len() {
            return self.chars().nth(current);
        }
        return None;
    }
}

pub enum Style {
    Allman,
    KnR,
    Stroustrup,
}

pub const COMMENT_OFFSET_SENTINEL: char = '\x01';
pub const COMMENT_COL0_SENTINEL: char = '\x02';

pub fn visual_col_width(ws: &str) -> usize {
    let mut col = 0usize;
    for ch in ws.chars() {
        if ch == '\t' {
            col += 4 - (col % 4);
        } else {
            col += 1;
        }
    }
    col
}

pub fn line_prefix_before(byte_idx: usize, src: &str) -> &str {
    let clamped = byte_idx.min(src.len());
    let prefix = &src[..clamped];
    match prefix.rfind('\n') {
        Some(nl) => &prefix[nl + 1..],
        None => prefix,
    }
}

pub fn trailing_comment_gap(start_byte: usize, src: &str) -> Option<String> {
    let before = line_prefix_before(start_byte, src);
    if before.trim().is_empty() {
        return None;
    }
    let trimmed = before.trim_end_matches(|c: char| c == ' ' || c == '\t');
    Some(before[trimmed.len()..].to_string())
}

pub fn has_code_after_on_line(end_byte: usize, src: &str) -> bool {
    let clamped = end_byte.min(src.len());
    let rest = &src[clamped..];
    let after = match rest.find('\n') {
        Some(nl) => &rest[..nl],
        None => rest,
    };
    !after.trim().is_empty()
}

pub fn format_standalone_comment_indent(
    raw: String,
    comment_start_byte: usize,
    ref_start_byte: Option<usize>,
    src: &str,
    is_top_level: bool,
) -> String {
    if trailing_comment_gap(comment_start_byte, src).is_some() {
        return raw;
    }
    let comment_ws = line_prefix_before(comment_start_byte, src);
    let comment_col = visual_col_width(comment_ws);

    if is_top_level {
        if comment_col > 0 {
            return format!("{}{}{}", COMMENT_OFFSET_SENTINEL, comment_ws, raw);
        }
        return raw;
    }

    let Some(ref_byte) = ref_start_byte else {
        if comment_col == 0 {
            return raw
                .split('\n')
                .map(|l| format!("{}{}", COMMENT_COL0_SENTINEL, l))
                .collect::<Vec<_>>()
                .join("\n");
        }
        return raw;
    };

    let ref_ws = line_prefix_before(ref_byte, src);
    // Only compare if ref_ws really is pure leading indentation
    if !ref_ws.trim().is_empty() {
        return raw;
    }
    let ref_col = visual_col_width(ref_ws);

    if comment_col == ref_col {
        return raw;
    }

    if comment_col == 0 && ref_col > 0 {
        return raw
            .split('\n')
            .map(|l| format!("{}{}", COMMENT_COL0_SENTINEL, l))
            .collect::<Vec<_>>()
            .join("\n");
    }

    if comment_col > ref_col {
        let extra_ws = if comment_ws.starts_with(ref_ws) {
            comment_ws[ref_ws.len()..].to_string()
        } else {
            " ".repeat(comment_col - ref_col)
        };
        let mut lines_iter = raw.split('\n');
        let mut out_lines = Vec::<String>::new();
        if let Some(first) = lines_iter.next() {
            out_lines.push(format!("{}{}{}", COMMENT_OFFSET_SENTINEL, extra_ws, first));
        }
        for sub in lines_iter {
            let sub_rest = if sub.starts_with(ref_ws) {
                &sub[ref_ws.len()..]
            } else {
                sub
            };
            out_lines.push(format!("{}{}", COMMENT_OFFSET_SENTINEL, sub_rest));
        }
        return out_lines.join("\n");
    }

    raw
}

pub fn attach_trailing_comment(
    target: &mut String,
    gap: &str,
    comment: &str,
    ensure_nl_for_line_comment: bool,
) {
    let mut nl_count = 0usize;
    while target.ends_with('\n') {
        target.pop();
        nl_count += 1;
    }
    while target.ends_with(' ') || target.ends_with('\t') {
        target.pop();
    }
    let clean_comment = comment
        .trim_start_matches(COMMENT_OFFSET_SENTINEL)
        .trim_start_matches(COMMENT_COL0_SENTINEL);
    target.push_str(gap);
    target.push_str(clean_comment);
    if nl_count > 0 {
        for _ in 0..nl_count {
            target.push('\n');
        }
    } else if ensure_nl_for_line_comment && clean_comment.trim_start().starts_with("//") {
        target.push('\n');
    }
}

pub fn append_comment_to_result(
    result: &mut String,
    comment: &str,
    start_byte: usize,
    src: &str,
    standalone_prefix: &str,
    standalone_suffix: &str,
    ensure_nl_for_line_comment: bool,
) -> bool {
    if !result.trim().is_empty() {
        if let Some(gap) = trailing_comment_gap(start_byte, src) {
            attach_trailing_comment(result, &gap, comment, ensure_nl_for_line_comment);
            return true;
        }
    }
    result.push_str(standalone_prefix);
    result.push_str(comment);
    result.push_str(standalone_suffix);
    false
}

pub fn push_comment_to_parts(
    parts: &mut Vec<String>,
    comment: &str,
    start_byte: usize,
    src: &str,
    indent_with_tab: bool,
) -> bool {
    if let Some(last) = parts.last_mut() {
        if !last.trim().is_empty() {
            if let Some(gap) = trailing_comment_gap(start_byte, src) {
                attach_trailing_comment(last, &gap, comment, false);
                return true;
            }
        }
    }
    if indent_with_tab {
        parts.push(add_all_leading_tabs(comment.to_string()));
    } else {
        parts.push(comment.to_string());
    }
    false
}

pub fn push_expr_comment(
    parts: &mut Vec<String>,
    comment: &str,
    start_byte: usize,
    end_byte: usize,
    src: &str,
) {
    let clean_trim = comment
        .trim_start_matches(COMMENT_OFFSET_SENTINEL)
        .trim_start_matches(COMMENT_COL0_SENTINEL)
        .trim_start();
    let is_line_comment = clean_trim.starts_with("//");
    let has_after = has_code_after_on_line(end_byte, src);

    if let Some(gap) = trailing_comment_gap(start_byte, src) {
        if let Some(last) = parts.last_mut() {
            while last.ends_with(' ') || last.ends_with('\t') {
                last.pop();
            }
            last.push_str(&gap);
            last.push_str(comment);
            if is_line_comment || !has_after {
                last.push_str("\n\t");
            }
            return;
        }
    }

    // Standalone comment inside an expression/header
    if let Some(last) = parts.last_mut() {
        if last.ends_with("\n\t") || last.ends_with('\n') {
            while last.ends_with('\t') || last.ends_with(' ') {
                last.pop();
            }
            last.push_str(comment);
            if is_line_comment || !has_after {
                last.push_str("\n\t");
            }
            return;
        }
    }
    if is_line_comment || !has_after {
        parts.push(format!("\n{}\n\t", comment));
    } else {
        parts.push(format!("\n{}", comment));
    }
}

pub fn append_expr_comment(
    result: &mut String,
    comment: &str,
    start_byte: usize,
    end_byte: usize,
    src: &str,
) {
    let clean_trim = comment
        .trim_start_matches(COMMENT_OFFSET_SENTINEL)
        .trim_start_matches(COMMENT_COL0_SENTINEL)
        .trim_start();
    let is_line_comment = clean_trim.starts_with("//");
    let has_after = has_code_after_on_line(end_byte, src);

    if let Some(gap) = trailing_comment_gap(start_byte, src) {
        if !result.trim().is_empty() {
            let had_trailing_space = result.ends_with(' ');
            while result.ends_with(' ') || result.ends_with('\t') {
                result.pop();
            }
            result.push_str(&gap);
            result.push_str(comment);
            if is_line_comment || !has_after {
                result.push_str("\n\t");
            } else if had_trailing_space {
                result.push(' ');
            }
            return;
        }
    }

    if !result.ends_with('\n') {
        while result.ends_with(' ') || result.ends_with('\t') {
            result.pop();
        }
        result.push('\n');
    }
    result.push_str(comment);
    if is_line_comment || !has_after {
        result.push_str("\n\t");
    }
}

pub fn strip_trailing_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;
    let mut last_code_end = 0usize;
    let mut comment_start: Option<usize> = None;

    while i < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
            i += 1;
            last_code_end = i;
            continue;
        }
        if b == b'\\' && (in_string || in_char) {
            escaped = true;
            i += 1;
            last_code_end = i;
            continue;
        }
        if b == b'"' && !in_char {
            in_string = !in_string;
            i += 1;
            last_code_end = i;
            continue;
        }
        if b == b'\'' && !in_string {
            in_char = !in_char;
            i += 1;
            last_code_end = i;
            continue;
        }
        if !in_string && !in_char && i + 1 < bytes.len() && b == b'/' {
            if bytes[i + 1] == b'/' {
                return line[..i].trim_end();
            }
            if bytes[i + 1] == b'*' {
                let start = i;
                i += 2;
                let mut closed = false;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        i += 2;
                        closed = true;
                        break;
                    }
                    i += 1;
                }
                if !closed {
                    return line[..start].trim_end();
                }
                if comment_start.is_none() {
                    comment_start = Some(start);
                }
                continue;
            }
        }
        if !b.is_ascii_whitespace() {
            comment_start = None;
            last_code_end = i + 1;
        }
        i += 1;
    }

    if let Some(cs) = comment_start {
        line[..cs].trim_end()
    } else {
        let _ = last_code_end;
        line
    }
}

fn has_unclosed_block_comment(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if b == b'\\' && (in_string || in_char) {
            escaped = true;
            i += 1;
            continue;
        }
        if b == b'"' && !in_char {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if b == b'\'' && !in_string {
            in_char = !in_char;
            i += 1;
            continue;
        }
        if !in_string && !in_char && i + 1 < bytes.len() && b == b'/' {
            if bytes[i + 1] == b'/' {
                return false;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                let mut closed = false;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        i += 2;
                        closed = true;
                        break;
                    }
                    i += 1;
                }
                if !closed {
                    return true;
                }
                continue;
            }
        }
        i += 1;
    }
    false
}

pub fn line_trimmed_for_comment_check(line: &str) -> &str {
    line.trim_start()
        .trim_start_matches(COMMENT_OFFSET_SENTINEL)
        .trim_start_matches(COMMENT_COL0_SENTINEL)
        .trim_start()
}

pub fn format_else_lines(file: &mut String, style: &Style) {
    match style {
        Style::KnR => return,
        Style::Allman => format_to_allman(file),
        Style::Stroustrup => format_to_stroustrup(file),
    }
}

fn split_keyword_to_newlines(line: &str, keyword: &str) -> String {
    let trimmed = line_trimmed_for_comment_check(line);
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with('#') {
        return line.to_string();
    }
    let indents = line.chars().take_while(|&c| c == '\t').count();
    let indent_str: String = "\t".repeat(indents);
    let chars: Vec<char> = line.chars().collect();
    let kw_chars: Vec<char> = keyword.chars().collect();
    let kw_len = kw_chars.len();
    let mut out = String::new();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            out.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char && i + 1 < chars.len() && c == '/' {
            if chars[i + 1] == '/' {
                for &rem in &chars[i..] {
                    out.push(rem);
                }
                break;
            }
            if chars[i + 1] == '*' {
                out.push('/');
                out.push('*');
                i += 2;
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                        out.push('*');
                        out.push('/');
                        i += 2;
                        break;
                    }
                    out.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        if !in_string && !in_char && i + kw_len <= chars.len() && &chars[i..i + kw_len] == kw_chars.as_slice() {
            let prev_is_word = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '#');
            let next_is_word = i + kw_len < chars.len() && (chars[i + kw_len].is_alphanumeric() || chars[i + kw_len] == '_');
            let only_indent_before = out.lines().last().map(|l| l.trim().is_empty()).unwrap_or(true);
            if !prev_is_word && !next_is_word && !only_indent_before {
                while out.ends_with(' ') || out.ends_with('\t') {
                    out.pop();
                }
                out.push('\n');
                out.push_str(&indent_str);
                out.push_str(keyword);
                i += kw_len;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    return out;
}

fn format_to_allman(file: &mut String) {
    let mut split_lines = Vec::<String>::new();
    for line in file.lines() {
        let s1 = split_keyword_to_newlines(line, "else");
        for sub in s1.split('\n') {
            let s2 = split_keyword_to_newlines(sub, "catch");
            for sub2 in s2.split('\n') {
                split_lines.push(sub2.to_string());
            }
        }
    }
    let mut lines = split_lines;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_start();
        if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') || line.starts_with('#') {
            i += 1;
            continue;
        }
        if let Some(pos) = lines[i].find('{') {
            if lines[i].trim() != "{" && !lines[i][pos + 1..].contains('}') {
                let indent = lines[i].chars().take_while(|&c| c == '\t').collect::<String>();
                let before = lines[i][..pos].trim_end().to_string();
                let after = lines[i][pos + 1..].to_string();
                lines[i] = before;
                lines.insert(i + 1, format!("{}{{{}", indent, after));
                i += 1;
            }
        }
        i += 1;
    }
    *file = lines.join("\n");
}

fn format_to_stroustrup(file: &mut String) {
    let mut result_lines = Vec::<String>::new();
    for line in file.lines() {
        let s1 = split_keyword_to_newlines(line, "else");
        for sub in s1.split('\n') {
            let s2 = split_keyword_to_newlines(sub, "catch");
            for sub2 in s2.split('\n') {
                result_lines.push(sub2.to_string());
            }
        }
    }
    *file = result_lines.join("\n");
}

pub fn tidy_up_loose_ends(file: &mut String, style: Style) {
    let mut lines: Vec<String> = file.lines().into_iter().map(|x| x.to_string()).collect();
    let lines_clone: Vec<String> = lines.clone();
    for (i, line) in lines_clone.into_iter().enumerate() {
        if line.contains(",") {
            lines[i] = ensure_space_after_char(&line, ',');
        }
    }
    *file = lines.join("\n");
    remove_blank_lines_from_blocks(file);
    ensure_no_consecutive_blank_lines(file);
    ensure_proper_doc_comment_spacing(file);
    fix_indentation_levels(file);
    file.retain(|c| c != COMMENT_OFFSET_SENTINEL && c != COMMENT_COL0_SENTINEL);
    shift_back_preproc_lines(file);
    join_single_line_constructs(file, style);
    ensure_blank_line_between_self_contained_blocks(file);
    ensure_no_consecutive_blank_lines(file);
    format_long_lines(file, 100);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScopeKind {
    StorageContainer,
    ExecutionOrValue,
}

fn skip_leading_template_and_typedef(hdr: &str) -> &str {
    let mut s = hdr.trim_start();
    loop {
        if let Some(after_kw) = s.strip_prefix("template") {
            if after_kw.starts_with(' ') || after_kw.starts_with('\t') || after_kw.starts_with('<') {
                let bytes = after_kw.as_bytes();
                let mut idx = 0usize;
                while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
                    idx += 1;
                }
                if idx < bytes.len() && bytes[idx] == b'<' {
                    let mut depth = 0usize;
                    while idx < bytes.len() {
                        if bytes[idx] == b'<' {
                            depth += 1;
                        } else if bytes[idx] == b'>' {
                            depth = depth.saturating_sub(1);
                            if depth == 0 {
                                idx += 1;
                                break;
                            }
                        }
                        idx += 1;
                    }
                    s = after_kw[idx..].trim_start();
                    continue;
                }
            }
        }
        if let Some(rest) = s.strip_prefix("typedef ") {
            s = rest.trim_start();
            continue;
        }
        if let Some(rest) = s.strip_prefix("inline ") {
            s = rest.trim_start();
            continue;
        }
        break;
    }
    s
}

fn has_function_paren_outside_angles(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut angle_depth = 0usize;
    let mut word = String::new();
    for i in 0..chars.len() {
        let c = chars[i];
        if c == '<' {
            angle_depth += 1;
            word.clear();
        } else if c == '>' {
            angle_depth = angle_depth.saturating_sub(1);
            word.clear();
        } else if angle_depth == 0 {
            if c.is_ascii_alphanumeric() || c == '_' {
                word.push(c);
            } else if c == '(' {
                if !matches!(
                    word.as_str(),
                    "alignas" | "__attribute__" | "__declspec" | "typeof" | "decltype"
                ) {
                    return true;
                }
                word.clear();
            } else if !c.is_whitespace() {
                word.clear();
            }
        }
    }
    false
}

fn skip_leading_extern_linkage(hdr: &str) -> &str {
    let s = hdr.trim_start();
    if let Some(after_ext) = s.strip_prefix("extern") {
        let after_ws = after_ext.trim_start();
        if let Some(after_open_q) = after_ws.strip_prefix('"') {
            if let Some(close_q) = after_open_q.find('"') {
                return after_open_q[close_q + 1..].trim_start();
            }
        }
    }
    s
}

fn has_top_level_assignment_eq(s: &str) -> bool {
    let rest = skip_leading_template_and_typedef(skip_leading_extern_linkage(s));
    let chars: Vec<char> = rest.chars().collect();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut angle_depth = 0usize;
    let mut current_prefix = String::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            current_prefix.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            current_prefix.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            current_prefix.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            current_prefix.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char {
            match c {
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '<' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                    let prev_is_ident = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
                    let next_c = chars.get(i + 1).copied();
                    if prev_is_ident && next_c != Some('<') && next_c != Some('=') && last_word(&current_prefix) != "operator" {
                        angle_depth += 1;
                    }
                }
                '>' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && angle_depth > 0 => {
                    if !(i > 0 && chars[i - 1] == '-') {
                        angle_depth = angle_depth.saturating_sub(1);
                    }
                }
                '=' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && angle_depth == 0 => {
                    // Allow `operator=`, `operator==`, `operator!=`, `operator+=`, `operator<<=`, `operator<=>`, etc.
                    let is_op_overload = if let Some(op_pos) = current_prefix.rfind("operator") {
                        let before_ok = op_pos == 0 || {
                            let prev = current_prefix[..op_pos].chars().last().unwrap_or(' ');
                            !prev.is_ascii_alphanumeric() && prev != '_'
                        };
                        let after_op = &current_prefix[op_pos + "operator".len()..];
                        before_ok && after_op.chars().all(|ch| ch.is_whitespace() || matches!(ch, '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '^' | '&' | '|'))
                    } else {
                        false
                    };
                    if !is_op_overload {
                        return true;
                    }
                }
                _ => {}
            }
        }
        current_prefix.push(c);
        i += 1;
    }
    false
}

fn is_storage_container_header(hdr: &str) -> bool {
    let trimmed = hdr.trim();
    if trimmed == "namespace"
        || trimmed.starts_with("namespace ")
        || trimmed.starts_with("inline namespace ")
    {
        return true;
    }
    if trimmed.starts_with("extern \"") {
        let after_ext = skip_leading_extern_linkage(trimmed);
        return after_ext.is_empty();
    }
    let rest = skip_leading_template_and_typedef(trimmed);
    let is_record = rest == "class"
        || rest.starts_with("class ")
        || rest == "struct"
        || rest.starts_with("struct ")
        || rest == "union"
        || rest.starts_with("union ");
    is_record && !has_function_paren_outside_angles(rest)
}

fn is_function_signature_header(line: &str) -> bool {
    let trimmed = line_trimmed_for_comment_check(line);
    if trimmed.is_empty()
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with('#')
        || has_unclosed_block_comment(line)
    {
        return false;
    }
    let code = strip_trailing_comment(line).trim();
    if code.is_empty()
        || code.ends_with(';')
        || code.ends_with('{')
        || code.ends_with('}')
        || code.ends_with(':')
        || code.ends_with(',')
        || code.ends_with('=')
        || has_top_level_assignment_eq(code)
    {
        return false;
    }
    if is_storage_container_header(code) {
        return false;
    }
    let rest = skip_leading_template_and_typedef(skip_leading_extern_linkage(code));
    if rest.is_empty()
        || rest == "namespace"
        || rest.starts_with("namespace ")
        || ((rest == "enum" || rest.starts_with("enum ")) && !has_function_paren_outside_angles(rest))
    {
        return false;
    }
    let first_w = rest
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .next()
        .unwrap_or("");
    if matches!(
        first_w,
        "if" | "else" | "for" | "while" | "do" | "switch" | "try" | "catch"
    ) {
        return false;
    }
    has_function_paren_outside_angles(rest) && rest.contains(')')
}

fn is_empty_function_body_header(lines: &[String], idx: usize) -> bool {
    if is_function_signature_header(&lines[idx]) {
        return true;
    }
    let trimmed = line_trimmed_for_comment_check(&lines[idx]);
    if trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with('#')
        || has_unclosed_block_comment(&lines[idx])
    {
        return false;
    }
    let code = strip_trailing_comment(&lines[idx]).trim();
    if code.starts_with(": ") && idx > 0 && is_function_signature_header(&lines[idx - 1]) {
        return true;
    }
    false
}

fn is_single_line_self_contained_block(code: &str) -> bool {
    if code.starts_with(": ") && code.ends_with("{}") {
        return true;
    }
    let brace_pos = if code.ends_with("{}") {
        code.len() - 2
    } else if let Some(pos) = code.find('{') {
        pos
    } else {
        return false;
    };
    if !(code.ends_with('}') || code.ends_with("};")) {
        return false;
    }
    let before = code[..brace_pos].trim();
    if before.is_empty() || has_top_level_assignment_eq(before) {
        return false;
    }
    let rest = skip_leading_template_and_typedef(skip_leading_extern_linkage(before));
    if rest == "enum"
        || rest.starts_with("enum ")
        || rest == "struct"
        || rest.starts_with("struct ")
        || rest == "union"
        || rest.starts_with("union ")
        || rest == "class"
        || rest.starts_with("class ")
        || rest == "namespace"
        || rest.starts_with("namespace ")
        || before.starts_with("extern \"") && rest.is_empty()
    {
        return true;
    }
    if before.contains('(') && before.contains(')') {
        let first_w = rest.split(|c: char| !c.is_ascii_alphanumeric() && c != '_').next().unwrap_or("");
        if !matches!(first_w, "if" | "while" | "for" | "switch" | "catch") {
            return true;
        }
    }
    false
}

fn count_braces_outside_literals(code: &str) -> (usize, usize) {
    let chars: Vec<char> = code.chars().collect();
    let mut opens = 0usize;
    let mut closes = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            i += 1;
            continue;
        }
        if !in_string && !in_char {
            if i + 1 < chars.len() && c == '/' {
                if chars[i + 1] == '/' {
                    break;
                }
                if chars[i + 1] == '*' {
                    i += 2;
                    while i + 1 < chars.len() {
                        if chars[i] == '*' && chars[i + 1] == '/' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }
            }
            if c == '{' {
                opens += 1;
            } else if c == '}' {
                closes += 1;
            }
        }
        i += 1;
    }
    (opens, closes)
}

fn find_raw_string_end_delim(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i + 2 < bytes.len() {
        if bytes[i] == b'R' && bytes[i + 1] == b'"' {
            let prev_ok = i == 0 || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            if prev_ok {
                if let Some(rel_open) = line[i + 2..].find('(') {
                    let delim = &line[i + 2..i + 2 + rel_open];
                    if !delim.contains(' ') && !delim.contains('\\') && !delim.contains('"') {
                        let end_pat = format!("){}\"", delim);
                        let after_open = &line[i + 2 + rel_open + 1..];
                        if !after_open.contains(&end_pat) {
                            return Some(end_pat);
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn mark_blank_before_block_header(
    lines: &[String],
    in_macro: &[bool],
    in_comment: &[bool],
    in_raw_str: &[bool],
    brace_line_idx: usize,
    brace_is_standalone: bool,
    insert_before: &mut [bool],
) -> String {
    let brace_tabs = lines[brace_line_idx].chars().take_while(|&c| c == '\t').count();
    let mut hdr_end = brace_line_idx;
    let mut h = brace_line_idx;

    if brace_is_standalone && h > 0 {
        h -= 1;
        hdr_end = h + 1;
        while h > 0 && !in_macro[h - 1] && !in_raw_str[h - 1] {
            let ht = lines[h].chars().take_while(|&c| c == '\t').count();
            let hs = lines[h].trim_start();
            if ht > brace_tabs || hs.starts_with(':') || hs.starts_with(')') {
                h -= 1;
            } else {
                break;
            }
        }
    } else if !brace_is_standalone {
        while h > 0 && !in_macro[h - 1] && !in_raw_str[h - 1] {
            let hs = lines[h].trim_start();
            if hs.starts_with(':') || hs.starts_with(')') {
                h -= 1;
            } else {
                break;
            }
        }
    }

    while h > 0 && !in_macro[h - 1] && !in_comment[h - 1] {
        let prev_hs = lines[h - 1].trim_start();
        if (prev_hs.starts_with("template ") || prev_hs.starts_with("template<"))
            && !prev_hs.trim_end().ends_with(';')
        {
            h -= 1;
        } else {
            break;
        }
    }

    let hdr_text = if brace_is_standalone && h < hdr_end {
        lines[h..hdr_end]
            .iter()
            .map(|s| strip_trailing_comment(s).trim())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        let code = strip_trailing_comment(&lines[brace_line_idx]).trim();
        code.strip_suffix('{').unwrap_or(code).trim().to_string()
    };

    // Step above any attached doc/line comments immediately preceding this block
    let mut top = h;
    while top > 0 {
        let prev_t = lines[top - 1].trim();
        if in_comment[top - 1]
            || prev_t.starts_with("//")
            || prev_t.starts_with("/*")
            || prev_t.starts_with('*')
        {
            top -= 1;
        } else {
            break;
        }
    }

    if top > 0 {
        let above = &lines[top - 1];
        let above_trim = above.trim();
        let above_code = strip_trailing_comment(above).trim();
        let is_container_or_section_open = above_trim.is_empty()
            || in_macro[top - 1]
            || above.trim_end().ends_with('\\')
            || above_code == "{"
            || above_code.ends_with('{')
            || matches!(above_code, "public:" | "private:" | "protected:")
            || above_trim.starts_with("#ifdef")
            || above_trim.starts_with("#ifndef")
            || above_trim.starts_with("#if ")
            || above_trim.starts_with("#if(")
            || above_trim.starts_with("#elif")
            || above_trim.starts_with("#else");
        if !is_container_or_section_open {
            insert_before[top] = true;
        }
    }

    hdr_text
}

fn mark_blank_after_block_close(lines: &[String], close_line_idx: usize, insert_before: &mut [bool]) {
    let n = lines.len();
    if close_line_idx + 1 >= n {
        return;
    }
    let nxt_trim = lines[close_line_idx + 1].trim();
    let nxt_code = strip_trailing_comment(&lines[close_line_idx + 1]).trim();
    if !nxt_trim.is_empty()
        && !nxt_code.starts_with('}')
        && !nxt_trim.starts_with("#endif")
        && !nxt_trim.starts_with("#else")
        && !nxt_trim.starts_with("#elif")
    {
        insert_before[close_line_idx + 1] = true;
    }
}

fn ensure_blank_line_between_self_contained_blocks(file: &mut String) {
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    if lines.is_empty() {
        return;
    }

    let n = lines.len();
    let mut in_macro = vec![false; n];
    let mut in_comment = vec![false; n];
    let mut in_raw_str = vec![false; n];

    let mut macro_cont = false;
    let mut block_comment = false;
    let mut raw_end: Option<String> = None;

    for i in 0..n {
        let l = &lines[i];
        let t = l.trim();

        if let Some(ref end_pat) = raw_end {
            in_raw_str[i] = true;
            if l.contains(end_pat.as_str()) {
                raw_end = None;
            }
            continue;
        }

        if block_comment {
            in_comment[i] = true;
            if t.contains("*/") {
                block_comment = false;
            }
            continue;
        }

        if macro_cont {
            in_macro[i] = true;
            macro_cont = l.trim_end().ends_with('\\');
            continue;
        }

        if t.starts_with('#') && l.trim_end().ends_with('\\') {
            in_macro[i] = true;
            macro_cont = true;
            continue;
        }

        if t.starts_with("/*") {
            in_comment[i] = true;
            if !t.contains("*/") {
                block_comment = true;
            }
            continue;
        }

        if has_unclosed_block_comment(l) {
            block_comment = true;
        }

        if let Some(end_pat) = find_raw_string_end_delim(l) {
            raw_end = Some(end_pat);
        }
    }

    let mut insert_before = vec![false; n];
    // Stack of (ScopeKind, is_self_contained_block). Index 0 is the root file scope.
    let mut stack: Vec<(ScopeKind, bool)> = vec![(ScopeKind::StorageContainer, false)];
    let mut preproc_saved_stacks: Vec<Vec<(ScopeKind, bool)>> = Vec::new();

    for i in 0..n {
        if in_macro[i] || in_comment[i] || in_raw_str[i] {
            continue;
        }
        let l = &lines[i];
        let trimmed = line_trimmed_for_comment_check(l);
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        if trimmed.starts_with('#') {
            if trimmed.starts_with("#ifdef")
                || trimmed.starts_with("#ifndef")
                || trimmed.starts_with("#if ")
                || trimmed.starts_with("#if(")
            {
                preproc_saved_stacks.push(stack.clone());
            } else if trimmed.starts_with("#elif") || trimmed.starts_with("#else") {
                if let Some(saved) = preproc_saved_stacks.last() {
                    stack = saved.clone();
                }
            } else if trimmed.starts_with("#endif") {
                preproc_saved_stacks.pop();
            }
            continue;
        }

        let code = strip_trailing_comment(l).trim();
        if code.is_empty() {
            continue;
        }

        let (opens, closes) = count_braces_outside_literals(code);
        if opens == 0 && closes == 0 {
            continue;
        }

        let in_storage = stack.last().map(|(k, _)| *k == ScopeKind::StorageContainer).unwrap_or(true);

        // Single-line balanced braces on the same line (e.g. `enum Foo { A, B };`, `Payload p{};`, `if (x) { return; }`)
        if opens == closes {
            // Handle `} else {` / `} catch (...) {` where closes == 1 and opens == 1, starting with `}`
            if code.starts_with('}') && code.ends_with('{') {
                if stack.len() > 1 {
                    stack.pop();
                }
                stack.push((ScopeKind::ExecutionOrValue, false));
                continue;
            }
            if in_storage && is_single_line_self_contained_block(code) {
                let _ = mark_blank_before_block_header(
                    &lines,
                    &in_macro,
                    &in_comment,
                    &in_raw_str,
                    i,
                    false,
                    &mut insert_before,
                );
                mark_blank_after_block_close(&lines, i, &mut insert_before);
            }
            continue;
        }

        // Net closing braces on this line (`}`, `};`, `} as;`, `} while (...);`, etc.)
        if closes > opens {
            let diff = closes - opens;
            let mut closed_self_contained = false;
            for _ in 0..diff {
                if stack.len() > 1 {
                    if let Some((_, was_sc)) = stack.pop() {
                        if was_sc {
                            closed_self_contained = true;
                        }
                    }
                }
            }
            if closed_self_contained
                && !code.starts_with("} else")
                && !code.starts_with("} while")
                && !code.starts_with("} catch")
            {
                mark_blank_after_block_close(&lines, i, &mut insert_before);
            }
            continue;
        }

        // Net opening braces on this line (`opens > closes`)
        let diff = opens - closes;
        if in_storage && diff == 1 {
            let brace_is_standalone = code == "{";
            let is_var_brace_init = !brace_is_standalone && code.contains('=');
            if !is_var_brace_init && (brace_is_standalone || is_storage_container_header(code.strip_suffix('{').unwrap_or(code))) {
                let hdr = mark_blank_before_block_header(
                    &lines,
                    &in_macro,
                    &in_comment,
                    &in_raw_str,
                    i,
                    brace_is_standalone,
                    &mut insert_before,
                );
                let child_kind = if is_storage_container_header(&hdr) {
                    ScopeKind::StorageContainer
                } else {
                    ScopeKind::ExecutionOrValue
                };
                stack.push((child_kind, true));
                continue;
            }
        }

        for _ in 0..diff {
            stack.push((ScopeKind::ExecutionOrValue, false));
        }
    }

    let mut out = Vec::<String>::with_capacity(n + 64);
    for i in 0..n {
        if insert_before[i] && !out.last().map(|s| s.trim().is_empty()).unwrap_or(true) && !lines[i].trim().is_empty() {
            out.push(String::new());
        }
        out.push(lines[i].clone());
    }
    *file = out.join("\n") + "\n";
}

pub fn detect_indentation(line: &String) -> usize {
    let mut result = 0;
    for c in line.chars() {
        if c != '\t' { break; }
        result += 1;
    }
    return result;
}

fn fix_indentation(line: &String, current: usize, target: usize) -> String {
    let mut prefix = String::new();
    let result: String;
    if current < target {
        for _ in current..target {
            prefix += "\t";
        }        
        result = format!("{}{}", prefix, line);
    }
    else {
        let mut temp = line.clone();
        for _ in target..current {
            temp = remove_single_tab(&temp);
        }
        result = temp;
    }
    return result;
}

fn fix_indentation_levels(file: &mut String) {
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    let mut indent_level: usize = 0;
    let mut brace_stack = Vec::<bool>::new();
    let mut pending_no_indent_brace = false;
    let mut in_block_comment = false;

    for line in lines.into_iter() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        if let Some(pos) = line.find(COMMENT_COL0_SENTINEL) {
            let payload = &line[pos + COMMENT_COL0_SENTINEL.len_utf8()..];
            let payload_trim = payload.trim();
            if in_block_comment {
                if payload_trim.contains("*/") {
                    in_block_comment = false;
                }
            } else if payload_trim.starts_with("/*") && !payload_trim.contains("*/") {
                in_block_comment = true;
            }
            result.push(payload.to_string());
            continue;
        }

        if let Some(pos) = line.find(COMMENT_OFFSET_SENTINEL) {
            let payload = &line[pos + COMMENT_OFFSET_SENTINEL.len_utf8()..];
            let payload_trim = payload.trim();
            if in_block_comment {
                if payload_trim.contains("*/") {
                    in_block_comment = false;
                }
            } else if payload_trim.starts_with("/*") && !payload_trim.contains("*/") {
                in_block_comment = true;
            }
            result.push(format!("{}{}", "\t".repeat(indent_level), payload));
            continue;
        }

        if in_block_comment {
            if indent_level == 0 {
                result.push(line.clone());
            } else {
                let current_level = detect_indentation(&line);
                result.push(fix_indentation(&line, current_level, indent_level));
            }
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            if indent_level == 0 {
                result.push(line.clone());
            } else {
                let current_level = detect_indentation(&line);
                result.push(fix_indentation(&line, current_level, indent_level));
            }
            continue;
        }

        if trimmed.starts_with("//") {
            let current_level = detect_indentation(&line);
            result.push(fix_indentation(&line, current_level, indent_level));
            continue;
        }

        if trimmed.starts_with('#') {
            result.push(trimmed.to_string());
            continue;
        }

        let code_part = strip_trailing_comment(trimmed);

        if (code_part.starts_with("namespace ") || code_part == "namespace" || code_part.starts_with("inline namespace "))
            && !code_part.contains(';')
            && !code_part.contains('=')
        {
            if code_part.ends_with('{') {
                let current_level = detect_indentation(&line);
                result.push(fix_indentation(&line, current_level, indent_level));
                brace_stack.push(false);
                if has_unclosed_block_comment(&line) {
                    in_block_comment = true;
                }
                continue;
            } else {
                pending_no_indent_brace = true;
            }
        } else if code_part.starts_with("extern \"") && code_part.ends_with('{') {
            let current_level = detect_indentation(&line);
            result.push(fix_indentation(&line, current_level, indent_level));
            brace_stack.push(false);
            if has_unclosed_block_comment(&line) {
                in_block_comment = true;
            }
            continue;
        } else if code_part.starts_with("extern \"") && !code_part.contains(';') && !code_part.ends_with("{}") {
            pending_no_indent_brace = true;
        }

        if code_part == "}" || code_part.starts_with('}') {
            let did_indent = brace_stack.pop().unwrap_or(true);
            if did_indent && indent_level > 0 {
                indent_level -= 1;
            }
        }

        let target_indent = if code_part.starts_with(": ") {
            indent_level + 1
        } else {
            indent_level
        };
        let current_level = detect_indentation(&line);
        if current_level != target_indent {
            let temp = fix_indentation(&line, current_level, target_indent);
            result.push(temp);
        } else {
            result.push(line.clone());
        }

        if code_part == "{" || (code_part.ends_with('{') && !code_part.ends_with("{}")) {
            if pending_no_indent_brace {
                brace_stack.push(false);
                pending_no_indent_brace = false;
            } else {
                brace_stack.push(true);
                indent_level += 1;
            }
        } else if code_part != "{" && !code_part.starts_with("namespace") && !code_part.starts_with("inline namespace") && !code_part.starts_with("extern \"") {
            pending_no_indent_brace = false;
        }

        if has_unclosed_block_comment(&line) {
            in_block_comment = true;
        }
    }
    *file = result.join("\n") + "\n";
}

fn line_is_function_def(line: &String, pattern: &regex::Regex) -> bool {
    return pattern.is_match(line)
}

fn line_is_doc_comment(line: &String) -> bool {
    return line.trim_start().starts_with("*") && line.contains("*/");
}

fn find_next_function_def(remaining_lines: &[String], pattern: &regex::Regex) -> i32 {
    for (i, line) in remaining_lines.into_iter().enumerate() {
        if line_is_function_def(line, &pattern) {
            return i as i32;
        }
    }
    return -1;
}

fn find_next_doc_comment(remaining_lines: &[String]) -> i32 {
    for (i, line) in remaining_lines.into_iter().enumerate() {
        if line.trim_start().starts_with("*") && line.contains("*/") {
            return i as i32;
        }
    }
    return -1;
}

fn ensure_proper_doc_comment_spacing(file: &mut String) {
    let mut lines: Vec<String> = file.trim().lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    let pattern = Regex::new("^(\\w*\\s*)*\\w+\\s+\\w+\\(.*\\)\\s*;?\\s*$").unwrap();
    let mut lines_to_remove = Vec::<usize>::new();
    for (i, line) in lines.clone().into_iter().enumerate() {
        if i > 0 && i < lines.len()-1 {
            if line.trim().is_empty() {
                if line_is_doc_comment(lines.iter().nth(i-1).unwrap()) &&
                   line_is_function_def(lines.iter().nth(i+1).unwrap(), &pattern) {
                   continue;
               }
            }
        }
        result.push(line);
    }
    *file = result.join("\n").trim_end().to_string() + "\n";
}

pub fn ensure_space_after_char(line: &String, target: char) -> String {
    let trimmed = line_trimmed_for_comment_check(line);
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with('#') {
        return line.clone();
    }
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            result.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char && i + 1 < chars.len() && c == '/' {
            if chars[i + 1] == '/' {
                for &rem in &chars[i..] {
                    result.push(rem);
                }
                break;
            }
            if chars[i + 1] == '*' {
                result.push('/');
                result.push('*');
                i += 2;
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                        result.push('*');
                        result.push('/');
                        i += 2;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        result.push(c);
        if !in_string && !in_char && i + 1 < chars.len() {
            if c == target && chars[i + 1] != ' ' && chars[i + 1] != target && chars[i + 1] != '=' {
                result.push(' ');
            }
        }
        i += 1;
    }
    return result;
}


fn remove_blank_lines_from_blocks(file: &mut String) {
    let lines: Vec<String> = file.trim().lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    let mut in_block = false;
    let mut prev_non_empty = String::new();
    for line in lines.into_iter() {
        let trimmed = line.trim();
        let is_ns_or_extern = prev_non_empty.starts_with("namespace ")
            || prev_non_empty == "namespace"
            || prev_non_empty.starts_with("inline namespace ")
            || prev_non_empty.starts_with("extern \"");
        let is_type_def = prev_non_empty.starts_with("class ")
            || prev_non_empty.starts_with("struct ")
            || prev_non_empty.starts_with("typedef struct")
            || prev_non_empty.starts_with("union ")
            || prev_non_empty.starts_with("typedef union")
            || prev_non_empty.starts_with("enum ")
            || prev_non_empty.starts_with("typedef enum");
        let at_block_end = in_block && (line.starts_with('}') || line == "}" || line == "};");
        if !in_block && line == "{" && !is_ns_or_extern && !is_type_def {
            in_block = true;
        } else if at_block_end {
            in_block = false;
        }
        if !trimmed.is_empty() {
            prev_non_empty = trimmed.to_string();
        }
        if in_block && !check_line_is_blank(&line) {
            result.push(line);
        } else if !in_block {
            result.push(line);
        }
    }
    *file = result.join("\n").trim_end().to_string() + "\n";
}

fn ensure_no_consecutive_blank_lines(file: &mut String) {
    let mut lines: Vec<String> = file.trim().lines().map(|x| x.to_string()).collect();
    let mut result = Vec::<String>::new();
    for (i, line) in lines.clone().into_iter().enumerate() {
        if i < lines.len()-1 {
            let line_empty = check_line_is_blank(&line);
            if line_empty {
                if !check_line_is_blank(&result[result.len()-1]) {
                    result.push(line.clone());
                }
                continue;
            }
        }
        result.push(line);
    }
    *file = result.join("\n").trim_end().to_string() + "\n";
}

//TODO: make a function that scans for the specific constructs that get aggressively formatted (e.g., maps)
pub fn scan_for_lines_before_blank_lines(file: String) -> Vec<(usize, String)> {
    let mut result = Vec::<(usize, String)>::new();
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut open_braces = 0;
    for (i, line) in lines.clone().into_iter().enumerate() {
        // only add lines that are inside of blocks
        if line.contains("{") { open_braces += 1; }
        if line.contains("}") { open_braces -= 1; }
        if open_braces > 0 {
            if i < lines.len()-1  {
                let has_no_visible_chars = {
                    lines[i+1]
                        .chars()
                        .all(|x| x.is_whitespace() || x == '\t')
                };
                if lines[i+1].is_empty() || has_no_visible_chars {
                    result.push((i+1, line));
                }
            }
        }
    }
    return result;
}

fn spaces_to_tabs(line: &String) -> String {
    return line.replace("    ", "\t");
}

fn display_width(line: &str) -> usize {
    let mut col = 0;
    for c in line.chars() {
        if c == '\t' {
            col += 4;
        } else {
            col += 1;
        }
    }
    return col;
}

pub fn is_standalone_statement_macro_token(tok: &str) -> bool {
    if tok.len() < 3 || matches!(tok, "NULL" | "TRUE" | "FALSE" | "EOF") {
        return false;
    }
    let first = tok.chars().next().unwrap_or(' ');
    if !(first.is_ascii_uppercase() || first == '_') {
        return false;
    }
    let mut has_upper = false;
    for c in tok.chars() {
        if c.is_ascii_uppercase() {
            has_upper = true;
        } else if !(c.is_ascii_digit() || c == '_') {
            return false;
        }
    }
    has_upper
}

pub fn merge_incomplete_decl_into_parts(parts: &mut Vec<String>, continuation: &str) {
    if let Some(last) = parts.last_mut() {
        let mut nl_count = 0usize;
        while last.ends_with('\n') {
            last.pop();
            nl_count += 1;
        }
        while last.ends_with(' ') || last.ends_with('\t') {
            last.pop();
        }
        last.push(' ');
        last.push_str(continuation.trim_start());
        for _ in 0..nl_count {
            last.push('\n');
        }
    } else {
        parts.push(continuation.to_string());
    }
}

pub fn merge_incomplete_decl_into_result(result: &mut String, continuation: &str) {
    let mut nl_count = 0usize;
    while result.ends_with('\n') {
        result.pop();
        nl_count += 1;
    }
    while result.ends_with(' ') || result.ends_with('\t') {
        result.pop();
    }
    result.push(' ');
    result.push_str(continuation.trim_start());
    for _ in 0..nl_count {
        result.push('\n');
    }
}

fn is_clean_paren_suffix(s: &str) -> bool {
    if s.is_empty() || s == ";" || s == "," || s == "{}" {
        return true;
    }
    let has_term = s.ends_with(';') || s.ends_with("{}");
    let without_term = s
        .strip_suffix(';')
        .or_else(|| s.strip_suffix("{}"))
        .unwrap_or(s)
        .trim();
    if without_term.is_empty() {
        return true;
    }
    if without_term.contains('{') || without_term.contains('[') {
        return false;
    }
    if without_term.starts_with("const")
        || without_term.starts_with("override")
        || without_term.starts_with("final")
        || without_term.starts_with("noexcept")
        || without_term.starts_with("->")
        || without_term == "= 0"
        || without_term == "= default"
        || without_term == "= delete"
    {
        return true;
    }
    // Trailing attribute macro on a function declaration/definition, e.g. `) _PRINTF_FORMAT_ATTRIBUTE_2_0;` or `) _PRINTF_FORMAT_ATTRIBUTE_2_0 {}`
    if has_term {
        let first_ch = without_term.chars().next().unwrap_or(' ');
        let has_expr_ops = without_term.contains("&&")
            || without_term.contains("||")
            || without_term.contains("==")
            || without_term.contains("!=")
            || without_term.contains('?')
            || without_term.contains('+');
        if (first_ch == '_' || first_ch.is_ascii_alphabetic()) && !has_expr_ops {
            return true;
        }
    }
    false
}

fn split_top_level_args(chars: &[char]) -> Vec<String> {
    let mut args = Vec::<String>::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut angle_depth = 0usize;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            current.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            current.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            current.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            current.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char {
            if i + 1 < chars.len() && c == '/' && chars[i + 1] == '*' {
                current.push('/');
                current.push('*');
                i += 2;
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                        current.push('*');
                        current.push('/');
                        i += 2;
                        break;
                    }
                    current.push(chars[i]);
                    i += 1;
                }
                continue;
            }
            match c {
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '<' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                    let prev_is_ident = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
                    let next_c = chars.get(i + 1).copied();
                    if prev_is_ident && next_c != Some('<') && next_c != Some('=') && last_word(&current) != "operator" {
                        angle_depth += 1;
                    }
                }
                '>' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && angle_depth > 0 => {
                    if !(i > 0 && chars[i - 1] == '-') {
                        angle_depth = angle_depth.saturating_sub(1);
                    }
                }
                ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 && angle_depth == 0 => {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        args.push(trimmed);
                    }
                    current.clear();
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }
        current.push(c);
        i += 1;
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        args.push(trimmed);
    }
    args
}

fn has_top_level_logical_op(code: &str) -> bool {
    let chars: Vec<char> = code.chars().collect();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            i += 1;
            continue;
        }
        if !in_string && !in_char {
            if i + 1 < chars.len() && c == '/' && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                '|' | '&'
                    if depth == 0
                        && i > 0
                        && i + 2 < chars.len()
                        && chars[i - 1] == ' '
                        && chars[i + 1] == c
                        && chars[i + 2] == ' ' =>
                {
                    return true;
                }
                _ => {}
            }
        }
        i += 1;
    }
    false
}

fn try_split_concatenated_strings(line: &str, base_tabs: usize) -> Option<Vec<String>> {
    let code_only = strip_trailing_comment(line);
    let code_part = code_only.trim_end();
    let trailing_comment = &line[code_part.len()..];
    let trimmed = code_part.trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let mut segments = Vec::<String>::new();
    let mut i = 0usize;

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        if chars[i] == ',' || chars[i] == ';' {
            if i + 1 == chars.len() {
                if let Some(last) = segments.last_mut() {
                    last.push(chars[i]);
                }
                break;
            }
            return None;
        }
        if chars[i] != '"' {
            return None;
        }
        let mut lit = String::new();
        lit.push('"');
        i += 1;
        let mut escaped = false;
        let mut closed = false;
        while i < chars.len() {
            let c = chars[i];
            lit.push(c);
            i += 1;
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            if c == '"' {
                closed = true;
                break;
            }
        }
        if !closed {
            return None;
        }
        segments.push(lit);
    }

    if segments.len() < 2 {
        return None;
    }

    let indent = "\t".repeat(base_tabs);
    let mut out: Vec<String> = segments
        .into_iter()
        .map(|seg| format!("{}{}", indent, seg))
        .collect();
    if !trailing_comment.is_empty() {
        if let Some(last) = out.last_mut() {
            last.push_str(trailing_comment);
        }
    }
    Some(out)
}

fn try_expand_paren_call(line: &str, target_len: usize, rec_depth: usize) -> Option<Vec<String>> {
    let base_tabs = line.chars().take_while(|&c| c == '\t').count();
    let base_indent = "\t".repeat(base_tabs);
    let arg_indent = "\t".repeat(base_tabs + 1);

    let code_only = strip_trailing_comment(line);
    let code_part = code_only.trim_end();
    let trailing_comment = &line[code_part.len()..];

    let chars: Vec<char> = code_part.chars().collect();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut current_open: Option<usize> = None;
    let mut candidates = Vec::<(usize, usize)>::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            i += 1;
            continue;
        }
        if !in_string && !in_char {
            if i + 1 < chars.len() && c == '/' && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            match c {
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '(' => {
                    if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 {
                        current_open = Some(i);
                    }
                    paren_depth += 1;
                }
                ')' => {
                    if paren_depth == 1 && bracket_depth == 0 && brace_depth == 0 {
                        if let Some(open_idx) = current_open.take() {
                            candidates.push((open_idx, i));
                        }
                    }
                    paren_depth = paren_depth.saturating_sub(1);
                }
                _ => {}
            }
        }
        i += 1;
    }

    for (open_idx, close_idx) in candidates {
        if open_idx == 0 {
            continue;
        }
        let prev_c = chars[open_idx - 1];
        let before_str: String = chars[..open_idx].iter().collect();
        let prev_w = last_word(&before_str);
        if matches!(
            prev_w,
            "if" | "while" | "for" | "switch" | "catch" | "sizeof" | "alignof" | "return" | "case" | "delete" | "throw"
        ) {
            continue;
        }
        let is_call_like_open = prev_c.is_alphanumeric()
            || matches!(prev_c, '_' | '>' | ')' | ']')
            || (before_str.contains("operator") && !prev_c.is_whitespace());
        if !is_call_like_open {
            continue;
        }

        let after_close_raw: String = chars[close_idx + 1..].iter().collect();
        if !is_clean_paren_suffix(after_close_raw.trim()) {
            continue;
        }

        let args = split_top_level_args(&chars[open_idx + 1..close_idx]);
        if args.is_empty() {
            continue;
        }

        let prefix_with_open: String = chars[..=open_idx].iter().collect();
        if args.len() == 1 {
            let arg0 = &args[0];
            let worthwhile = display_width(&prefix_with_open) <= target_len
                && (display_width(arg0) > 12 || arg0.contains('"') || arg0.contains('('));
            if !worthwhile {
                continue;
            }
        }

        let mut out = Vec::<String>::new();
        out.push(prefix_with_open);
        for (idx, arg) in args.iter().enumerate() {
            let suffix = if idx + 1 < args.len() { "," } else { "" };
            let arg_line = format!("{}{}{}", arg_indent, arg, suffix);
            if rec_depth < 4 && display_width(strip_trailing_comment(&arg_line)) > target_len {
                out.extend(wrap_single_long_line_depth(&arg_line, target_len, rec_depth + 1));
            } else {
                out.push(arg_line);
            }
        }
        out.push(format!("{}{}){}{}", base_indent, "", after_close_raw, trailing_comment));
        return Some(out);
    }

    None
}

fn wrap_single_long_line(line: &str, target_len: usize) -> Vec<String> {
    wrap_single_long_line_depth(line, target_len, 0)
}

fn wrap_single_long_line_depth(line: &str, target_len: usize, rec_depth: usize) -> Vec<String> {
    let trimmed = line_trimmed_for_comment_check(line);
    let code_only = strip_trailing_comment(line);
    if display_width(code_only) <= target_len
        || trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || trimmed.starts_with('#')
        || trimmed.starts_with("for ")
        || trimmed.starts_with("for(")
        || line.contains("R\"")
    {
        return vec![line.to_string()];
    }

    let base_tabs = line.chars().take_while(|&c| c == '\t').count();

    if let Some(str_lines) = try_split_concatenated_strings(line, base_tabs) {
        return str_lines;
    }

    if rec_depth < 4 && !has_top_level_logical_op(code_only) {
        if let Some(expanded) = try_expand_paren_call(line, target_len, rec_depth) {
            return expanded;
        }
    }

    let cont_indent = "\t".repeat(base_tabs + 1);
    let mut output = Vec::<String>::new();
    let mut current = line.to_string();

    for _ in 0..16 {
        if display_width(strip_trailing_comment(&current)) <= target_len {
            break;
        }
        let chars: Vec<char> = current.chars().collect();
        let mut col = 0usize;
        let mut in_string = false;
        let mut in_char = false;
        let mut escaped = false;
        let mut best_comma: Option<(usize, usize)> = None; // (char_idx_after_comma, depth)
        let mut fallback_comma: Option<usize> = None;
        let mut best_logic: Option<usize> = None;
        let mut depth = 0usize;

        for i in 0..chars.len() {
            let c = chars[i];
            col += if c == '\t' { 4 } else { 1 };
            if escaped {
                escaped = false;
                continue;
            }
            if c == '\\' && (in_string || in_char) {
                escaped = true;
                continue;
            }
            if c == '"' && !in_char {
                in_string = !in_string;
                continue;
            }
            if c == '\'' && !in_string {
                in_char = !in_char;
                continue;
            }
            if in_string || in_char {
                continue;
            }
            if i + 1 < chars.len() && c == '/' && (chars[i + 1] == '/' || chars[i + 1] == '*') {
                break;
            }
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ',' if i + 1 < chars.len() && chars[i + 1] == ' ' => {
                    if col <= target_len {
                        match best_comma {
                            None => best_comma = Some((i + 1, depth)),
                            Some((_, best_d)) if depth <= best_d => best_comma = Some((i + 1, depth)),
                            _ => {}
                        }
                        fallback_comma = Some(i + 1);
                    } else if best_comma.is_none() && fallback_comma.is_none() {
                        fallback_comma = Some(i + 1);
                        break;
                    }
                }
                '|' | '&' if i + 2 < chars.len() && i > 0 && chars[i - 1] == ' ' && chars[i + 1] == c && chars[i + 2] == ' ' => {
                    if col + 2 <= target_len {
                        best_logic = Some(i + 2);
                    }
                }
                _ => {}
            }
        }

        let split_at = best_comma
            .map(|(idx, _)| idx)
            .or(fallback_comma)
            .or(best_logic);

        if let Some(idx) = split_at {
            let left: String = chars[..idx].iter().collect();
            let right_raw: String = chars[idx..].iter().collect();
            let right_trimmed = right_raw.trim_start();
            if right_trimmed.is_empty() {
                break;
            }
            output.push(left.trim_end().to_string());
            current = format!("{}{}", cont_indent, right_trimmed);
            if rec_depth < 4 && !has_top_level_logical_op(strip_trailing_comment(&current)) {
                if display_width(strip_trailing_comment(&current)) > target_len {
                    if let Some(expanded) = try_expand_paren_call(&current, target_len, rec_depth + 1) {
                        output.extend(expanded);
                        return output;
                    }
                }
            }
        } else {
            break;
        }
    }

    output.push(current);
    return output;
}

fn format_long_lines(file: &mut String, target_len: usize) {
    let mut wrapped = Vec::<String>::new();
    for line in file.lines() {
        wrapped.extend(wrap_single_long_line(line, target_len));
    }
    *file = wrapped.join("\n").trim().to_string() + "\n";
}

//TODO: create functions/algorithms to format specific kinds of long lines.
//this kind of almost works for certain cases
fn format_long_string_line(line: &String, target_len: usize) -> String {
    let mut result = line.clone();
    if let Some(idx) = line.find("\"") {
        let indents = count_leading_chars(&line, '\t');
        let mut s = "\n".to_string();
        for _ in 0..indents+1 { s.push('\t'); }
        result.insert_str(idx-1, s.as_str());
        let r_idx = result.rfind("\"").unwrap();
        if r_idx != idx && line.chars().nth(r_idx) == Some(')') {
            s = s[..s.len()-1].to_string();
            result.insert_str(r_idx+1, s.as_str());
        }
    }
    return result;
}
// fn format_long_array_line(line: String, target_len: usize) -> String {
    
// }

// fn format_long_function_definition_line(line: String, target_len: usize) -> String {
    
// }

// This function works unless the code jumps by more lines than the following block contains.
pub fn add_blank_lines_back(file: &mut String, target_lines: Vec<(usize, String)>) {
    let mut lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    // prinln!("{:#?}", &target_lines);
    for mut entry in target_lines.clone() {
        if lines.len() > entry.0+1 {
            let line = spaces_to_tabs(&entry.1);
            if lines[entry.0] == line {
                lines.insert(entry.0+1, "".to_string());
            }
            else {
                let mut idx = entry.0;
                'Inner: for i in (0..entry.0).rev() {
                    if &i < &lines.len() {
                        if &lines[i] == &line {
                            idx = i+1;
                            break 'Inner;
                        }
                    }
                }
                lines.insert(idx, "".to_string());
            }
        }
        else {
            let line = spaces_to_tabs(&entry.1);
            let mut idx = entry.0;
            'Inner: for i in (0..entry.0).rev() {
                if &i < &lines.len() {
                    if &lines[i] == &line {
                        idx = i+1;
                        break 'Inner;
                    }
                }
            }
            lines.insert(idx, "".to_string());
        }
    }
    *file = lines.join("\n");
}

fn join_single_line_constructs(file: &mut String, style: Style) {
    let mut allman = false;
    match style {
        Style::Allman => allman = true, 
        _ => (),
    };
    let lines: Vec<String> = file.split("\n").map(|x| x.to_string()).collect();
    let mut parts = Vec::<String>::new();
    let mut temp = String::new();
    let mut end = true;
    let mut indent = 0;
    for (i, line) in lines.iter().enumerate() {
        if allman && i < lines.len()-1 {
            let check = line.trim();
            let check2 = lines.iter().nth(i+1).unwrap().trim();
            if check.starts_with("switch") && (!check2.ends_with("{") && !check.ends_with(";")) {
                // must be added here to preserve indentation
                temp += format!("{} ", line.trim_end()).as_str();
                end = false;
                continue;
            }
        }
        else {
            let check = line.trim();
            if check.starts_with("switch") && (!check.ends_with("{") && !check.ends_with(";")) {
                temp += format!("{} ", line.trim_end()).as_str();
                end = false;
                continue;
            }
        }
        if !end && line.trim_end().ends_with(";") {
            end = true;
            temp += line.trim();
            parts.push(temp);
            temp = "".to_string();
            continue;
        }
        else if !end {
            temp += format!("{} ", line.trim()).as_str();
            continue;
        }
        parts.push(line.clone());
    }
    *file = parts.join("\n");
}

fn shift_back_preproc_lines(file: &mut String) {
    let lines: Vec<String> = file.split("\n").map(|x| x.to_string()).collect();
    let mut parts = Vec::<String>::new();
    for line in lines {
        let trimmed = line_trimmed_for_comment_check(&line);
        let code_only = strip_trailing_comment(&line);
        if trimmed.starts_with('#') {
            parts.push(line.trim_start().to_string());
        }
        else if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with('*') && code_only.trim_end().ends_with(':') {
            let temp = remove_single_tab(&line);
            parts.push(temp);
        }
        else {
            parts.push(line);
        }
    }
    *file = parts.join("\n");
}

pub fn close_empty_curly_brace_blocks(file: &mut String) {
    let has_trailing_nl = file.ends_with('\n');
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    if lines.len() < 2 {
        return;
    }
    let mut out = Vec::<String>::with_capacity(lines.len());
    let mut in_block_comment = false;
    let mut raw_end: Option<String> = None;
    let mut i = 0usize;

    while i < lines.len() {
        let l = &lines[i];
        let trimmed = line_trimmed_for_comment_check(l);

        if let Some(ref end_pat) = raw_end {
            if l.contains(end_pat.as_str()) {
                raw_end = None;
            }
            out.push(l.clone());
            i += 1;
            continue;
        }
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            out.push(l.clone());
            i += 1;
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            out.push(l.clone());
            i += 1;
            continue;
        }
        if has_unclosed_block_comment(l) {
            in_block_comment = true;
            out.push(l.clone());
            i += 1;
            continue;
        }
        if let Some(end_pat) = find_raw_string_end_delim(l) {
            raw_end = Some(end_pat);
            out.push(l.clone());
            i += 1;
            continue;
        }

        // Collapse empty function/method bodies: `void example()\n{\n}` -> `void example() {}`
        if i + 2 < lines.len()
            && lines[i + 1].trim() == "{"
            && !line_trimmed_for_comment_check(&lines[i + 2]).starts_with("//")
            && !line_trimmed_for_comment_check(&lines[i + 2]).starts_with("/*")
            && !has_unclosed_block_comment(&lines[i + 2])
            && strip_trailing_comment(&lines[i + 2]).trim() == "}"
            && is_empty_function_body_header(&lines, i)
        {
            let code_i = strip_trailing_comment(&lines[i]).trim_end();
            let trail_i = &lines[i][code_i.len()..];
            let code_close = strip_trailing_comment(&lines[i + 2]).trim_end();
            let trail_close = &lines[i + 2][code_close.len()..];
            if trail_i.is_empty() || trail_close.is_empty() {
                let trailing = if !trail_i.is_empty() { trail_i } else { trail_close };
                out.push(format!("{} {{}}{}", code_i, trailing));
                i += 3;
                continue;
            }
        }

        // Collapse same-line `{` + next-line `}` (empty control flow and empty lambdas)
        if i + 1 < lines.len()
            && lines[i].trim_end().ends_with('{')
            && strip_trailing_comment(&lines[i]).trim_end().ends_with('{')
            && lines[i].trim() != "{"
        {
            let next_trim = lines[i + 1].trim();
            let is_plain_close = next_trim == "}";
            let is_lambda_close = next_trim.starts_with('}')
                && !next_trim[1..].is_empty()
                && next_trim[1..].chars().all(|c| matches!(c, ';' | ',' | ')'))
                && lines[i].contains(']');
            if is_plain_close || is_lambda_close {
                out.push(format!("{}{}", lines[i].trim_end(), next_trim));
                i += 2;
                continue;
            }
        }

        out.push(l.clone());
        i += 1;
    }

    *file = out.join("\n");
    if has_trailing_nl {
        file.push('\n');
    }
}

pub fn sort_include_groups(file: String) -> String {
    let mut lines: Vec<String> = file.lines().map(|line| line.to_string()).collect();
    let mut result = Vec::new();
    let mut temp_group = Vec::new();
    for line in lines.into_iter() {
        if line.trim_start().starts_with("#include") {
            temp_group.push(line);
        }
        else {
            if !temp_group.is_empty() {
                for _ in 0..temp_group.len() {
                    sort_includes(&mut temp_group);
                }
                // Add sorted group to the result
                result.extend(temp_group.drain(..));
            }
            result.push(line);
        }
    }
    if !temp_group.is_empty() {
        sort_includes(&mut temp_group);
        result.extend(temp_group.drain(..));
    }
    return result.join("\n");
}

fn remove_comment(line: String) -> String {
    let idx = line.find("//").unwrap_or(line.len());
    let mut result: String;
    if idx == line.len() { result = line; }
    else { result = line[..idx].to_string(); }
    return result;
}

fn sort_includes(includes: &mut Vec<String>) {
    includes.sort_by(|a, b| {
        let mut a2 = a.clone();
        let mut b2 = b.clone();
        a2 = remove_comment(a2);
        b2 = remove_comment(b2);
        let a_has_angle = a2.contains('<') && a2.contains('>');
        let b_has_angle = b2.contains('<') && b2.contains('>');
        match (a_has_angle, b_has_angle) {
            (true, false) => Ordering::Less,    // Angle brackets come first
            (false, true) => Ordering::Greater, // Double quotes come after
            _ => a.cmp(b), // If both are the same type, sort normally
        }
    });
}

pub fn add_all_leading_tabs(text: String) -> String {
    let lines: Vec<&str> = text.split("\n").collect();
    let mut temp_vec = Vec::<String>::new();
    for line in lines {
        let temp_str = format!("\t{}", line);
        temp_vec.push(temp_str);
    }
    return temp_vec.join("\n");
}

fn is_non_type_keyword(word: &str) -> bool {
    matches!(
        word,
        "return" | "case" | "sizeof" | "else" | "throw" | "delete" | "new" | "co_return" | "co_yield" | "co_await" | "goto" | "if" | "while" | "for" | "switch" | "do"
    )
}

fn last_word(s: &str) -> &str {
    let trimmed = s.trim_end();
    let bytes = trimmed.as_bytes();
    let mut idx = bytes.len();
    while idx > 0 && (bytes[idx - 1].is_ascii_alphanumeric() || bytes[idx - 1] == b'_') {
        idx -= 1;
    }
    return &trimmed[idx..];
}

pub fn fix_stars(file: String) -> String {
    let has_trailing_newline = file.ends_with('\n');
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    let mut parts = Vec::<String>::new();
    let mut in_block_comment = false;

    for line in lines {
        let trimmed = line_trimmed_for_comment_check(&line);
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            parts.push(line);
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            parts.push(line);
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with('#') || line.contains("R\"") {
            parts.push(line);
            continue;
        }

        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut temp_line = String::new();
        let mut in_string = false;
        let mut in_char = false;
        let mut escaped = false;
        let mut i = 0;

        while i < len {
            let c = chars[i];
            if escaped {
                escaped = false;
                temp_line.push(c);
                i += 1;
                continue;
            }
            if c == '\\' && (in_string || in_char) {
                escaped = true;
                temp_line.push(c);
                i += 1;
                continue;
            }
            if c == '"' && !in_char {
                in_string = !in_string;
                temp_line.push(c);
                i += 1;
                continue;
            }
            if c == '\'' && !in_string {
                in_char = !in_char;
                temp_line.push(c);
                i += 1;
                continue;
            }
            if !in_string && !in_char && i + 1 < len && c == '/' {
                if chars[i + 1] == '/' {
                    for &rem in &chars[i..] {
                        temp_line.push(rem);
                    }
                    break;
                }
                if chars[i + 1] == '*' {
                    temp_line.push('/');
                    temp_line.push('*');
                    i += 2;
                    let mut closed = false;
                    while i < len {
                        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                            temp_line.push('*');
                            temp_line.push('/');
                            i += 2;
                            closed = true;
                            break;
                        }
                        temp_line.push(chars[i]);
                        i += 1;
                    }
                    if !closed {
                        in_block_comment = true;
                        break;
                    }
                    continue;
                }
            }
            if !in_string && !in_char && len > 1 && i <= len.saturating_sub(2) && i >= 1 {
                // Remove space after dereference `*` at start of expression / inside parens: `(* p)` -> `(*p)`
                if c == ' '
                    && chars[i - 1] == '*'
                    && (chars[i + 1].is_alphanumeric() || chars[i + 1] == '_' || chars[i + 1] == '(')
                    && i >= 2
                    && matches!(chars[i - 2], '(' | '\t' | '=' | ',' | ';' | '!' | '[' | '?' | ':')
                {
                    i += 1;
                    continue;
                }
                // Keep binary multiplication `a * b`
                if c == '*' && chars[i - 1] == ' ' && chars[i + 1] == ' ' {
                    temp_line.push(c);
                    i += 1;
                    continue;
                }
                // Collapse ` **` after a type name into `**` (e.g., `Task ** tasks` -> `Task** tasks`)
                if c == '*' && chars[i - 1] == ' ' && chars[i + 1] == '*' {
                    let prev_non_space = temp_line.trim_end().chars().last();
                    let prev_w = last_word(&temp_line);
                    if let Some(p) = prev_non_space {
                        if (p.is_alphanumeric() || p == '_' || p == '>') && !is_non_type_keyword(prev_w) {
                            while temp_line.ends_with(' ') {
                                temp_line.pop();
                            }
                            temp_line.push('*');
                            i += 1;
                            continue;
                        }
                    }
                }
                // `Type*name` -> `Type* name` (unless `operator*`)
                if c == '*'
                    && (chars[i + 1].is_alphanumeric() || chars[i + 1] == '_')
                    && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '*' || chars[i - 1] == '>')
                    && last_word(&temp_line) != "operator"
                {
                    temp_line.push('*');
                    temp_line.push(' ');
                    i += 1;
                    continue;
                }
                // `Type *name` -> `Type* name`, only when preceded by a type name (not `return *p` or `= *p`!)
                if c == '*'
                    && chars[i - 1] == ' '
                    && (chars[i + 1].is_alphanumeric() || chars[i + 1] == '_')
                {
                    let prev_non_space = temp_line.trim_end().chars().last();
                    let prev_w = last_word(&temp_line);
                    if let Some(p) = prev_non_space {
                        let is_template_close = p == '>' && temp_line.trim_end().len() >= 2 && !temp_line.trim_end()[..temp_line.trim_end().len() - 1].ends_with(' ');
                        if (p.is_alphanumeric() || p == '_' || p == '*' || is_template_close) && !is_non_type_keyword(prev_w) {
                            temp_line.pop();
                            temp_line.push('*');
                            temp_line.push(' ');
                            i += 1;
                            continue;
                        }
                    }
                }
                // `(int *)` -> `(int*)`
                if c == '*' && chars[i - 1] == ' ' && chars[i + 1] == ')' {
                    let prev_non_space = temp_line.trim_end().chars().last();
                    if let Some(p) = prev_non_space {
                        if p.is_alphanumeric() || p == '_' || p == '*' || p == '>' {
                            temp_line.pop();
                            temp_line.push('*');
                            i += 1;
                            continue;
                        }
                    }
                }
            }
            temp_line.push(c);
            i += 1;
        }
        parts.push(temp_line);
    }
    let mut joined = parts.join("\n");
    if has_trailing_newline {
        joined.push('\n');
    }
    return joined;
}

pub fn remove_dereference_spaces(line: String) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();
    let mut skip = false;
    for i in 0..chars.len() {
        if skip {
            skip = false;
            continue;
        }
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == ' ' {
            skip = true;
        }
        result.push(chars[i]);
    }
    return result;
}

pub fn remove_pointer_spaces(line: String) -> String {
    if line.contains('\n') {
        return line
            .split('\n')
            .map(|l| {
                let t = line_trimmed_for_comment_check(l);
                if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
                    l.to_string()
                } else {
                    remove_pointer_spaces_single(l.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    remove_pointer_spaces_single(line)
}

fn remove_pointer_spaces_single(line: String) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            result.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char && i + 1 < chars.len() && c == '/' {
            if chars[i + 1] == '/' {
                for &rem in &chars[i..] {
                    result.push(rem);
                }
                break;
            }
            if chars[i + 1] == '*' {
                result.push('/');
                result.push('*');
                i += 2;
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                        result.push('*');
                        result.push('/');
                        i += 2;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        if !in_string && !in_char && i + 1 < chars.len() && i > 0 {
            // Do not turn binary multiplication `a * b` into `a* b`!
            // Only attach `*` to left if followed by `*`, `)`, `,`, `>`, `[`, or an identifier when before `=` / inside params.
            let next_after_star = if i + 2 < chars.len() { Some(chars[i + 2]) } else { None };
            let is_binary_mult = chars[i + 1] == '*' && next_after_star == Some(' ') && {
                // Check if there is an `=` before `i`, meaning we are on the RHS initializer expression!
                result.contains('=')
            };
            let prev_w = last_word(&result);
            let skip = c == ' '
                && chars[i + 1] == '*'
                && !is_binary_mult
                && !is_non_type_keyword(prev_w)
                && (chars[i - 1].is_alphanumeric()
                    || chars[i - 1] == '_'
                    || chars[i - 1] == '*'
                    || chars[i - 1] == '>');
            if skip {
                i += 1;
                continue;
            }
        }
        result.push(c);
        i += 1;
    }
    return result;
}

pub fn switch_pointer_spaces(line: String) -> String {
    return remove_pointer_spaces(line);
}

pub fn remove_reference_spaces(line: String) -> String {
    if line.contains('\n') {
        return line
            .split('\n')
            .map(|l| {
                let t = line_trimmed_for_comment_check(l);
                if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
                    l.to_string()
                } else {
                    remove_reference_spaces_single(l.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    remove_reference_spaces_single(line)
}

fn remove_reference_spaces_single(line: String) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            result.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char && i + 1 < chars.len() && c == '/' {
            if chars[i + 1] == '/' {
                for &rem in &chars[i..] {
                    result.push(rem);
                }
                break;
            }
            if chars[i + 1] == '*' {
                result.push('/');
                result.push('*');
                i += 2;
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                        result.push('*');
                        result.push('/');
                        i += 2;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        if !in_string && !in_char && i + 1 < chars.len() && i > 0 {
            let after_eq = result.contains('=');
            let is_single_ref = chars[i + 1] == '&' && (i + 2 >= chars.len() || chars[i + 2] != '&');
            let is_rvalue_ref = !after_eq && chars[i + 1] == '&' && i + 2 < chars.len() && chars[i + 2] == '&';
            let prev_w = last_word(&result);
            let skip = c == ' '
                && (is_single_ref || is_rvalue_ref)
                && !is_non_type_keyword(prev_w)
                && (chars[i - 1].is_alphanumeric()
                    || chars[i - 1] == '_'
                    || chars[i - 1] == '>'
                    || chars[i - 1] == '*');
            if skip {
                i += 1;
                continue;
            }
        }
        result.push(c);
        i += 1;
    }
    return result;
}

fn remove_single_tab(line: &String) -> String {
    let mut line = line.clone();
    if line.starts_with("\t") { line.remove(0); }
    return line;
}

pub fn remove_all_spaces(line: String) -> String {
    let chars: Vec<char> = line.chars().filter(|c| *c != ' ').collect();
    let mut result = String::new();
    for c in chars {
        result.push(c);
    }
    return result;
}

pub fn remove_object_constructor_space(line: String) -> String {
    let chars: Vec<char> = line.chars().collect();
    if let Some(pos) = chars.iter().position(|&c| c == '(') {
        if pos > 0 && chars[pos - 1] == ' ' {
            let mut out = String::with_capacity(line.len());
            for (idx, &ch) in chars.iter().enumerate() {
                if idx != pos - 1 {
                    out.push(ch);
                }
            }
            return out;
        }
    }
    return line;
}

pub fn remove_unnecessary_spaces(line: &String) -> String {
    if line.contains('\n') {
        let mut out = Vec::<String>::new();
        for (idx, sub) in line.split('\n').enumerate() {
            let check = line_trimmed_for_comment_check(sub);
            if check.starts_with("//") || check.starts_with("/*") || check.starts_with('*') {
                if let Some(pos) = sub.find(COMMENT_OFFSET_SENTINEL).or_else(|| sub.find(COMMENT_COL0_SENTINEL)) {
                    out.push(sub[pos..].to_string());
                } else {
                    let tabs = sub.chars().take_while(|&c| c == '\t').count();
                    out.push(format!("{}{}", "\t".repeat(tabs), sub.trim_start()));
                }
            } else if idx > 0 {
                let tabs = sub.chars().filter(|&c| c == '\t').count().max(1);
                let cleaned = remove_unnecessary_spaces_single(sub.trim_start());
                if cleaned.is_empty() {
                    continue;
                }
                out.push(format!("{}{}", "\t".repeat(tabs), cleaned));
            } else {
                out.push(remove_unnecessary_spaces_single(sub));
            }
        }
        return out.join("\n");
    }
    remove_unnecessary_spaces_single(line)
}

fn remove_unnecessary_spaces_single(line: &str) -> String {
    let leading_tokens = ['(', '[', ' ', '!'];
    let ending_tokens = [')', ']', ' ', ',', ';'];
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::new();
    let mut skip = false;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if skip {
            skip = false;
            i += 1;
            continue;
        }
        if escaped {
            escaped = false;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            result.push(c);
            i += 1;
            continue;
        }
        if !in_string && !in_char && i + 1 < chars.len() && c == '/' {
            if chars[i + 1] == '/' {
                for &rem in &chars[i..] {
                    result.push(rem);
                }
                break;
            }
            if chars[i + 1] == '*' {
                result.push('/');
                result.push('*');
                i += 2;
                while i < chars.len() {
                    if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                        result.push('*');
                        result.push('/');
                        i += 2;
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        if !in_string && !in_char && i + 1 < chars.len() {
            let next = chars[i + 1];
            // Do not collapse intentional spacing immediately before a trailing comment (`foo();  // comment`)
            let next_starts_comment = next == ' ' && {
                let mut k = i + 1;
                while k < chars.len() && chars[k] == ' ' {
                    k += 1;
                }
                k + 1 < chars.len() && chars[k] == '/' && (chars[k + 1] == '/' || chars[k + 1] == '*')
            };
            if ending_tokens.contains(&next) && c == ' ' && !next_starts_comment {
                i += 1;
                continue;
            }
            // Remove space before `[` only when attached to an identifier/type/subscript (`arr [0]` -> `arr[0]`),
            // NOT before a C++ lambda capture (`= [x]`, `, [x]`, `return [x]`)!
            if next == '[' && c == ' ' {
                let prev_non_space = result.trim_end().chars().last();
                let prev_w = last_word(&result);
                if let Some(p) = prev_non_space {
                    if (p.is_alphanumeric() || p == '_' || p == ']' || p == ')') && prev_w != "return" && prev_w != "delete" {
                        i += 1;
                        continue;
                    }
                }
            }
            if leading_tokens.contains(&c) && next == ' ' && !next_starts_comment {
                skip = true;
            }
        }
        result.push(c);
        i += 1;
    }
    return result.trim().to_string();
}

pub fn remove_double_spaces(line: &String) -> String {
    let mut result = "".to_string();
    for (i, c) in line.chars().enumerate() {
        if i < line.len()-1 {
            if c == ' ' && line.at(i+1) == Some(' ') { continue; }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn remove_whitespace_before_commas(line: &String) -> String {
    let mut result = "".to_string();
    for (i, c) in line.chars().enumerate() {
        if i < line.len()-1 {
            if c == ' ' && line.at(i+1) == Some(',') { continue; }
        }
        result += c.to_string().as_str();
    }
    return result;
}

pub fn add_leading_whitespace(src: String, amount: usize) -> String {
    let mut prefix: String = "".to_string();
    for _ in 0..amount { prefix += " " };
    return format!("{}{}", prefix, src);
}

pub fn count_leading_chars(src: &String, target: char) -> usize {
    let mut count = 0;
    for c in src.chars() {
        if c != target {
            break;
        }
        count += 1;
    }
    return count;
}

pub fn sanitize(input: String) -> String {
    let allowed_chars = Regex::new(r"[a-zA-Z0-9.\-_\+\=/\s]+").unwrap();
    let captures = allowed_chars.captures(&input).unwrap();
    let result = captures.get(0).unwrap().as_str().to_string();
    return result;
}

fn line_has_no_alphanumeric(line: &str) -> bool {
    for c in line.chars() {
        if c.is_alphanumeric() {
            return false;
        }
    }
    return true;
}

pub fn remove_blank_lines(lines: Vec<&str>) -> String {
    let result: Vec<&str> = lines.into_iter().filter(|line| !line.trim().is_empty()).collect();
    return result.join("\n");
}

pub fn write_file(path: &String, contents: &[u8]) -> Result<(), Error> {
    let mut dest = File::create(path).unwrap();
    let ok = dest.write_all(contents);
    return ok;
}

pub fn extract_txt_opts(args: Vec<&str>) -> options::TxtOpts {
    let start = args.iter().position(|x| x == &"-o" || x == &"--opts").unwrap() + 1;
    let remainder = args[start..].into_iter().map(|x| x.to_owned()).collect::<Vec<&str>>();
    let pattern = Regex::new(r"\s-\w").unwrap();
    let remainder_string = &remainder.join(" ");
    let matches = &pattern.find_iter(&remainder_string).map(|x| x.as_str()).collect::<Vec<&str>>();
    let end: usize;
    if matches.len() > 0 {
        let flag = remainder.iter().filter(|x| x.contains("-")).collect::<Vec<&&str>>()[0].to_owned();
        let start = &start.clone();
        end = args[*start..].iter().position(|x| x == &flag).unwrap() + start;
    }
    else {
        end = args.len();
    }
    let opts = args[start..end].into_iter().map(|s| s.parse::<usize>().expect("Error parsing number.")).collect::<Vec<usize>>();
    let result: TxtOpts;
    let cols = opts[0];
    let spacing = opts[1];
    result = TxtOpts{
        columns: cols,
        spacing: spacing
    };
    return result;
}

pub fn intersperse<T: Clone>(vec: Vec<T>, value: T) -> Vec<T> {
    if vec.is_empty() {
        return vec;
    }
    let mut result = Vec::<T>::new();
    for item in vec {
        result.push(item);
        result.push(value.clone());
    }
    result = result[..result.len()-1].into_iter().map(|x| x.to_owned()).collect::<Vec<T>>();
    return result;
}

pub fn median(lengths: &Vec<usize>) -> usize {
    let mut sorted_list = lengths.clone();
    sorted_list.sort();
    let idx = sorted_list.len() / 2;
    return sorted_list[idx];
}

pub fn check_for_even_line_length(lines: &Vec<&str>) -> bool {
    let line_lens = lines.iter().map(|x| x.len()).collect::<Vec<usize>>();
    let med = median(&line_lens);
    let check_val = (med as f32 * 0.8).ceil();
    let mut count = 0;
    for i in 0..line_lens.len() {
        if (line_lens[i] as f32) < check_val && i != 0 {
            count += 1;
        }
    }
    return count == 0;
}

pub fn ends_with_brace(line: &String) -> bool {
    return line.trim_end().ends_with("{");
}

fn check_line_is_blank(line: &String) -> bool {
    return line.is_empty() || line.chars().all(|x| x.is_whitespace() || x == '\t');
}

pub fn infer_file_type(filepath: &String) -> String {
    let supported_types = get_file_extensions_list();
    for item in supported_types {
        if filepath.contains(&item) {
            return item.to_string();
        }
    }
    return "unknown".to_string();
}

pub fn get_file_extensions_list() -> Vec<String> {
    // .cpp has to stay before .c, otherwise it breaks
    let exts = {
        vec![".cpp", ".cc", ".C", ".c", ".hpp", ".hh", ".H", ".h", ".go", ".py", ".txt"]
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
    };
    return exts;
}

pub fn display_file_extensions() {
    let lines = get_file_extensions_list();
    for line in lines {
        println!("{}", line);
    }
}

pub fn check_valid_file_ext(path: &String) -> bool {
    let exts = get_file_extensions_list();
    for ext in exts {
        if path.contains(&ext) { return true; }
    }
    return false;
}

pub fn print_usage() {
println!(
"
USAGE:\nblfmt <file-path> <flags + opts>

FLAGS:
    -h or --help
(Code only):
    -  or --stdin <file-ext>
(C/C++ only):
    -s or --style allman|knr|stroustrup(default)
(Txt only):
    -o or --options <columns> <spacing> (numbers)
    -t or --titles \"Places\" \"where\" \"you\" \"want\" \"paragraphs\" ...
    (The txt flags may be combined)
"
);
}

pub fn write_results(path: &String, results: String) {
    let ok = write_file(path, results.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: '{}'", path),
        Err(e) => println!("Error during `format_txt_file()`: {}", e),
    };
}

pub fn print_results(results: &String) {
    println!("{}", results);
}

