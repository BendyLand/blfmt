use std::{borrow::Cow, fs, fs::File, io::Write, io::Error};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub fn trailing_comment_gap<'a>(start_byte: usize, src: &'a str) -> Option<&'a str> {
    let before = line_prefix_before(start_byte, src);
    if before.trim().is_empty() {
        return None;
    }
    let trimmed = before.trim_end_matches(|c: char| c == ' ' || c == '\t');
    Some(&before[trimmed.len()..])
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
    raw: &str,
    comment_start_byte: usize,
    ref_start_byte: Option<usize>,
    src: &str,
    is_top_level: bool,
) -> String {
    if trailing_comment_gap(comment_start_byte, src).is_some() {
        return raw.to_string();
    }
    let comment_ws = line_prefix_before(comment_start_byte, src);
    let comment_col = visual_col_width(comment_ws);

    if is_top_level {
        if comment_col > 0 {
            return format!("{}{}{}", COMMENT_OFFSET_SENTINEL, comment_ws, raw);
        }
        return raw.to_string();
    }

    let Some(ref_byte) = ref_start_byte else {
        if comment_col == 0 {
            return raw
                .split('\n')
                .map(|l| format!("{}{}", COMMENT_COL0_SENTINEL, l))
                .collect::<Vec<_>>()
                .join("\n");
        }
        return raw.to_string();
    };

    let ref_ws = line_prefix_before(ref_byte, src);
    // Only compare if ref_ws really is pure leading indentation
    if !ref_ws.trim().is_empty() {
        return raw.to_string();
    }
    let ref_col = visual_col_width(ref_ws);

    if comment_col == ref_col {
        return raw.to_string();
    }

    if comment_col == 0 && ref_col > 0 {
        return raw
            .split('\n')
            .map(|l| format!("{}{}", COMMENT_COL0_SENTINEL, l))
            .collect::<Vec<_>>()
            .join("\n");
    }

    if comment_col > ref_col {
        let extra_ws: Cow<'_, str> = if comment_ws.starts_with(ref_ws) {
            Cow::Borrowed(&comment_ws[ref_ws.len()..])
        } else {
            Cow::Owned(" ".repeat(comment_col - ref_col))
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

    raw.to_string()
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
            attach_trailing_comment(result, gap, comment, ensure_nl_for_line_comment);
            return true;
        }
    }
    result.push_str(standalone_prefix);
    result.push_str(comment);
    result.push_str(standalone_suffix);
    false
}

pub fn push_comment_to_parts<'a>(
    parts: &mut Vec<Cow<'a, str>>,
    comment: &str,
    start_byte: usize,
    src: &str,
    indent_with_tab: bool,
) -> bool {
    if let Some(last) = parts.last_mut() {
        if !last.trim().is_empty() {
            if let Some(gap) = trailing_comment_gap(start_byte, src) {
                attach_trailing_comment(last.to_mut(), gap, comment, false);
                return true;
            }
        }
    }
    if indent_with_tab {
        parts.push(Cow::Owned(add_all_leading_tabs(comment)));
    } else {
        parts.push(Cow::Owned(comment.to_string()));
    }
    false
}

