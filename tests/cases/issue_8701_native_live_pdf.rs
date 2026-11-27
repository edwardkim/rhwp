//! Checks the bound, revision and read-only behavior of Native worker PDF export.

#![cfg(not(target_arch = "wasm32"))]

use std::thread;
use std::time::{Duration, Instant};

use rhwp::native_host::runtime::{
    NativeRuntime, NativeRuntimeCommand as Command, NativeRuntimeCompletion, NativeRuntimeConfig,
    NativeRuntimeError, NativeRuntimeInfo, NativeRuntimeOutput as Output, NativeRuntimeTarget,
    MAX_LIVE_PDF_BYTES, MAX_LIVE_PDF_PAGES,
};

fn complete(runtime: &mut NativeRuntime, command: Command) -> NativeRuntimeCompletion {
    let request = runtime.try_submit(command).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(completion) = runtime.try_recv().unwrap() {
            assert_eq!(completion.request_id, request);
            return completion;
        }
        assert!(Instant::now() < deadline, "worker completion deadline");
        thread::sleep(Duration::from_millis(2));
    }
}

fn info(runtime: &mut NativeRuntime, command: Command) -> NativeRuntimeInfo {
    let Output::Info(info) = complete(runtime, command).result.unwrap() else {
        panic!("expected worker document info");
    };
    info
}

fn export(
    target: NativeRuntimeTarget,
    first_page: u32,
    page_count: u32,
    max_output_bytes: usize,
) -> Command {
    Command::ExportPdf {
        target,
        font_generation: 0,
        first_page,
        page_count,
        max_output_bytes,
    }
}

#[test]
fn invalid_pdf_range_and_byte_bounds_are_rejected_at_submission() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let current = info(&mut runtime, Command::New { replace: None }).target();
    for command in [
        export(current, 0, 0, 4096),
        export(current, 0, MAX_LIVE_PDF_PAGES + 1, 4096),
        export(current, u32::MAX, 1, 4096),
        export(current, 0, 1, 0),
        export(current, 0, 1, MAX_LIVE_PDF_BYTES + 1),
    ] {
        assert_eq!(
            runtime.try_submit(command).unwrap_err().error,
            NativeRuntimeError::InvalidArgument
        );
    }
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn pdf_command_reports_feature_disabled_on_a_valid_request() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let current = info(&mut runtime, Command::New { replace: None }).target();
    let completed = complete(&mut runtime, export(current, 0, 1, 4096));
    assert_eq!(completed.result.unwrap_err().code, "featureDisabled");
}

#[cfg(feature = "native-skia")]
#[test]
fn selected_pdf_uses_unsaved_revision_and_does_not_mutate_the_document() {
    use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
    use rhwp::DocumentCore;

    let mut source = DocumentCore::new_empty();
    source.create_blank_document_native().unwrap();
    let first_text = "FirstPageMarker";
    source.insert_text_native(0, 0, 0, first_text).unwrap();
    source
        .insert_page_break_native(0, 0, first_text.len())
        .unwrap();
    source.insert_text_native(0, 1, 0, "SecondPage").unwrap();
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let opened = info(
        &mut runtime,
        Command::Open {
            replace: None,
            bytes: source.export_hwpx_native().unwrap(),
            password: None,
        },
    );
    assert_eq!(opened.page_count, 2);
    let original = opened.target();
    let second_page_start = NativeTextPosition {
        section: 0,
        paragraph: 1,
        scalar_offset: 0,
        cell_path: Vec::new(),
    };
    info(
        &mut runtime,
        Command::BeginEdit {
            target: original,
            selection: NativeTextSelection {
                anchor: second_page_start.clone(),
                focus: second_page_start,
            },
        },
    );
    let marker = "UnsavedPdfMarker";
    let edited = complete(
        &mut runtime,
        Command::Replace {
            target: original,
            text: marker.into(),
        },
    );
    assert!(matches!(edited.result, Ok(Output::Edited(_))));
    let live = info(&mut runtime, Command::Info {});
    let current = live.target();
    assert!(live.dirty && live.can_undo);
    assert_eq!(
        complete(&mut runtime, export(original, 1, 1, 1024 * 1024))
            .result
            .unwrap_err()
            .code,
        "staleRevision"
    );
    let too_small = complete(&mut runtime, export(current, 1, 1, 1));
    assert_eq!(too_small.result.unwrap_err().code, "limitExceeded");
    let outside = complete(&mut runtime, export(current, 2, 1, 1024 * 1024));
    assert_eq!(outside.result.unwrap_err().code, "invalidArgument");

    let Output::ExportedPdf {
        target: printed,
        font_generation,
        first_page,
        page_count,
        bytes,
    } = complete(&mut runtime, export(current, 1, 1, 1024 * 1024))
        .result
        .unwrap()
    else {
        panic!("expected PDF bytes");
    };
    assert_eq!(printed, current);
    assert_eq!((font_generation, first_page, page_count), (0, 1, 1));
    assert!(bytes.starts_with(b"%PDF-"));
    assert!(bytes.windows(5).any(|part| part == b"%%EOF"));
    #[cfg(feature = "pdf-import")]
    {
        let document =
            pdf_oxide::PdfDocument::from_bytes_with_limits(bytes.into_vec(), Default::default())
                .unwrap();
        assert_eq!(document.page_count().unwrap(), 1);
        let text: String = document
            .extract_text(0)
            .unwrap()
            .split_whitespace()
            .collect();
        assert!(text.contains(marker), "{text}");
        assert!(!text.contains(first_text), "{text}");
    }
    assert_eq!(info(&mut runtime, Command::Info {}), live);

    let started = complete(&mut runtime, Command::BeginComposition { target: current });
    assert!(matches!(started.result, Ok(Output::Edited(_))));
    let composing = info(&mut runtime, Command::Info {});
    assert!(composing.composition_id.is_some());
    let blocked = complete(&mut runtime, export(composing.target(), 1, 1, 1024 * 1024));
    assert_eq!(blocked.result.unwrap_err().code, "unsupported");
    assert_eq!(info(&mut runtime, Command::Info {}), composing);
}
