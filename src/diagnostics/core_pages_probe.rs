//! [#2169] DocumentCore(편집기 코어) 페이지 맵 프로브.
//!
//! 뷰어 코어(wasm_api dump-pages)와 편집기 코어(DocumentCore)의 페이지네이션
//! 갈림 지점을 특정한다. 페이지별 첫 텍스트(≤20자)를 출력해 diff 가능하게 한다.
//!
//! 사용: rhwp core-pages <파일.hwp>

use crate::document_core::DocumentCore;
use crate::renderer::render_tree::{RenderNode, RenderNodeType};

fn first_text(n: &RenderNode, out: &mut String) {
    if out.len() >= 60 {
        return;
    }
    if let RenderNodeType::TextRun(tr) = &n.node_type {
        if !tr.text.trim().is_empty() && out.len() < 60 {
            out.push_str(tr.text.trim());
            out.push(' ');
        }
    }
    for c in &n.children {
        first_text(c, out);
        if out.len() >= 60 {
            return;
        }
    }
}

pub fn run(args: &[String]) {
    let Some(path) = args.first() else {
        eprintln!("사용: rhwp core-pages <파일>");
        return;
    };
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("읽기 실패: {e}");
            return;
        }
    };
    let core = match DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("파싱 실패: {e:?}");
            return;
        }
    };
    let n = core.page_count();
    println!("core pages: {n}");
    for p in 0..n {
        let mut s = String::new();
        if let Ok(tree) = core.build_page_render_tree(p) {
            first_text(&tree.root, &mut s);
        }
        let head: String = s.chars().take(24).collect();
        println!("p{:03} {}", p + 1, head);
    }
}