pub fn push_expr_comment<'a>(
    parts: &mut Vec<Cow<'a, str>>,
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
            let s = last.to_mut();
            while s.ends_with(' ') || s.ends_with('\t') {
                s.pop();
            }
            s.push_str(gap);
            s.push_str(comment);
            if is_line_comment || !has_after {
                s.push_str("\n\t");
            }
            return;
        }
    }

    // Standalone comment inside an expression/header
    if let Some(last) = parts.last_mut() {
        if last.ends_with("\n\t") || last.ends_with('\n') {
            let s = last.to_mut();
            while s.ends_with('\t') || s.ends_with(' ') {
                s.pop();
            }
            s.push_str(comment);
            if is_line_comment || !has_after {
                s.push_str("\n\t");
            }
            return;
        }
    }
    if is_line_comment || !has_after {
        parts.push(Cow::Owned(format!("\n{}\n\t", comment)));
    } else {
        parts.push(Cow::Owned(format!("\n{}", comment)));
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
            result.push_str(gap);
            result.push_str(comment);
            if is_line_comment || !has_after {
                result.push_str("\n\t");
            } else if had_trailing_space || matches!(src.as_bytes().get(end_byte), Some(b' ' | b'\t')) {
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
    let mut in_macro_cont = false;
    for line in file.lines() {
        let trimmed = line.trim_start();
        if in_macro_cont {
            split_lines.push(line.to_string());
            in_macro_cont = line.trim_end().ends_with('\\');
            continue;
        }
        if trimmed.starts_with('#') {
            split_lines.push(line.to_string());
            in_macro_cont = line.trim_end().ends_with('\\');
            continue;
        }
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
    in_macro_cont = false;
    while i < lines.len() {
        let line = lines[i].trim_start();
        if in_macro_cont {
            in_macro_cont = lines[i].trim_end().ends_with('\\');
            i += 1;
            continue;
        }
        if line.starts_with('#') {
            in_macro_cont = lines[i].trim_end().ends_with('\\');
            i += 1;
            continue;
        }
        if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
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
    let mut in_macro_cont = false;
    for line in file.lines() {
        let trimmed = line.trim_start();
        if in_macro_cont {
            result_lines.push(line.to_string());
            in_macro_cont = line.trim_end().ends_with('\\');
            continue;
        }
        if trimmed.starts_with('#') {
            result_lines.push(line.to_string());
            in_macro_cont = line.trim_end().ends_with('\\');
            continue;
        }
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
    let ensured: Vec<Cow<'_, str>> = file
        .lines()
        .map(|line| {
            if line.contains(',') {
                Cow::Owned(ensure_space_after_char(line, ','))
            } else {
                Cow::Borrowed(line)
            }
        })
        .collect();
    *file = ensured.join("\n");
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
    align_multiline_define_backslashes(file, 100);
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

    // Separate #include groups from #define groups, and ensure blank lines around multiline #define blocks.
    for i in 0..n {
        if !in_comment[i] && !in_raw_str[i] {
            let t = lines[i].trim_start();
            if t.starts_with("#define") && i > 0 && lines[i - 1].trim_start().starts_with("#include") {
                insert_before[i] = true;
            } else if t.starts_with("#include")
                && i > 0
                && (lines[i - 1].trim_start().starts_with("#define") || in_macro[i - 1])
            {
                insert_before[i] = true;
            }
        }
    }

    let mut m_idx = 0usize;
    while m_idx < n {
        let t = lines[m_idx].trim_start();
        if t.starts_with("#define") && lines[m_idx].trim_end().ends_with('\\') {
            let start_m = m_idx;
            let mut end_m = m_idx;
            let mut in_mc_comment = has_unclosed_block_comment(&lines[m_idx]);
            while end_m + 1 < n && (lines[end_m].trim_end().ends_with('\\') || in_mc_comment) {
                end_m += 1;
                if in_mc_comment {
                    if lines[end_m].contains("*/") {
                        in_mc_comment = false;
                    }
                } else if has_unclosed_block_comment(&lines[end_m]) {
                    in_mc_comment = true;
                }
            }
            if end_m > start_m {
                let mut top = start_m;
                while top > 0 {
                    let pt = lines[top - 1].trim();
                    if pt.starts_with("//") || pt.starts_with("/*") || pt.starts_with('*') || pt.ends_with("*/") {
                        top -= 1;
                    } else {
                        break;
                    }
                }
                if top > 0 {
                    let prev = lines[top - 1].trim();
                    if !prev.is_empty()
                        && prev != "{"
                        && !prev.starts_with("#ifdef")
                        && !prev.starts_with("#ifndef")
                        && !prev.starts_with("#if ")
                        && !prev.starts_with("#if(")
                    {
                        insert_before[top] = true;
                    }
                }
                if end_m + 1 < n {
                    let next = lines[end_m + 1].trim();
                    if !next.is_empty()
                        && next != "}"
                        && next != "};"
                        && !next.starts_with("#endif")
                        && !next.starts_with("#else")
                        && !next.starts_with("#elif")
                    {
                        insert_before[end_m + 1] = true;
                    }
                }
            }
            m_idx = end_m + 1;
        } else {
            m_idx += 1;
        }
    }

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

pub fn detect_indentation(line: &str) -> usize {
    line.bytes().take_while(|&b| b == b'\t').count()
}

fn fix_indentation<'a>(line: &'a str, current: usize, target: usize) -> Cow<'a, str> {
    if current == target {
        return Cow::Borrowed(line);
    }
    if current < target {
        let mut out = String::with_capacity((target - current) + line.len());
        for _ in current..target {
            out.push('\t');
        }
        out.push_str(line);
        Cow::Owned(out)
    } else {
        let mut rest = line;
        for _ in target..current {
            rest = remove_single_tab(rest);
        }
        Cow::Borrowed(rest)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IndentBraceKind {
    Normal,
    NoIndent,
    CaseBlock,
}

fn fix_indentation_levels(file: &mut String) {
    let updated = {
        let lines: Vec<&str> = file.lines().collect();
        let mut result: Vec<Cow<'_, str>> = Vec::with_capacity(lines.len());
        let mut indent_level: usize = 0;
        let mut brace_stack = Vec::<IndentBraceKind>::new();
        let mut pending_no_indent_brace = false;
        let mut in_block_comment = false;
        let mut in_macro_cont = false;
        let mut in_macro_block_comment = false;

        for line in lines {
            if in_macro_cont {
                result.push(Cow::Borrowed(line));
                if in_macro_block_comment {
                    if line.contains("*/") {
                        in_macro_block_comment = false;
                    }
                } else if has_unclosed_block_comment(line) {
                    in_macro_block_comment = true;
                }
                in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
                continue;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                result.push(Cow::Borrowed(""));
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
                result.push(Cow::Borrowed(payload));
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
                if indent_level == 0 {
                    result.push(Cow::Borrowed(payload));
                } else {
                    result.push(Cow::Owned(format!("{}{}", "\t".repeat(indent_level), payload)));
                }
                continue;
            }

            if in_block_comment {
                if indent_level == 0 {
                    result.push(Cow::Borrowed(line));
                } else {
                    let current_level = detect_indentation(line);
                    result.push(fix_indentation(line, current_level, indent_level));
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
                    result.push(Cow::Borrowed(line));
                } else {
                    let current_level = detect_indentation(line);
                    result.push(fix_indentation(line, current_level, indent_level));
                }
                continue;
            }

            if trimmed.starts_with("//") {
                let current_level = detect_indentation(line);
                result.push(fix_indentation(line, current_level, indent_level));
                continue;
            }

            if trimmed.starts_with('#') {
                result.push(Cow::Borrowed(trimmed));
                in_macro_block_comment = has_unclosed_block_comment(trimmed);
                in_macro_cont = trimmed.ends_with('\\') || in_macro_block_comment;
                continue;
            }

            let code_part = strip_trailing_comment(trimmed);

            if (code_part.starts_with("namespace ") || code_part == "namespace" || code_part.starts_with("inline namespace "))
                && !code_part.contains(';')
                && !code_part.contains('=')
            {
                if code_part.ends_with('{') {
                    let current_level = detect_indentation(line);
                    result.push(fix_indentation(line, current_level, indent_level));
                    brace_stack.push(IndentBraceKind::NoIndent);
                    if has_unclosed_block_comment(line) {
                        in_block_comment = true;
                    }
                    continue;
                } else {
                    pending_no_indent_brace = true;
                }
            } else if code_part.starts_with("extern \"") && code_part.ends_with('{') {
                let current_level = detect_indentation(line);
                result.push(fix_indentation(line, current_level, indent_level));
                brace_stack.push(IndentBraceKind::NoIndent);
                if has_unclosed_block_comment(line) {
                    in_block_comment = true;
                }
                continue;
            } else if code_part.starts_with("extern \"") && !code_part.contains(';') && !code_part.ends_with("{}") {
                pending_no_indent_brace = true;
            }

            let mut closed_case_block = false;
            if code_part == "}" || code_part.starts_with('}') {
                match brace_stack.pop().unwrap_or(IndentBraceKind::Normal) {
                    IndentBraceKind::Normal => {
                        if indent_level > 0 {
                            indent_level -= 1;
                        }
                    }
                    IndentBraceKind::NoIndent => {}
                    IndentBraceKind::CaseBlock => {
                        closed_case_block = true;
                    }
                }
            }

            let is_case_block_open = (code_part.starts_with("case ")
                || code_part.starts_with("case(")
                || code_part.starts_with("default:")
                || code_part.starts_with("default :"))
                && code_part.contains(':')
                && code_part.ends_with('{')
                && !code_part.ends_with("{}");

            let target_indent = if code_part.starts_with(": ") {
                indent_level + 1
            } else if closed_case_block || is_case_block_open {
                indent_level.saturating_sub(1)
            } else {
                indent_level
            };
            let current_level = detect_indentation(line);
            result.push(fix_indentation(line, current_level, target_indent));

            if code_part == "{" || (code_part.ends_with('{') && !code_part.ends_with("{}")) {
                if pending_no_indent_brace {
                    brace_stack.push(IndentBraceKind::NoIndent);
                    pending_no_indent_brace = false;
                } else if is_case_block_open {
                    brace_stack.push(IndentBraceKind::CaseBlock);
                } else {
                    brace_stack.push(IndentBraceKind::Normal);
                    indent_level += 1;
                }
            } else if code_part != "{" && !code_part.starts_with("namespace") && !code_part.starts_with("inline namespace") && !code_part.starts_with("extern \"") {
                pending_no_indent_brace = false;
            }

            if has_unclosed_block_comment(line) {
                in_block_comment = true;
            }
        }
        let mut out = result.join("\n");
        out.push('\n');
        out
    };
    *file = updated;
}

fn line_is_function_def(line: &str, pattern: &regex::Regex) -> bool {
    pattern.is_match(line)
}

fn line_is_doc_comment(line: &str) -> bool {
    line.trim_start().starts_with('*') && line.contains("*/")
}

fn find_next_function_def(remaining_lines: &[&str], pattern: &regex::Regex) -> i32 {
    for (i, line) in remaining_lines.iter().enumerate() {
        if line_is_function_def(line, pattern) {
            return i as i32;
        }
    }
    -1
}

fn find_next_doc_comment(remaining_lines: &[&str]) -> i32 {
    for (i, line) in remaining_lines.iter().enumerate() {
        if line.trim_start().starts_with('*') && line.contains("*/") {
            return i as i32;
        }
    }
    -1
}

fn ensure_proper_doc_comment_spacing(file: &mut String) {
    let updated = {
        let lines: Vec<&str> = file.trim().lines().collect();
        let mut result: Vec<&str> = Vec::with_capacity(lines.len());
        let mut pattern: Option<Regex> = None;
        for (i, &line) in lines.iter().enumerate() {
            if i > 0 && i + 1 < lines.len() && line.trim().is_empty() && line_is_doc_comment(lines[i - 1]) {
                let pat = pattern.get_or_insert_with(|| {
                    Regex::new("^(\\w*\\s*)*\\w+\\s+\\w+\\(.*\\)\\s*;?\\s*$").unwrap()
                });
                if line_is_function_def(lines[i + 1], pat) {
                    continue;
                }
            }
            result.push(line);
        }
        let joined = result.join("\n");
        let mut out = joined.trim_end().to_string();
        out.push('\n');
        out
    };
    *file = updated;
}

fn ends_with_overloaded_operator_token(s: &str) -> bool {
    let Some(op_pos) = s.rfind("operator") else {
        return false;
    };
    if op_pos > 0 {
        let prev = s[..op_pos].chars().last().unwrap_or(' ');
        if prev.is_ascii_alphanumeric() || prev == '_' {
            return false;
        }
    }
    let after = &s[op_pos + "operator".len()..];
    !after.is_empty()
        && after.chars().all(|ch| {
            matches!(
                ch,
                '=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '^' | '&' | '|' | ','
            )
        })
}

pub fn ensure_space_after_char(line: &str, target: char) -> String {
    let trimmed = line_trimmed_for_comment_check(line);
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with('#') {
        return line.to_string();
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
            if c == target
                && chars[i + 1] != ' '
                && chars[i + 1] != target
                && chars[i + 1] != '='
                && !ends_with_overloaded_operator_token(&result)
            {
                result.push(' ');
            }
        }
        i += 1;
    }
    return result;
}


fn remove_blank_lines_from_blocks(file: &mut String) {
    let updated = {
        let lines: Vec<&str> = file.trim().lines().collect();
        let mut result: Vec<&str> = Vec::with_capacity(lines.len());
        let mut in_block = false;
        let mut prev_non_empty: &str = "";
        for line in lines {
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
                prev_non_empty = trimmed;
            }
            if in_block && !check_line_is_blank(line) {
                result.push(line);
            } else if !in_block {
                result.push(line);
            }
        }
        let joined = result.join("\n");
        let mut out = joined.trim_end().to_string();
        out.push('\n');
        out
    };
    *file = updated;
}

fn ensure_no_consecutive_blank_lines(file: &mut String) {
    let updated = {
        let lines: Vec<&str> = file.trim().lines().collect();
        let mut result: Vec<&str> = Vec::with_capacity(lines.len());
        for (i, &line) in lines.iter().enumerate() {
            if i + 1 < lines.len() && check_line_is_blank(line) {
                if let Some(&prev) = result.last() {
                    if !check_line_is_blank(prev) {
                        result.push(line);
                    }
                }
                continue;
            }
            result.push(line);
        }
        let joined = result.join("\n");
        let mut out = joined.trim_end().to_string();
        out.push('\n');
        out
    };
    *file = updated;
}

//TODO: make a function that scans for the specific constructs that get aggressively formatted (e.g., maps)
pub fn scan_for_lines_before_blank_lines(file: &str) -> Vec<(usize, String)> {
    let mut result = Vec::<(usize, String)>::new();
    let lines: Vec<&str> = file.lines().collect();
    let mut open_braces = 0;
    for (i, &line) in lines.iter().enumerate() {
        // only add lines that are inside of blocks
        if line.contains('{') { open_braces += 1; }
        if line.contains('}') { open_braces -= 1; }
        if open_braces > 0 && i + 1 < lines.len() {
            let has_no_visible_chars = lines[i + 1]
                .chars()
                .all(|x| x.is_whitespace() || x == '\t');
            if lines[i + 1].is_empty() || has_no_visible_chars {
                result.push((i + 1, line.to_string()));
            }
        }
    }
    result
}

fn spaces_to_tabs(line: &str) -> String {
    line.replace("    ", "\t")
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

pub fn merge_incomplete_decl_into_parts<'a>(parts: &mut Vec<Cow<'a, str>>, continuation: &str) {
    if let Some(last) = parts.last_mut() {
        let s = last.to_mut();
        let mut nl_count = 0usize;
        while s.ends_with('\n') {
            s.pop();
            nl_count += 1;
        }
        while s.ends_with(' ') || s.ends_with('\t') {
            s.pop();
        }
        s.push(' ');
        s.push_str(continuation.trim_start());
        for _ in 0..nl_count {
            s.push('\n');
        }
    } else {
        parts.push(Cow::Owned(continuation.to_string()));
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

fn find_control_flow_cond_parens(chars: &[char]) -> Option<(usize, usize)> {
    let n = chars.len();
    let mut i = 0usize;
    while i < n && chars[i].is_whitespace() {
        i += 1;
    }
    if i < n && chars[i] == '}' {
        i += 1;
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
    }

    let read_word = |start: usize| -> (&str, usize) {
        let mut end = start;
        while end < n && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        let s: String = chars[start..end].iter().collect();
        // Match against static keywords so we don't need to return a borrowed String
        let kw = match s.as_str() {
            "if" => "if",
            "else" => "else",
            "while" => "while",
            "constexpr" => "constexpr",
            _ => "",
        };
        (kw, end)
    };

    let (w1, mut pos) = read_word(i);
    if w1 == "else" {
        while pos < n && chars[pos].is_whitespace() {
            pos += 1;
        }
        let (w2, pos2) = read_word(pos);
        if w2 != "if" {
            return None;
        }
        pos = pos2;
    } else if w1 != "if" && w1 != "while" {
        return None;
    }

    while pos < n && chars[pos].is_whitespace() {
        pos += 1;
    }
    if w1 == "if" || w1 == "else" {
        let (w_ce, pos_ce) = read_word(pos);
        if w_ce == "constexpr" {
            pos = pos_ce;
            while pos < n && chars[pos].is_whitespace() {
                pos += 1;
            }
        }
    }

    if pos >= n || chars[pos] != '(' {
        return None;
    }
    let open_idx = pos;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut k = open_idx;
    while k < n {
        let c = chars[k];
        if escaped {
            escaped = false;
            k += 1;
            continue;
        }
        if c == '\\' && (in_string || in_char) {
            escaped = true;
            k += 1;
            continue;
        }
        if c == '"' && !in_char {
            in_string = !in_string;
            k += 1;
            continue;
        }
        if c == '\'' && !in_string {
            in_char = !in_char;
            k += 1;
            continue;
        }
        if !in_string && !in_char {
            if k + 1 < n && c == '/' && chars[k + 1] == '*' {
                k += 2;
                while k + 1 < n {
                    if chars[k] == '*' && chars[k + 1] == '/' {
                        k += 2;
                        break;
                    }
                    k += 1;
                }
                continue;
            }
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((open_idx, k));
                }
            }
        }
        k += 1;
    }
    None
}

fn align_trailing_logical_ops(lines: &mut [String], target_len: usize) {
    let mut op_lines: Vec<(usize, String, &'static str, usize)> = Vec::new();
    for (idx, l) in lines.iter().enumerate() {
        let op = if l.ends_with(" &&") {
            Some("&&")
        } else if l.ends_with(" ||") {
            Some("||")
        } else {
            None
        };
        if let Some(op_str) = op {
            let prefix = l[..l.len() - 3].trim_end().to_string();
            let pre_w = display_width(&prefix);
            op_lines.push((idx, prefix, op_str, pre_w));
        }
    }
    if op_lines.len() < 2 {
        return;
    }
    let max_pre_w = op_lines.iter().map(|(_, _, _, w)| *w).max().unwrap_or(0);
    if max_pre_w + 3 > target_len {
        return;
    }
    for (idx, prefix, op_str, pre_w) in op_lines {
        let pad = " ".repeat(max_pre_w - pre_w + 1);
        lines[idx] = format!("{}{}{}", prefix, pad, op_str);
    }
}

fn try_wrap_conditional_groups(line: &str, target_len: usize, rec_depth: usize) -> Option<Vec<String>> {
    let code_only = strip_trailing_comment(line);
    let code_part = code_only.trim_end();
    let trailing_comment = &line[code_part.len()..];
    let trimmed_code = code_part.trim_start();
    if trimmed_code.starts_with("template ") || trimmed_code.starts_with("template<") {
        return None;
    }

    let base_tabs = line.chars().take_while(|&c| c == '\t').count();
    let cont_indent = "\t".repeat(base_tabs + 1);
    let chars: Vec<char> = code_part.chars().collect();
    let (scan_start, scan_end) = match find_control_flow_cond_parens(&chars) {
        Some((open_idx, close_idx)) => (open_idx + 1, close_idx),
        None => (0, chars.len()),
    };
    if scan_start >= scan_end {
        return None;
    }

    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut min_comma_depth: Option<usize> = None;
    let mut min_ternary_depth: Option<usize> = None;
    let mut logic_ops: Vec<(usize, usize)> = Vec::new();
    let mut i = scan_start;

    while i < scan_end {
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
            if i + 1 < scan_end && c == '/' && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < scan_end {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
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
                ',' if brace_depth == 0 && bracket_depth == 0 => {
                    min_comma_depth = Some(match min_comma_depth {
                        Some(d) => d.min(paren_depth),
                        None => paren_depth,
                    });
                }
                '?' if brace_depth == 0
                    && bracket_depth == 0
                    && i > scan_start
                    && i + 1 < scan_end
                    && chars[i - 1] == ' '
                    && chars[i + 1] == ' ' =>
                {
                    min_ternary_depth = Some(match min_ternary_depth {
                        Some(d) => d.min(paren_depth),
                        None => paren_depth,
                    });
                }
                '&' | '|'
                    if brace_depth == 0
                        && bracket_depth == 0
                        && i > scan_start
                        && i + 2 < scan_end
                        && chars[i - 1] == ' '
                        && chars[i + 1] == c
                        && chars[i + 2] == ' ' =>
                {
                    logic_ops.push((i, paren_depth));
                }
                _ => {}
            }
        }
        i += 1;
    }

    if logic_ops.is_empty() {
        return None;
    }
    let min_d = logic_ops.iter().map(|&(_, d)| d).min().unwrap();
    if min_d > 1 {
        return None;
    }
    if let Some(cd) = min_comma_depth {
        if cd <= min_d {
            return None;
        }
    }
    if let Some(td) = min_ternary_depth {
        if td < min_d {
            return None;
        }
    }

    let outer_ops: Vec<usize> = logic_ops
        .iter()
        .filter(|&&(_, d)| d == min_d)
        .map(|&(pos, _)| pos)
        .collect();
    if outer_ops.is_empty() {
        return None;
    }
    let has_compound_subgroup = logic_ops.iter().any(|&(_, d)| d > min_d);

    let mut clauses: Vec<(String, Option<&'static str>)> = Vec::with_capacity(outer_ops.len() + 1);
    for (k, &op_pos) in outer_ops.iter().enumerate() {
        let op_str = if chars[op_pos] == '&' { "&&" } else { "||" };
        if k == 0 {
            let first_seg: String = chars[..op_pos].iter().collect();
            clauses.push((first_seg.trim_end().to_string(), Some(op_str)));
        } else {
            let prev_end = outer_ops[k - 1] + 2;
            let mid_seg: String = chars[prev_end..op_pos].iter().collect();
            clauses.push((mid_seg.trim().to_string(), Some(op_str)));
        }
    }
    let last_start = *outer_ops.last().unwrap() + 2;
    let last_seg: String = chars[last_start..].iter().collect();
    clauses.push((last_seg.trim().to_string(), None));

    let mut packed_lines = Vec::<String>::new();
    if has_compound_subgroup {
        for (k, (seg, op)) in clauses.iter().enumerate() {
            let prefix = if k == 0 { "" } else { cont_indent.as_str() };
            let formatted = match op {
                Some(op_s) => format!("{}{} {}", prefix, seg, op_s),
                None => format!("{}{}", prefix, seg),
            };
            packed_lines.push(formatted);
        }
    } else {
        let mut cur_line = match clauses[0].1 {
            Some(op_s) => format!("{} {}", clauses[0].0, op_s),
            None => clauses[0].0.clone(),
        };
        for (seg, op) in clauses.iter().skip(1) {
            let piece = match op {
                Some(op_s) => format!("{} {}", seg, op_s),
                None => seg.clone(),
            };
            let candidate = format!("{} {}", cur_line, piece);
            if display_width(&candidate) <= target_len {
                cur_line = candidate;
            } else {
                packed_lines.push(cur_line);
                cur_line = format!("{}{}", cont_indent, piece);
            }
        }
        packed_lines.push(cur_line);
    }

    if packed_lines.len() < 2 {
        return None;
    }

    let mut out_lines = Vec::<String>::new();
    for pl in packed_lines {
        if rec_depth < 4 && display_width(&pl) > target_len {
            out_lines.extend(wrap_single_long_line_depth(&pl, target_len, rec_depth + 1));
        } else {
            out_lines.push(pl);
        }
    }

    if rec_depth == 0 {
        align_trailing_logical_ops(&mut out_lines, target_len);
    }

    if !trailing_comment.is_empty() {
        if let Some(last) = out_lines.last_mut() {
            last.push_str(trailing_comment);
        }
    }

    Some(out_lines)
}

fn try_expand_brace_initializer(line: &str, target_len: usize, rec_depth: usize) -> Option<Vec<String>> {
    let code_only = strip_trailing_comment(line);
    let code_part = code_only.trim_end();
    let trailing_comment = &line[code_part.len()..];
    if !code_part.ends_with("};") {
        return None;
    }

    let chars: Vec<char> = code_part.chars().collect();
    let n = chars.len();
    if n < 5 {
        return None;
    }
    let close_brace_idx = n - 2; // index of `}` in `};`

    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut eq_brace_open: Option<usize> = None;
    let mut matching_close: Option<usize> = None;
    let mut i = 0usize;

    while i < n {
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
            if i + 1 < n && c == '/' && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < n {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            match c {
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '=' if eq_brace_open.is_none()
                    && paren_depth == 0
                    && bracket_depth == 0
                    && brace_depth == 0 =>
                {
                    let prev_ok = i == 0 || !matches!(chars[i - 1], '=' | '!' | '<' | '>');
                    let next_ok = i + 1 < n && chars[i + 1] != '=';
                    if prev_ok && next_ok {
                        let mut k = i + 1;
                        while k < n && chars[k].is_whitespace() {
                            k += 1;
                        }
                        if k < n && chars[k] == '{' {
                            eq_brace_open = Some(k);
                        }
                    }
                }
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth = brace_depth.saturating_sub(1);
                    if eq_brace_open.is_some() && brace_depth == 0 && matching_close.is_none() {
                        matching_close = Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }

    let open_brace_idx = eq_brace_open?;
    if matching_close != Some(close_brace_idx) || open_brace_idx + 1 >= close_brace_idx {
        return None;
    }

    let elems = split_top_level_args(&chars[open_brace_idx + 1..close_brace_idx]);
    if elems.len() < 2 {
        return None;
    }

    let prefix_with_brace: String = chars[..=open_brace_idx].iter().collect();
    if elems[0].starts_with('{') {
        let first_inline_width = display_width(&prefix_with_brace) + display_width(&elems[0]) + 1;
        if first_inline_width <= target_len {
            return None;
        }
    }

    let base_tabs = line.chars().take_while(|&c| c == '\t').count();
    let elem_indent = "\t".repeat(base_tabs + 1);

    let mut packed_lines = Vec::<String>::new();
    packed_lines.push(prefix_with_brace);

    let mut cur_line = String::new();
    for (idx, elem) in elems.iter().enumerate() {
        let piece = if idx + 1 < elems.len() {
            format!("{},", elem)
        } else {
            format!("{}}};", elem)
        };
        if cur_line.is_empty() {
            cur_line = format!("{}{}", elem_indent, piece);
        } else {
            let candidate = format!("{} {}", cur_line, piece);
            if display_width(&candidate) <= target_len {
                cur_line = candidate;
            } else {
                packed_lines.push(cur_line);
                cur_line = format!("{}{}", elem_indent, piece);
            }
        }
    }
    if !cur_line.is_empty() {
        packed_lines.push(cur_line);
    }

    let mut out_lines = Vec::<String>::new();
    for pl in packed_lines {
        if rec_depth < 4 && display_width(&pl) > target_len {
            out_lines.extend(wrap_single_long_line_depth(&pl, target_len, rec_depth + 1));
        } else {
            out_lines.push(pl);
        }
    }

    if !trailing_comment.is_empty() {
        if let Some(last) = out_lines.last_mut() {
            last.push_str(trailing_comment);
        }
    }

    Some(out_lines)
}

fn try_wrap_template_declaration(line: &str, target_len: usize, rec_depth: usize) -> Option<Vec<String>> {
    let code_only = strip_trailing_comment(line);
    let code_part = code_only.trim_end();
    let trailing_comment = &line[code_part.len()..];
    let trimmed_code = code_part.trim_start();
    if !(trimmed_code.starts_with("template ") || trimmed_code.starts_with("template<")) {
        return None;
    }

    let chars: Vec<char> = code_part.chars().collect();
    let n = chars.len();
    let base_tabs = line.chars().take_while(|&c| c == '\t').count();
    let base_indent = "\t".repeat(base_tabs);
    let cont_indent = "\t".repeat(base_tabs + 1);

    let mut idx = 0usize;
    while idx < n && chars[idx].is_whitespace() {
        idx += 1;
    }
    // Skip "template"
    idx += "template".len();
    while idx < n && chars[idx].is_whitespace() {
        idx += 1;
    }
    if idx >= n || chars[idx] != '<' {
        return None;
    }
    let tmpl_open_idx = idx;

    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut angle_depth = 0usize;
    let mut tmpl_close_idx: Option<usize> = None;
    let mut top_commas: Vec<usize> = Vec::new();
    let mut i = tmpl_open_idx;

    while i < n {
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
            if i + 1 < n && c == '/' && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < n {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
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
                    if i > tmpl_open_idx && i + 1 < n && chars[i + 1] == '<' {
                        i += 2;
                        continue;
                    }
                    angle_depth += 1;
                }
                '>' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                    if i > 0 && chars[i - 1] == '-' {
                        i += 1;
                        continue;
                    }
                    angle_depth = angle_depth.saturating_sub(1);
                    if angle_depth == 0 {
                        tmpl_close_idx = Some(i);
                        break;
                    }
                }
                ',' if angle_depth == 1
                    && paren_depth == 0
                    && bracket_depth == 0
                    && brace_depth == 0 =>
                {
                    top_commas.push(i);
                }
                _ => {}
            }
        }
        i += 1;
    }

    let close_idx = tmpl_close_idx?;
    let after_tmpl_raw: String = chars[close_idx + 1..].iter().collect();
    let after_tmpl = after_tmpl_raw.trim_start();

    // Case A: `template <...> decl` exceeds column limit -> break right after `template <...>` at `base_indent`
    if !after_tmpl.is_empty() {
        let tmpl_line: String = chars[..=close_idx].iter().collect();
        let decl_line = format!("{}{}{}", base_indent, after_tmpl, trailing_comment);
        let mut out_lines = Vec::<String>::new();
        if rec_depth < 4 && display_width(&tmpl_line) > target_len {
            out_lines.extend(wrap_single_long_line_depth(&tmpl_line, target_len, rec_depth + 1));
        } else {
            out_lines.push(tmpl_line);
        }
        if rec_depth < 4 && display_width(strip_trailing_comment(&decl_line)) > target_len {
            out_lines.extend(wrap_single_long_line_depth(&decl_line, target_len, rec_depth + 1));
        } else {
            out_lines.push(decl_line);
        }
        return Some(out_lines);
    }

    // Case B: `template <...>` alone exceeds column limit -> wrap at top-level template parameter commas
    if top_commas.is_empty() {
        return None;
    }

    let prefix_open: String = chars[..=tmpl_open_idx].iter().collect();
    let mut params: Vec<String> = Vec::with_capacity(top_commas.len() + 1);
    let mut seg_start = tmpl_open_idx + 1;
    for &c_pos in &top_commas {
        let seg: String = chars[seg_start..c_pos].iter().collect();
        params.push(seg.trim().to_string());
        seg_start = c_pos + 1;
    }
    let last_seg: String = chars[seg_start..close_idx].iter().collect();
    params.push(last_seg.trim().to_string());

    let mut packed = Vec::<String>::new();
    let mut cur_line = String::new();
    for (k, param) in params.iter().enumerate() {
        let suffix = if k + 1 < params.len() { "," } else { ">" };
        let piece = format!("{}{}", param, suffix);
        if k == 0 {
            cur_line = format!("{}{}", prefix_open, piece);
        } else {
            let candidate = format!("{} {}", cur_line, piece);
            if display_width(&candidate) <= target_len {
                cur_line = candidate;
            } else {
                packed.push(cur_line);
                cur_line = format!("{}{}", cont_indent, piece);
            }
        }
    }
    if !cur_line.is_empty() {
        packed.push(cur_line);
    }
    if packed.len() < 2 {
        return None;
    }
    if !trailing_comment.is_empty() {
        if let Some(last) = packed.last_mut() {
            last.push_str(trailing_comment);
        }
    }
    Some(packed)
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

    if let Some(tmpl_lines) = try_wrap_template_declaration(line, target_len, rec_depth) {
        return tmpl_lines;
    }

    if let Some(str_lines) = try_split_concatenated_strings(line, base_tabs) {
        return str_lines;
    }

    if let Some(brace_lines) = try_expand_brace_initializer(line, target_len, rec_depth) {
        return brace_lines;
    }

    if rec_depth < 4 && !has_top_level_logical_op(code_only) {
        if let Some(expanded) = try_expand_paren_call(line, target_len, rec_depth) {
            return expanded;
        }
    }

    if let Some(cond_lines) = try_wrap_conditional_groups(line, target_len, rec_depth) {
        return cond_lines;
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
        let mut fallback_comma: Option<(usize, usize)> = None;
        let mut best_logic: Option<(usize, usize)> = None; // (char_idx_after_op, depth)
        let mut fallback_logic: Option<(usize, usize)> = None;
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
                        fallback_comma = Some((i + 1, depth));
                    } else if best_comma.is_none() && fallback_comma.is_none() {
                        fallback_comma = Some((i + 1, depth));
                    }
                }
                '|' | '&' if i + 2 < chars.len() && i > 0 && chars[i - 1] == ' ' && chars[i + 1] == c && chars[i + 2] == ' ' => {
                    if col + 2 <= target_len {
                        match best_logic {
                            None => best_logic = Some((i + 2, depth)),
                            Some((_, best_d)) if depth <= best_d => best_logic = Some((i + 2, depth)),
                            _ => {}
                        }
                    } else {
                        match fallback_logic {
                            None => fallback_logic = Some((i + 2, depth)),
                            Some((_, fb_d)) if depth < fb_d => fallback_logic = Some((i + 2, depth)),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        let comma_cand = best_comma.or(fallback_comma);
        let logic_cand = best_logic.or(fallback_logic);
        let split_at = match (comma_cand, logic_cand) {
            (Some((c_idx, c_d)), Some((l_idx, l_d))) => {
                if l_d < c_d {
                    Some(l_idx)
                } else {
                    Some(c_idx)
                }
            }
            (Some((c_idx, _)), None) => Some(c_idx),
            (None, Some((l_idx, _))) => Some(l_idx),
            (None, None) => None,
        };

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
    let mut in_macro_cont = false;
    let mut in_macro_block_comment = false;
    for line in file.lines() {
        if in_macro_cont {
            wrapped.push(line.to_string());
            if in_macro_block_comment {
                if line.contains("*/") {
                    in_macro_block_comment = false;
                }
            } else if has_unclosed_block_comment(line) {
                in_macro_block_comment = true;
            }
            in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            wrapped.push(line.to_string());
            in_macro_block_comment = has_unclosed_block_comment(trimmed);
            in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
            continue;
        }
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
    let allman = matches!(style, Style::Allman);
    let updated = {
        let lines: Vec<&str> = file.split('\n').collect();
        let mut parts: Vec<Cow<'_, str>> = Vec::with_capacity(lines.len());
        let mut temp = String::new();
        let mut end = true;
        let mut in_macro_cont = false;
        let mut in_macro_block_comment = false;
        for (i, &line) in lines.iter().enumerate() {
            if in_macro_cont {
                parts.push(Cow::Borrowed(line));
                if in_macro_block_comment {
                    if line.contains("*/") {
                        in_macro_block_comment = false;
                    }
                } else if has_unclosed_block_comment(line) {
                    in_macro_block_comment = true;
                }
                in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
                continue;
            }
            let t_start = line.trim_start();
            if t_start.starts_with('#') {
                parts.push(Cow::Borrowed(line));
                in_macro_block_comment = has_unclosed_block_comment(t_start);
                in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
                continue;
            }
            if allman && i + 1 < lines.len() {
                let check = line.trim();
                let check2 = lines[i + 1].trim();
                if check.starts_with("switch") && (!check2.ends_with('{') && !check.ends_with(';')) {
                    temp.push_str(line.trim_end());
                    temp.push(' ');
                    end = false;
                    continue;
                }
            } else {
                let check = line.trim();
                if check.starts_with("switch") && (!check.ends_with('{') && !check.ends_with(';')) {
                    temp.push_str(line.trim_end());
                    temp.push(' ');
                    end = false;
                    continue;
                }
            }
            if !end && line.trim_end().ends_with(';') {
                end = true;
                temp.push_str(line.trim());
                parts.push(Cow::Owned(std::mem::take(&mut temp)));
                continue;
            } else if !end {
                temp.push_str(line.trim());
                temp.push(' ');
                continue;
            }
            parts.push(Cow::Borrowed(line));
        }
        parts.join("\n")
    };
    *file = updated;
}

fn shift_back_preproc_lines(file: &mut String) {
    let updated = {
        let lines = file.split('\n');
        let mut parts: Vec<&str> = Vec::new();
        let mut in_macro_cont = false;
        let mut in_macro_block_comment = false;
        for line in lines {
            if in_macro_cont {
                parts.push(line);
                if in_macro_block_comment {
                    if line.contains("*/") {
                        in_macro_block_comment = false;
                    }
                } else if has_unclosed_block_comment(line) {
                    in_macro_block_comment = true;
                }
                in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
                continue;
            }
            let trimmed = line_trimmed_for_comment_check(line);
            let code_only = strip_trailing_comment(line);
            if trimmed.starts_with('#') {
                parts.push(line.trim_start());
                in_macro_block_comment = has_unclosed_block_comment(trimmed);
                in_macro_cont = line.trim_end().ends_with('\\') || in_macro_block_comment;
            } else if !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with('*')
                && code_only.trim_end().ends_with(':')
            {
                parts.push(remove_single_tab(line));
            } else {
                parts.push(line);
            }
        }
        parts.join("\n")
    };
    *file = updated;
}

pub fn close_empty_curly_brace_blocks(file: &mut String) {
    let has_trailing_nl = file.ends_with('\n');
    let lines: Vec<String> = file.lines().map(|x| x.to_string()).collect();
    if lines.len() < 2 {
        return;
    }
    let mut out = Vec::<String>::with_capacity(lines.len());
    let mut in_block_comment = false;
    let mut in_macro_cont = false;
    let mut raw_end: Option<String> = None;
    let mut i = 0usize;

    while i < lines.len() {
        let l = &lines[i];
        let trimmed = line_trimmed_for_comment_check(l);

        if in_macro_cont {
            in_macro_cont = l.trim_end().ends_with('\\');
            out.push(l.clone());
            i += 1;
            continue;
        }
        if trimmed.starts_with('#') && l.trim_end().ends_with('\\') {
            in_macro_cont = true;
            out.push(l.clone());
            i += 1;
            continue;
        }

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

        // Collapse same-line `{` + next-line `}` (empty control flow and empty lambdas, excluding `case`/`default`)
        let code_i_start = strip_trailing_comment(&lines[i]).trim_start();
        let is_case_or_default = code_i_start.starts_with("case ")
            || code_i_start.starts_with("case(")
            || code_i_start.starts_with("default:")
            || code_i_start.starts_with("default :");
        if i + 1 < lines.len()
            && !is_case_or_default
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

pub fn sort_include_groups(file: &str) -> String {
    let mut result: Vec<&str> = Vec::new();
    let mut temp_group: Vec<&str> = Vec::new();
    for line in file.lines() {
        if line.trim_start().starts_with("#include") {
            temp_group.push(line);
        } else {
            if !temp_group.is_empty() {
                sort_includes(&mut temp_group);
                result.extend(temp_group.drain(..));
            }
            result.push(line);
        }
    }
    if !temp_group.is_empty() {
        sort_includes(&mut temp_group);
        result.extend(temp_group.drain(..));
    }
    result.join("\n")
}

fn strip_include_comment(line: &str) -> &str {
    match line.find("//") {
        Some(idx) => &line[..idx],
        None => line,
    }
}

fn sort_includes(includes: &mut [&str]) {
    includes.sort_by(|a, b| {
        let a2 = strip_include_comment(a);
        let b2 = strip_include_comment(b);
        let a_has_angle = a2.contains('<') && a2.contains('>');
        let b_has_angle = b2.contains('<') && b2.contains('>');
        match (a_has_angle, b_has_angle) {
            (true, false) => Ordering::Less,    // Angle brackets come first
            (false, true) => Ordering::Greater, // Double quotes come after
            _ => a.cmp(b), // If both are the same type, sort normally
        }
    });
}

pub fn add_all_leading_tabs(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for (idx, line) in text.split('\n').enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push('\t');
        out.push_str(line);
    }
    out
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

pub fn fix_stars(file: &str) -> String {
    let has_trailing_newline = file.ends_with('\n');
    let mut parts: Vec<Cow<'_, str>> = Vec::new();
    let mut in_block_comment = false;

    for line in file.lines() {
        let trimmed = line_trimmed_for_comment_check(line);
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            parts.push(Cow::Borrowed(line));
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            parts.push(Cow::Borrowed(line));
            continue;
        }
        if trimmed.starts_with("//") || trimmed.starts_with('#') || line.contains("R\"") || !line.contains('*') {
            if has_unclosed_block_comment(line) {
                in_block_comment = true;
            }
            parts.push(Cow::Borrowed(line));
            continue;
        }

        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut temp_line = String::with_capacity(line.len() + 4);
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
        parts.push(Cow::Owned(temp_line));
    }
    let mut joined = parts.join("\n");
    if has_trailing_newline {
        joined.push('\n');
    }
    return joined;
}

pub fn remove_dereference_spaces(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::with_capacity(line.len());
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

pub fn remove_pointer_spaces(line: &str) -> String {
    if line.contains('\n') {
        return line
            .split('\n')
            .map(|l| {
                let t = line_trimmed_for_comment_check(l);
                if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
                    Cow::Borrowed(l)
                } else {
                    Cow::Owned(remove_pointer_spaces_single(l))
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    remove_pointer_spaces_single(line)
}

fn remove_pointer_spaces_single(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::with_capacity(line.len());
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

pub fn switch_pointer_spaces(line: &str) -> String {
    return remove_pointer_spaces(line);
}

pub fn remove_reference_spaces(line: &str) -> String {
    if line.contains('\n') {
        return line
            .split('\n')
            .map(|l| {
                let t = line_trimmed_for_comment_check(l);
                if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
                    Cow::Borrowed(l)
                } else {
                    Cow::Owned(remove_reference_spaces_single(l))
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    remove_reference_spaces_single(line)
}

fn remove_reference_spaces_single(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut result = String::with_capacity(line.len());
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
            let inside_angles = result.matches('<').count() > result.matches('>').count();
            let is_single_ref = chars[i + 1] == '&' && (i + 2 >= chars.len() || chars[i + 2] != '&');
            let is_rvalue_ref = !after_eq
                && chars[i + 1] == '&'
                && i + 2 < chars.len()
                && chars[i + 2] == '&'
                && (!inside_angles || i + 3 >= chars.len() || chars[i + 3] != ' ');
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

fn remove_single_tab(line: &str) -> &str {
    line.strip_prefix('\t').unwrap_or(line)
}

pub fn remove_all_spaces(line: &str) -> String {
    line.chars().filter(|&c| c != ' ').collect()
}

pub fn remove_object_constructor_space(line: &str) -> String {
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
    line.to_string()
}

pub fn remove_unnecessary_spaces(line: &str) -> String {
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

pub fn remove_double_spaces(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ' ' && chars.peek() == Some(&' ') {
            continue;
        }
        result.push(c);
    }
    result
}

pub fn remove_whitespace_before_commas(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ' ' && chars.peek() == Some(&',') {
            continue;
        }
        result.push(c);
    }
    result
}

pub fn add_leading_whitespace(src: &str, amount: usize) -> String {
    let mut out = String::with_capacity(amount + src.len());
    for _ in 0..amount {
        out.push(' ');
    }
    out.push_str(src);
    out
}

pub fn count_leading_chars(src: &str, target: char) -> usize {
    src.chars().take_while(|&c| c == target).count()
}

pub fn sanitize(input: &str) -> String {
    let allowed_chars = Regex::new(r"[a-zA-Z0-9.\-_\+\=/\s]+").unwrap();
    let captures = allowed_chars.captures(input).unwrap();
    captures.get(0).unwrap().as_str().to_string()
}

fn line_has_no_alphanumeric(line: &str) -> bool {
    !line.chars().any(|c| c.is_alphanumeric())
}

pub fn remove_blank_lines(lines: Vec<&str>) -> String {
    let result: Vec<&str> = lines.into_iter().filter(|line| !line.trim().is_empty()).collect();
    return result.join("\n");
}

pub fn write_file(path: &str, contents: &[u8]) -> Result<(), Error> {
    let mut dest = File::create(path).unwrap();
    dest.write_all(contents)
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

pub const FILE_EXTENSIONS: &[&str] = &[
    ".cpp", ".cc", ".C", ".c", ".hpp", ".hh", ".H", ".h", ".go", ".py", ".txt",
];

pub fn ends_with_brace(line: &str) -> bool {
    line.trim_end().ends_with('{')
}

fn check_line_is_blank(line: &str) -> bool {
    line.is_empty() || line.chars().all(|x| x.is_whitespace() || x == '\t')
}

pub fn infer_file_type(filepath: &str) -> &'static str {
    for &item in FILE_EXTENSIONS {
        if filepath.ends_with(item) {
            return item;
        }
    }
    for &item in FILE_EXTENSIONS {
        if filepath.contains(item) {
            return item;
        }
    }
    "unknown"
}

pub fn get_file_extensions_list() -> &'static [&'static str] {
    // .cpp has to stay before .c, otherwise it breaks
    FILE_EXTENSIONS
}

pub fn display_file_extensions() {
    for &line in get_file_extensions_list() {
        println!("{}", line);
    }
}

pub fn check_valid_file_ext(path: &str) -> bool {
    get_file_extensions_list().iter().any(|&ext| path.contains(ext))
}

pub fn print_usage() {
println!(
"
USAGE:\nblfmt [flags + opts] <file-path>

FLAGS:
    -h or --help
    -w or --write
(Code only):
    -  or --stdin <file-ext>
(C/C++ only):
    -s or --style allman|knr|stroustrup(default)
(Txt only):
    -o, --opts, or --options <columns> <spacing> (numbers)
    -t or --titles \"Places\" \"where\" \"you\" \"want\" \"paragraphs\" ...
    (Flags and options may be placed in any order)
"
);
}

pub fn write_results(path: &str, results: &str) {
    let ok = write_file(path, results.as_bytes());
    match ok {
        Ok(_) => println!("Successfully wrote: '{}'", path),
        Err(e) => println!("Error during `format_txt_file()`: {}", e),
    };
}

pub fn print_results(results: &str) {
    println!("{}", results);
}

pub fn find_define_end_byte(start_byte: usize, src: &str) -> usize {
    let bytes = src.as_bytes();
    let n = bytes.len();
    let mut i = start_byte;
    let mut in_string = false;
    let mut in_char = false;
    let mut in_block_comment = false;
    let mut in_line_comment = false;

    while i < n {
        let b = bytes[i];
        if in_line_comment {
            if b == b'\n' {
                return i;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if i + 1 < n && b == b'*' && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_string {
            if b == b'\\' && i + 1 < n && bytes[i + 1] != b'\n' && bytes[i + 1] != b'\r' {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_string = false;
                i += 1;
                continue;
            }
            if b == b'\n' {
                let mut k = i;
                while k > start_byte && (bytes[k - 1] == b'\r' || bytes[k - 1] == b' ' || bytes[k - 1] == b'\t') {
                    k -= 1;
                }
                if k > start_byte && bytes[k - 1] == b'\\' {
                    i += 1;
                    continue;
                }
                return i;
            }
            i += 1;
            continue;
        }
        if in_char {
            if b == b'\\' && i + 1 < n && bytes[i + 1] != b'\n' && bytes[i + 1] != b'\r' {
                i += 2;
                continue;
            }
            if b == b'\'' {
                in_char = false;
                i += 1;
                continue;
            }
            if b == b'\n' {
                return i;
            }
            i += 1;
            continue;
        }

        if i + 1 < n && b == b'/' && bytes[i + 1] == b'*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if i + 1 < n && b == b'/' && bytes[i + 1] == b'/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if b == b'\'' {
            in_char = true;
            i += 1;
            continue;
        }
        if b == b'\n' {
            let mut k = i;
            while k > start_byte && (bytes[k - 1] == b'\r' || bytes[k - 1] == b' ' || bytes[k - 1] == b'\t') {
                k -= 1;
            }
            if k > start_byte && bytes[k - 1] == b'\\' {
                i += 1;
                continue;
            }
            return i;
        }
        i += 1;
    }
    n
}

pub fn consume_define_node(node: tree_sitter::Node, src: &str) -> (String, usize) {
    let start = node.start_byte();
    let full_end = find_define_end_byte(start, src).max(node.end_byte());
    let mut effective_end = node.end_byte();
    if full_end > node.end_byte() {
        let mut safe_to_extend = true;
        let mut sib_opt = node.next_sibling();
        while let Some(sib) = sib_opt {
            if sib.start_byte() >= full_end {
                break;
            }
            if sib.end_byte() > full_end + 1 {
                safe_to_extend = false;
                break;
            }
            sib_opt = sib.next_sibling();
        }
        if safe_to_extend {
            effective_end = full_end;
        }
    }
    let formatted = format_define_directive(&src[start..effective_end]);
    (formatted, effective_end)
}

fn strip_line_continuation(line: &str) -> (&str, bool) {
    let rtrimmed = line.trim_end_matches(|c| c == ' ' || c == '\t' || c == '\r');
    if let Some(without_slash) = rtrimmed.strip_suffix('\\') {
        (without_slash.trim_end_matches(|c| c == ' ' || c == '\t'), true)
    } else {
        (rtrimmed, false)
    }
}

fn visual_column_width(line: &str, tab_width: usize) -> usize {
    let mut col = 0usize;
    for c in line.chars() {
        if c == '\t' {
            col = (col / tab_width + 1) * tab_width;
        } else {
            col += 1;
        }
    }
    col
}

fn format_macro_param_list(params_inner: &str) -> String {
    if params_inner.trim().is_empty() {
        return "()".to_string();
    }
    let formatted: Vec<&str> = params_inner.split(',').map(|p| p.trim()).collect();
    format!("({})", formatted.join(", "))
}

fn parse_define_signature(line0_core: &str) -> Option<(String, &str, bool)> {
    let s = line0_core.trim_start();
    let after_hash = s.strip_prefix('#')?.trim_start();
    let after_kw = after_hash.strip_prefix("define")?;
    if !after_kw.is_empty() && !after_kw.starts_with(|c: char| c.is_ascii_whitespace()) {
        return None;
    }
    let rest = after_kw.trim_start();
    let name_len = rest
        .bytes()
        .take_while(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
        .count();
    if name_len == 0 {
        return None;
    }
    let name = &rest[..name_len];
    let after_name = &rest[name_len..];
    if after_name.starts_with('(') {
        // Function-like macro: '(' immediately follows NAME with no whitespace
        let bytes = after_name.as_bytes();
        let mut depth = 0usize;
        let mut close_idx = None;
        for (idx, &b) in bytes.iter().enumerate() {
            if b == b'(' {
                depth += 1;
            } else if b == b')' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    close_idx = Some(idx);
                    break;
                }
            }
        }
        if let Some(cidx) = close_idx {
            let params_fmt = format_macro_param_list(&after_name[1..cidx]);
            let sig = format!("#define {}{}", name, params_fmt);
            return Some((sig, &after_name[cidx + 1..], false));
        } else {
            // Parameter list itself spans multiple lines
            let sig = format!("#define {}{}", name, after_name.trim_end());
            return Some((sig, "", true));
        }
    }
    Some((format!("#define {}", name), after_name, false))
}

fn pop_matching_delim(stack: &mut Vec<char>, closer: char) {
    let target = match closer {
        '}' => '{',
        ')' => '(',
        ']' => '[',
        _ => return,
    };
    if target == '{' {
        if let Some(pos) = stack.iter().rposition(|&c| c == '{') {
            stack.truncate(pos);
        }
    } else if let Some(pos) = stack.iter().rposition(|&c| c == target || c == '{') {
        if stack[pos] == target {
            stack.remove(pos);
        }
    }
}

fn update_define_delim_stack(s: &str, stack: &mut Vec<char>, in_block_comment: &mut bool) {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while i < n {
        let c = chars[i];
        if *in_block_comment {
            if i + 1 < n && c == '*' && chars[i + 1] == '/' {
                *in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
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
            if i + 1 < n && c == '/' && chars[i + 1] == '/' {
                break;
            }
            if i + 1 < n && c == '/' && chars[i + 1] == '*' {
                *in_block_comment = true;
                i += 2;
                continue;
            }
            match c {
                '{' | '(' | '[' => stack.push(c),
                '}' | ')' | ']' => pop_matching_delim(stack, c),
                _ => {}
            }
        }
        i += 1;
    }
}

pub fn format_define_directive(raw_define: &str) -> String {
    let raw_define = raw_define.trim_end_matches(|c| c == '\n' || c == '\r');
    let raw_lines: Vec<&str> = raw_define.split('\n').collect();
    if raw_lines.len() <= 1 {
        let line = raw_lines.first().copied().unwrap_or("").trim();
        if let Some((sig, rem, _)) = parse_define_signature(line) {
            let body = rem.trim();
            if body.is_empty() {
                return sig;
            } else {
                return format!("{} {}", sig, body);
            }
        }
        return line.to_string();
    }

    let (line0_core, line0_has_slash) = strip_line_continuation(raw_lines[0]);
    let Some((sig, rem0, unclosed_params_on_line0)) = parse_define_signature(line0_core) else {
        return raw_define.trim_end().to_string();
    };

    let mut out_lines: Vec<(String, bool)> = Vec::with_capacity(raw_lines.len() + 1);
    let mut body_items: Vec<(&str, bool)> = Vec::with_capacity(raw_lines.len());

    let rem0_trim = rem0.trim();
    let rem0_is_only_comment = !rem0_trim.is_empty()
        && (rem0_trim.starts_with("//")
            || (rem0_trim.starts_with("/*") && rem0_trim.ends_with("*/") && !rem0_trim[2..rem0_trim.len() - 2].contains("*/")));

    if unclosed_params_on_line0 || rem0_trim.is_empty() {
        out_lines.push((sig, line0_has_slash));
    } else if rem0_is_only_comment {
        out_lines.push((format!("{} {}", sig, rem0_trim), line0_has_slash));
    } else {
        out_lines.push((sig, true));
        body_items.push((rem0_trim, line0_has_slash));
    }

    for &rl in &raw_lines[1..] {
        body_items.push(strip_line_continuation(rl));
    }

    let mut delim_stack: Vec<char> = Vec::new();
    let mut in_block_comment = false;
    let mut prev_incomplete_expr = false;
    if unclosed_params_on_line0 {
        delim_stack.push('(');
    }

    for (raw_core, has_slash) in body_items {
        let trimmed = raw_core.trim();
        if trimmed.is_empty() {
            out_lines.push((String::new(), has_slash));
            continue;
        }

        if in_block_comment {
            let brace_count = delim_stack.iter().filter(|&&c| c == '{').count();
            let in_open_paren = delim_stack.last().map(|&c| c == '(' || c == '[').unwrap_or(false);
            let tabs = 1 + brace_count + usize::from(in_open_paren);
            out_lines.push((format!("{}{}", "\t".repeat(tabs), trimmed), has_slash));
            update_define_delim_stack(trimmed, &mut delim_stack, &mut in_block_comment);
            continue;
        }

        // Pop leading closing delimiters at the start of this line before computing its indentation
        let mut lead_bytes = 0usize;
        for ch in trimmed.chars() {
            if matches!(ch, '}' | ')' | ']') {
                pop_matching_delim(&mut delim_stack, ch);
                lead_bytes += ch.len_utf8();
            } else if ch == ' ' || ch == '\t' {
                lead_bytes += ch.len_utf8();
            } else {
                break;
            }
        }

        let brace_count = delim_stack.iter().filter(|&&c| c == '{').count();
        let in_open_paren = delim_stack.last().map(|&c| c == '(' || c == '[').unwrap_or(false);
        let is_case_or_default = brace_count > 0
            && (trimmed.starts_with("case ")
                || trimmed.starts_with("case(")
                || trimmed.starts_with("default:")
                || trimmed.starts_with("default :"));
        let effective_brace = if is_case_or_default {
            brace_count - 1
        } else {
            brace_count
        };
        let extra_expr = if !in_open_paren
            && prev_incomplete_expr
            && !trimmed.starts_with('{')
            && !trimmed.starts_with('}')
            && !trimmed.starts_with("else")
            && !trimmed.starts_with("while")
        {
            1
        } else {
            0
        };
        let tabs = if trimmed.starts_with('#') && !trimmed.starts_with("##") && trimmed.len() > 1 && trimmed[1..].trim_start().starts_with(|c: char| c.starts_with_preproc_kw(trimmed)) {
            0
        } else {
            1 + effective_brace + usize::from(in_open_paren) + extra_expr
        };

        out_lines.push((format!("{}{}", "\t".repeat(tabs), trimmed), has_slash));

        update_define_delim_stack(&trimmed[lead_bytes..], &mut delim_stack, &mut in_block_comment);

        let code_no_comment = strip_trailing_comment(trimmed).trim_end();
        if !code_no_comment.is_empty() && !trimmed.starts_with("/*") {
            let in_paren_after = delim_stack.last().map(|&c| c == '(' || c == '[').unwrap_or(false);
            let is_colon_label = code_no_comment.ends_with(':')
                && !code_no_comment.ends_with("::")
                && (is_case_or_default || !code_no_comment.contains('?'));
            prev_incomplete_expr = !in_paren_after
                && !is_colon_label
                && code_no_comment.ends_with(|c: char| {
                    matches!(c, '=' | '+' | '-' | '*' | '/' | '%' | '|' | '&' | '^' | '<' | '>' | '?' | ':')
                });
        }
    }

    let mut rendered = Vec::<String>::with_capacity(out_lines.len());
    for (content, has_slash) in out_lines {
        if has_slash {
            if content.is_empty() {
                rendered.push("\\".to_string());
            } else {
                rendered.push(format!("{} \\", content));
            }
        } else {
            rendered.push(content);
        }
    }
    rendered.join("\n")
}

trait PreprocKwHelper {
    fn starts_with_preproc_kw(self, full_trimmed: &str) -> bool;
}

impl PreprocKwHelper for char {
    fn starts_with_preproc_kw(self, full_trimmed: &str) -> bool {
        let after = full_trimmed.strip_prefix('#').unwrap_or("").trim_start();
        after.starts_with("if ")
            || after.starts_with("if(")
            || after.starts_with("ifdef ")
            || after.starts_with("ifndef ")
            || after.starts_with("elif ")
            || after.starts_with("elif(")
            || after == "else"
            || after.starts_with("else ")
            || after == "endif"
            || after.starts_with("endif ")
            || after.starts_with("endif/")
    }
}

pub fn align_multiline_define_backslashes(file: &mut String, target_col_limit: usize) {
    let has_trailing_nl = file.ends_with('\n');
    let mut lines: Vec<String> = file.lines().map(|s| s.to_string()).collect();
    let n = lines.len();
    let mut i = 0usize;

    while i < n {
        let t = lines[i].trim_start();
        if t.starts_with("#define") && lines[i].trim_end().ends_with('\\') {
            let start = i;
            let mut end = i;
            let mut in_comment = has_unclosed_block_comment(&lines[i]);
            while end + 1 < n && (lines[end].trim_end().ends_with('\\') || in_comment) {
                end += 1;
                if in_comment {
                    if lines[end].contains("*/") {
                        in_comment = false;
                    }
                } else if has_unclosed_block_comment(&lines[end]) {
                    in_comment = true;
                }
            }

            let mut max_w = 0usize;
            let mut stripped: Vec<(String, bool, usize)> = Vec::with_capacity(end - start + 1);
            for line in &lines[start..=end] {
                let (core, has_slash) = strip_line_continuation(line);
                let w = if has_slash { visual_column_width(core, 4) } else { 0 };
                if has_slash && w > max_w {
                    max_w = w;
                }
                stripped.push((core.to_string(), has_slash, w));
            }

            let slash_col = target_col_limit.saturating_sub(1).max(max_w + 1);
            for (offset, (core, has_slash, w)) in stripped.into_iter().enumerate() {
                if has_slash {
                    let pad = slash_col.saturating_sub(w).max(usize::from(!core.is_empty()));
                    lines[start + offset] = format!("{}{}\\", core, " ".repeat(pad));
                }
            }

            i = end + 1;
        } else {
            i += 1;
        }
    }

    let mut out = lines.join("\n");
    if has_trailing_nl && !out.ends_with('\n') {
        out.push('\n');
    }
    *file = out;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_single_line_defines_on_one_line() {
        assert_eq!(
            format_define_directive("#define PATH_BUFFER_MAX 1024\n"),
            "#define PATH_BUFFER_MAX 1024"
        );
        assert_eq!(
            format_define_directive("#define CLAMP(val,lo,hi) ((val) < (lo) ? (lo) : (val))\n"),
            "#define CLAMP(val, lo, hi) ((val) < (lo) ? (lo) : (val))"
        );
        assert_eq!(
            format_define_directive("#define GLAD_GLX\n"),
            "#define GLAD_GLX"
        );
    }

    #[test]
    fn formats_and_aligns_multiline_define_backslashes() {
        let raw = "#define P(a,b,c) \\\ndo { \\\n  if (a) { \\\n    b = c; \\\n  } \\\n} while (0)\n";
        let mut formatted = format_define_directive(raw);
        align_multiline_define_backslashes(&mut formatted, 100);
        let lines: Vec<&str> = formatted.lines().collect();
        assert_eq!(lines.len(), 6);
        for line in &lines[..5] {
            assert!(line.ends_with('\\'));
            assert_eq!(visual_column_width(line, 4), 100);
        }
        assert!(lines[1].starts_with("\tdo {"));
        assert!(lines[2].starts_with("\t\tif (a) {"));
        assert!(lines[3].starts_with("\t\t\tb = c;"));
        assert!(lines[4].starts_with("\t\t}"));
        assert_eq!(lines[5], "\t} while (0)");
    }
}


