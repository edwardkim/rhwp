//! 관계형: 검색 주소·입력 범위·리비전과 치환의 상태 보존을 검증한다.
//! 시각 측정 비해당. 고정된 페이지 좌표를 검사하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::thread;
use std::time::{Duration, Instant};

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::model::control::Control;
use rhwp::native_host::runtime::{
    NativeRuntime, NativeRuntimeCommand as Command, NativeRuntimeConfig, NativeRuntimeFailure,
    NativeRuntimeOutput as Output, NativeRuntimeTarget,
};
use rhwp::native_host::search::{NativeSearchRequest, NativeSearchResult, MAX_SEARCH_TEXT_BYTES};
use rhwp::native_host::NativeDocumentSession;
use rhwp::DocumentCore;

fn submit(worker: &mut NativeRuntime, command: Command) -> Result<Output, NativeRuntimeFailure> {
    let submitted = worker.try_submit(command).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(completion) = worker.try_recv().unwrap() {
            assert_eq!(completion.request_id, submitted);
            return completion.result;
        }
        assert!(Instant::now() < deadline, "worker deadline");
        thread::sleep(Duration::from_millis(2));
    }
}

fn search_request(query: &str, max_matches: usize) -> NativeSearchRequest {
    NativeSearchRequest {
        query: query.into(),
        case_sensitive: false,
        max_matches,
        range: None,
    }
}

fn search(
    worker: &mut NativeRuntime,
    target: NativeRuntimeTarget,
    query: &str,
    max_matches: usize,
) -> NativeSearchResult {
    let Output::SearchText {
        target: actual,
        result,
    } = submit(
        worker,
        Command::SearchText {
            target,
            request: search_request(query, max_matches),
        },
    )
    .unwrap()
    else {
        panic!("expected search results");
    };
    assert_eq!(actual, target);
    result
}

fn point(offset: u32) -> NativeTextPosition {
    NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: offset,
        cell_path: Vec::new(),
    }
}

fn collapsed(offset: u32) -> NativeTextSelection {
    NativeTextSelection {
        anchor: point(offset),
        focus: point(offset),
    }
}

#[test]
fn worker_search_keeps_source_scalar_offsets_and_replaces_the_explicit_range() {
    let mut worker = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let Output::Info(initial) = submit(&mut worker, Command::New { replace: None }).unwrap() else {
        panic!("expected an open document");
    };
    let initial = initial.target();
    submit(
        &mut worker,
        Command::BeginEdit {
            target: initial,
            selection: collapsed(0),
        },
    )
    .unwrap();
    let Output::Edited(typed) = submit(
        &mut worker,
        Command::Replace {
            target: initial,
            text: "Aİxİx😀".into(),
        },
    )
    .unwrap() else {
        panic!("expected an edit result");
    };
    let target = NativeRuntimeTarget {
        revision: typed.revision,
        ..initial
    };
    let result = search(&mut worker, target, "İX", 1);
    assert!(result.truncated);
    assert_eq!(result.matches.len(), 1);
    let matched = &result.matches[0];
    assert!(matched.editable);
    assert_eq!(matched.selection.anchor, point(1));
    assert_eq!(matched.selection.focus, point(3));
    assert!(!search(&mut worker, target, "İX", 2).truncated);
    assert!(!search(&mut worker, target, "😀", 1).truncated);

    let live_selection = collapsed(0);
    submit(
        &mut worker,
        Command::SetSelection {
            target,
            selection: live_selection.clone(),
        },
    )
    .unwrap();
    let replace = || Command::ReplaceRange {
        target,
        selection: matched.selection.clone(),
        text: "한글".into(),
    };
    let Output::Edited(replaced) = submit(&mut worker, replace()).unwrap() else {
        panic!("expected an edit result");
    };
    assert_eq!(replaced.selection, collapsed(3));
    assert_eq!(
        submit(&mut worker, replace()).unwrap_err().code,
        "staleRevision"
    );
    let current = NativeRuntimeTarget {
        revision: replaced.revision,
        ..target
    };
    let other = search(&mut worker, current, "İX", 2);
    assert_eq!(other.matches.len(), 1);
    assert_eq!(other.matches[0].selection.anchor, point(3));

    let Output::Edited(undone) = submit(&mut worker, Command::Undo { target: current }).unwrap()
    else {
        panic!("expected an undo result");
    };
    assert_eq!(undone.selection, live_selection);
}

#[test]
fn truncated_paragraph_prefixes_cannot_produce_search_hits() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let paragraph = &mut core.document_mut().sections[0].paragraphs[0];
    paragraph.text = "x".repeat(MAX_SEARCH_TEXT_BYTES + 1);
    let session = NativeDocumentSession::from_core(core);
    let result = session.search_text(&search_request("x", 1)).unwrap();
    assert!(result.truncated);
    assert!(result.matches.is_empty());
}

#[test]
fn search_uses_the_editor_protection_policy_for_selectable_cells() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.paste_html_native(
        0,
        0,
        0,
        "<table><tr><td>protected</td><td>editable</td></tr></table>",
    )
    .unwrap();
    let table = core.document_mut().sections[0]
        .paragraphs
        .iter_mut()
        .flat_map(|paragraph| &mut paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("fixture table");
    table.cells[0].set_cell_protect(true);
    let session = NativeDocumentSession::from_core(core);
    let protected = session
        .search_text(&search_request("protected", 2))
        .unwrap();
    assert_eq!(protected.matches.len(), 1);
    assert!(!protected.matches[0].editable);
    let editable = session.search_text(&search_request("editable", 2)).unwrap();
    assert_eq!(editable.matches.len(), 1);
    assert!(editable.matches[0].editable);
}
