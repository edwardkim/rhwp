//! Native Rust regression-sweep signature tool.
//!
//! Prints one line per PDF in a corpus directory:
//!
//! ```text
//! <relpath>\t<text_hash>\t<nchars>\t<max_word_len>\t<page_rotations>
//! ```
//!
//! Build once per version and diff the outputs to find extraction regressions:
//!
//! ```text
//! cargo build --release --bin corpus_sig --jobs 3
//! ./target/release/corpus_sig <corpus_dir> > head.txt
//! # (in a v0.3.71 worktree) ./target/release/corpus_sig <corpus_dir> > base.txt
//! diff base.txt head.txt
//! ```
//!
//! Single process, release speed — no Python, no per-doc subprocess.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use pdf_oxide::document::PdfDocument;

fn collect_pdfs(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                collect_pdfs(&p, out);
            } else if p.extension().and_then(|e| e.to_str()) == Some("pdf") {
                out.push(p);
            }
        }
    }
}

fn main() {
    let root = std::env::args()
        .nth(1)
        .expect("usage: corpus_sig <corpus_dir>");
    let root = PathBuf::from(root);
    let mut pdfs = Vec::new();
    collect_pdfs(&root, &mut pdfs);
    eprintln!("corpus: {} pdfs", pdfs.len());

    for pdf in &pdfs {
        let rel = pdf.strip_prefix(&root).unwrap_or(pdf).display();
        let doc = match PdfDocument::open(pdf) {
            Ok(d) => d,
            Err(_) => {
                println!("{rel}\tOPEN_ERR\t0\t0\t[]");
                continue;
            },
        };
        let n = doc.page_count().unwrap_or(0);
        let mut all_text = String::new();
        let mut rots: Vec<i32> = Vec::with_capacity(n);
        let mut max_word = 0usize;
        for i in 0..n {
            rots.push(doc.get_page_rotation(i).unwrap_or(0));
            if let Ok(t) = doc.extract_text(i) {
                all_text.push_str(&t);
                all_text.push('\n');
            }
            if let Ok(words) = doc.extract_words(i) {
                for w in &words {
                    max_word = max_word.max(w.text.chars().count());
                }
            }
        }
        let mut h = DefaultHasher::new();
        all_text.hash(&mut h);
        println!(
            "{rel}\t{:016x}\t{}\t{}\t{:?}",
            h.finish(),
            all_text.chars().count(),
            max_word,
            rots
        );
    }
}
