//! 관계형: worker FIFO·입력/출력 한도·취소와 문서 상태·히스토리 보존을 검사한다.
//! 주석 갱신: 2026-11-07T16:41:01+09:00; 직전 파일 커밋: 2026-11-06T12:49:38+09:00 (8a679ed90b1f73aeac67ef384513e44a04b9ef0a).
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
//! Worker ownership/queue/state contracts, not Hancom output or latency goldens.

#![cfg(not(target_arch = "wasm32"))]

use std::thread;
use std::time::{Duration, Instant};

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::native_host::runtime::{
    NativeRequestId, NativeRuntime, NativeRuntimeCancelOutcome, NativeRuntimeCommand as Command,
    NativeRuntimeCompletion, NativeRuntimeConfig, NativeRuntimeError, NativeRuntimeInfo,
    NativeRuntimeOutput as Output, NativeRuntimeRegion, NativeRuntimeSkipReason,
    NativeRuntimeTarget,
};
use rhwp::native_host::save::NativeSaveFormat;
use rhwp::DocumentCore;

fn runtime() -> NativeRuntime {
    NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap()
}
fn next(runtime: &mut NativeRuntime) -> NativeRuntimeCompletion {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(result) = runtime.try_recv().unwrap() {
            return result;
        }
        assert!(Instant::now() < deadline, "worker completion deadline");
        thread::sleep(Duration::from_millis(2));
    }
}
fn run(runtime: &mut NativeRuntime, command: Command) -> Output {
    let id = runtime.try_submit(command).unwrap();
    let completed = next(runtime);
    assert_eq!(completed.request_id, id);
    completed.result.unwrap()
}
fn info(output: Output) -> NativeRuntimeInfo {
    match output {
        Output::Info(info) => info,
        other => panic!("expected info, got {other:?}"),
    }
}
fn new(runtime: &mut NativeRuntime) -> NativeRuntimeInfo {
    info(run(runtime, Command::New { replace: None }))
}
fn selection(offset: u32) -> NativeTextSelection {
    let position = NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: offset,
        cell_path: vec![],
    };
    NativeTextSelection {
        anchor: position.clone(),
        focus: position,
    }
}
fn render(target: NativeRuntimeTarget) -> Command {
    Command::RenderRegion {
        target,
        font_generation: 0,
        page: 0,
        region: NativeRuntimeRegion {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        },
        scale: 1.0,
    }
}

#[test]
fn runtime_input_contract_is_strict_and_debug_redacts_owned_payloads() {
    for config in [
        NativeRuntimeConfig {
            max_outstanding: 0,
            ..Default::default()
        },
        NativeRuntimeConfig {
            max_outstanding: 17,
            ..Default::default()
        },
        NativeRuntimeConfig {
            max_pending_bytes: 0,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            NativeRuntime::spawn(config),
            Err(NativeRuntimeError::InvalidConfig)
        ));
    }
    assert!(serde_json::from_str::<Command>(r#"{"op":"info","unexpected":true}"#).is_err());
    assert!(serde_json::from_str::<Command>(
        r#"{"op":"replace","target":{"documentId":1,"revision":0},"text":"x","unexpected":true}"#
    )
    .is_err());
    assert!(serde_json::from_str::<Command>(
        r#"{"op":"undo","target":{"documentId":1,"revision":-1}}"#
    )
    .is_err());
    assert!(serde_json::from_str::<Command>(r#"{"op":"undo","target":{"documentId":1}}"#).is_err());
    let command = Command::Open {
        replace: None,
        bytes: b"private-document".to_vec(),
        password: Some(b"private-password".to_vec()),
    };
    assert!(!format!("{command:?}").contains("private"));
}

#[test]
fn runtime_handles_move_but_documents_stay_on_workers_and_replacement_rejects_aba() {
    fn require_send<T: Send>() {}
    require_send::<NativeRuntime>();
    let mut first = runtime();
    let mut second = runtime();
    let initial = new(&mut first);
    let independent = new(&mut second);
    assert_ne!(initial.document_id, independent.document_id);
    assert_eq!(initial.revision, 0);
    let replacement = info(run(
        &mut first,
        Command::New {
            replace: Some(initial.target()),
        },
    ));
    assert_ne!(replacement.document_id, initial.document_id);
    assert_eq!(replacement.revision, 0);
    first
        .try_submit(Command::BeginEdit {
            target: initial.target(),
            selection: selection(0),
        })
        .unwrap();
    assert_eq!(next(&mut first).result.unwrap_err().code, "invalidArgument");
    assert_eq!(
        info(run(&mut first, Command::Info {})).document_id,
        replacement.document_id
    );
    first.try_submit(Command::New { replace: None }).unwrap();
    assert_eq!(next(&mut first).result.unwrap_err().code, "invalidArgument");
    assert_eq!(
        info(run(&mut second, Command::Info {})).document_id,
        independent.document_id
    );
}

#[test]
fn completion_credit_and_input_byte_capacity_reject_without_losing_input() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
        max_outstanding: 1,
        max_pending_bytes: 64,
    })
    .unwrap();
    let id = runtime.try_submit(Command::New { replace: None }).unwrap();
    let rejected = runtime.try_submit(Command::Info {}).unwrap_err();
    assert_eq!(rejected.error, NativeRuntimeError::QueueFull);
    assert_eq!(next(&mut runtime).request_id, id);
    assert!(matches!(
        run(&mut runtime, *rejected.command),
        Output::Info(_)
    ));
    let mut bytes = Vec::with_capacity(65);
    bytes.push(42);
    let rejected = runtime
        .try_submit(Command::Open {
            replace: None,
            bytes,
            password: None,
        })
        .unwrap_err();
    assert_eq!(rejected.error, NativeRuntimeError::ByteCapacity);
    let Command::Open { bytes, .. } = *rejected.command else {
        panic!("input ownership lost");
    };
    assert_eq!(bytes, [42]);
}

#[test]
fn failed_open_and_failed_begin_edit_preserve_the_previous_owner() {
    let mut runtime = runtime();
    let original = new(&mut runtime);
    runtime
        .try_submit(Command::Open {
            replace: Some(original.target()),
            bytes: b"not-a-document".to_vec(),
            password: None,
        })
        .unwrap();
    assert!(next(&mut runtime).result.is_err());
    assert_eq!(
        info(run(&mut runtime, Command::Info {})).document_id,
        original.document_id
    );
    runtime
        .try_submit(Command::BeginEdit {
            target: original.target(),
            selection: selection(100),
        })
        .unwrap();
    assert!(next(&mut runtime).result.is_err());
    let unchanged = info(run(&mut runtime, Command::Info {}));
    assert_eq!(unchanged.target(), original.target());
    assert!(!unchanged.editing);
}

#[test]
fn edits_and_save_results_remain_fifo_and_late_ack_does_not_clean_newer_edits() {
    let mut runtime = runtime();
    let initial = new(&mut runtime);
    run(
        &mut runtime,
        Command::BeginEdit {
            target: initial.target(),
            selection: selection(0),
        },
    );
    let edit = runtime
        .try_submit(Command::Replace {
            target: initial.target(),
            text: "가😀".into(),
        })
        .unwrap();
    let target = NativeRuntimeTarget {
        revision: 1,
        ..initial.target()
    };
    let query = runtime.try_submit(Command::Info {}).unwrap();
    assert_eq!(next(&mut runtime).request_id, edit);
    let queried = next(&mut runtime);
    assert_eq!(queried.request_id, query);
    assert_eq!(
        info(queried.result.unwrap())
            .selection
            .unwrap()
            .focus
            .scalar_offset,
        2
    );
    let Output::PreparedSave {
        document_id,
        prepared,
    } = run(
        &mut runtime,
        Command::PrepareSave {
            target,
            format: NativeSaveFormat::Hwp,
        },
    )
    else {
        panic!("save");
    };
    let reopened = DocumentCore::from_bytes(&prepared.bytes).unwrap();
    assert_eq!(reopened.document().sections[0].paragraphs[0].text, "가😀");
    assert_eq!(prepared.revision, 1);
    assert!(prepared.caret_saved);
    run(
        &mut runtime,
        Command::Replace {
            target,
            text: "나".into(),
        },
    );
    let Output::SaveStatus { status, .. } = run(
        &mut runtime,
        Command::AcknowledgeSave {
            document_id,
            token: prepared.token,
        },
    ) else {
        panic!("ack");
    };
    assert_eq!(status.saved_revision, Some(1));
    assert_eq!(status.revision, 2);
    assert!(status.dirty);
    let current = info(run(&mut runtime, Command::Info {}));
    assert!(current.can_undo);
    run(
        &mut runtime,
        Command::Undo {
            target: current.target(),
        },
    );
    assert!(info(run(&mut runtime, Command::Info {})).can_redo);
}

#[test]
fn ime_identity_and_save_blocking_use_the_existing_editor_contract() {
    let mut runtime = runtime();
    let initial = new(&mut runtime);
    run(
        &mut runtime,
        Command::BeginEdit {
            target: initial.target(),
            selection: selection(0),
        },
    );
    let Output::Edited(started) = run(
        &mut runtime,
        Command::BeginComposition {
            target: initial.target(),
        },
    ) else {
        panic!("composition");
    };
    let composition_id = started.composition_id.unwrap();
    let Output::Edited(updated) = run(
        &mut runtime,
        Command::UpdateComposition {
            target: initial.target(),
            composition_id,
            text: "가".into(),
        },
    ) else {
        panic!("preview");
    };
    let target = NativeRuntimeTarget {
        revision: updated.revision,
        ..initial.target()
    };
    runtime
        .try_submit(Command::PrepareSave {
            target,
            format: NativeSaveFormat::Hwp,
        })
        .unwrap();
    assert_eq!(
        next(&mut runtime).result.unwrap_err().code,
        "invalidArgument"
    );
    run(
        &mut runtime,
        Command::CommitComposition {
            target,
            composition_id,
        },
    );
    assert!(info(run(&mut runtime, Command::Info {})).can_undo);
    runtime
        .try_submit(Command::UpdateComposition {
            target,
            composition_id,
            text: "late".into(),
        })
        .unwrap();
    assert!(next(&mut runtime).result.is_err());
}

#[test]
fn only_successfully_accepted_render_requests_supersede_prior_results() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
        max_outstanding: 1,
        ..Default::default()
    })
    .unwrap();
    let target = new(&mut runtime).target();
    let accepted = runtime.try_submit(render(target)).unwrap();
    assert_eq!(
        runtime.try_submit(render(target)).unwrap_err().error,
        NativeRuntimeError::QueueFull
    );
    let completion = next(&mut runtime);
    assert_eq!(completion.request_id, accepted);
    #[cfg(feature = "native-skia")]
    assert!(matches!(completion.result.unwrap(), Output::Frame { .. }));
    #[cfg(not(feature = "native-skia"))]
    assert_eq!(completion.result.unwrap_err().code, "featureDisabled");
}

#[test]
fn newer_render_suppresses_even_an_already_queued_older_render_result() {
    let mut runtime = runtime();
    let target = new(&mut runtime).target();
    let older = runtime.try_submit(render(target)).unwrap();
    let newer = runtime.try_submit(render(target)).unwrap();
    let first = next(&mut runtime);
    assert_eq!(first.request_id, older);
    assert!(matches!(
        first.result.unwrap(),
        Output::Skipped(NativeRuntimeSkipReason::SupersededRender)
    ));
    let second = next(&mut runtime);
    assert_eq!(second.request_id, newer);
    #[cfg(feature = "native-skia")]
    assert!(matches!(second.result.unwrap(), Output::Frame { .. }));
    #[cfg(not(feature = "native-skia"))]
    assert_eq!(second.result.unwrap_err().code, "featureDisabled");
}

#[test]
fn cancellation_is_a_claim_race_not_a_promise_to_interrupt_native_calls() {
    let mut runtime = runtime();
    let initial = new(&mut runtime);
    let id = runtime
        .try_submit(Command::New {
            replace: Some(initial.target()),
        })
        .unwrap();
    let cancellation = runtime.cancel(id);
    let completion = next(&mut runtime);
    assert_eq!(completion.request_id, id);
    match cancellation {
        NativeRuntimeCancelOutcome::Cancelled => {
            assert!(matches!(completion.result.unwrap(), Output::Cancelled))
        }
        NativeRuntimeCancelOutcome::TooLate => {
            assert!(matches!(completion.result.unwrap(), Output::Info(_)))
        }
        NativeRuntimeCancelOutcome::Unknown => {
            panic!("accepted request disappeared before receipt")
        }
    }
    assert_eq!(runtime.cancel(id), NativeRuntimeCancelOutcome::Unknown);
    assert_eq!(
        runtime.cancel(NativeRequestId::from_raw(0)),
        NativeRuntimeCancelOutcome::Unknown
    );
}

#[test]
fn typed_geometry_queries_reuse_core_and_reject_stale_revision() {
    let mut runtime = runtime();
    let initial = new(&mut runtime);
    run(
        &mut runtime,
        Command::BeginEdit {
            target: initial.target(),
            selection: selection(0),
        },
    );
    let Output::Edited(edited) = run(
        &mut runtime,
        Command::Replace {
            target: initial.target(),
            text: "A😀B".into(),
        },
    ) else {
        panic!("edit");
    };
    let target = NativeRuntimeTarget {
        revision: edited.revision,
        ..initial.target()
    };
    assert!(
        matches!(run(&mut runtime, Command::PageSize { target, page: 0 }), Output::PageSize { size, .. } if size.width > 0.0 && size.height > 0.0)
    );
    let Output::CaretRect {
        caret: Some(caret), ..
    } = run(
        &mut runtime,
        Command::CaretRect {
            target,
            position: selection(1).focus,
        },
    )
    else {
        panic!("caret");
    };
    assert!(matches!(
        run(
            &mut runtime,
            Command::HitTest {
                target,
                page: caret.page,
                x: caret.x,
                y: caret.y + caret.height / 2.0
            }
        ),
        Output::HitTest { .. }
    ));
    let mut selected = selection(0);
    selected.focus.scalar_offset = 3;
    assert!(
        matches!(run(&mut runtime, Command::SelectionRects { target, selection: selected }), Output::SelectionRects { rects, .. } if !rects.is_empty())
    );
    runtime
        .try_submit(Command::PageSize {
            target: initial.target(),
            page: 0,
        })
        .unwrap();
    assert_eq!(next(&mut runtime).result.unwrap_err().code, "staleRevision");
}

#[test]
fn close_with_unread_completion_and_drop_never_wait_for_completion_capacity() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
        max_outstanding: 1,
        ..Default::default()
    })
    .unwrap();
    runtime.try_submit(Command::New { replace: None }).unwrap();
    runtime.close();
    runtime.close();
    assert_eq!(
        runtime.try_submit(Command::Info {}).unwrap_err().error,
        NativeRuntimeError::Closed
    );
    assert!(matches!(
        runtime.try_recv(),
        Err(NativeRuntimeError::Closed)
    ));
    let deadline = Instant::now() + Duration::from_secs(30);
    while !runtime.is_finished() {
        assert!(Instant::now() < deadline, "closed worker failed to exit");
        thread::sleep(Duration::from_millis(2));
    }
    assert!(runtime.try_join().unwrap());
    let (done, received) = std::sync::mpsc::sync_channel(1);
    let join = thread::spawn(move || {
        let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
            max_outstanding: 1,
            ..Default::default()
        })
        .unwrap();
        runtime.try_submit(Command::New { replace: None }).unwrap();
        drop(runtime);
        done.send(()).unwrap();
    });
    received.recv_timeout(Duration::from_secs(30)).unwrap();
    join.join().unwrap();
}

#[cfg(feature = "native-skia")]
#[test]
fn completed_frame_is_suppressed_after_a_document_replacement_finishes() {
    let mut runtime = runtime();
    let initial = new(&mut runtime);
    let frame = runtime.try_submit(render(initial.target())).unwrap();
    let replaced = runtime
        .try_submit(Command::New {
            replace: Some(initial.target()),
        })
        .unwrap();
    let barrier = runtime.try_submit(Command::Info {}).unwrap();
    // Claim/completion of this harmless trailing query proves replacement has
    // finished, without consuming the earlier frame or relying on a sleep race.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match runtime.cancel(barrier) {
            NativeRuntimeCancelOutcome::TooLate => break,
            NativeRuntimeCancelOutcome::Cancelled => (),
            NativeRuntimeCancelOutcome::Unknown => panic!("unread barrier disappeared"),
        }
        assert!(Instant::now() < deadline, "worker barrier deadline");
        thread::sleep(Duration::from_millis(2));
    }
    let first = next(&mut runtime);
    assert_eq!(first.request_id, frame);
    assert!(matches!(
        first.result.unwrap(),
        Output::Skipped(NativeRuntimeSkipReason::StaleFrame)
    ));
    let second = next(&mut runtime);
    assert_eq!(second.request_id, replaced);
    assert_ne!(
        info(second.result.unwrap()).document_id,
        initial.document_id
    );
    assert_eq!(next(&mut runtime).request_id, barrier);
}

fn tiled(target: NativeRuntimeTarget) -> Command {
    Command::RenderTiledRegion {
        target,
        font_generation: 0,
        page: 0,
        region: NativeRuntimeRegion {
            x: -0.1,
            y: 1.25,
            width: 1.2,
            height: 2.5,
        },
        scale: 8.0,
    }
}

#[test]
fn tiled_runtime_wire_preserves_target_and_rejects_unknown_fields() {
    let target = NativeRuntimeTarget {
        document_id: NativeRequestId::from_raw(17),
        revision: 4,
    };
    let encoded = serde_json::to_value(tiled(target)).unwrap();
    assert_eq!(encoded["op"], "renderTiledRegion");
    assert_eq!(encoded["target"]["revision"], 4);
    let Command::RenderTiledRegion { region, scale, .. } =
        serde_json::from_value(encoded.clone()).unwrap()
    else {
        panic!("wrong command");
    };
    assert_eq!(region.x, -0.1);
    assert_eq!(scale, 8.0);
    let mut invalid = encoded;
    invalid["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Command>(invalid).is_err());
}

#[test]
fn tiled_runtime_checks_revision_and_font_generation_before_rendering() {
    let mut runtime = runtime();
    let target = new(&mut runtime).target();
    let mut command = tiled(target);
    if let Command::RenderTiledRegion {
        font_generation, ..
    } = &mut command
    {
        *font_generation = 1;
    }
    runtime.try_submit(command).unwrap();
    assert_eq!(
        next(&mut runtime).result.unwrap_err().code,
        "invalidArgument"
    );
    runtime
        .try_submit(tiled(NativeRuntimeTarget {
            revision: target.revision + 1,
            ..target
        }))
        .unwrap();
    assert_eq!(next(&mut runtime).result.unwrap_err().code, "staleRevision");
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn tiled_runtime_and_raster_cache_controls_report_feature_disabled() {
    let mut runtime = runtime();
    let target = new(&mut runtime).target();
    for command in [
        tiled(target),
        Command::RasterCacheStats { target },
        Command::SetRasterCacheLimit {
            target,
            max_bytes: 0,
        },
    ] {
        runtime.try_submit(command).unwrap();
        assert_eq!(
            next(&mut runtime).result.unwrap_err().code,
            "featureDisabled"
        );
    }
}

fn viewport_command(
    target: NativeRuntimeTarget,
    region: NativeRuntimeRegion,
    scale: f64,
    use_tiles: bool,
) -> Command {
    if use_tiles {
        Command::RenderTiledRegion {
            target,
            font_generation: 0,
            page: 0,
            region,
            scale,
        }
    } else {
        Command::RenderRegion {
            target,
            font_generation: 0,
            page: 0,
            region,
            scale,
        }
    }
}

#[test]
fn runtime_output_reservations_match_enclosing_rgba_and_hold_until_receipt() {
    // floor(-0.3*1.25)..ceil(3.1*1.25) is [-1,4), and the
    // vertical interval is [0,4): exactly 5*4*4 = 80 RGBA bytes.
    let region = NativeRuntimeRegion {
        x: -0.3,
        y: 0.2,
        width: 3.4,
        height: 2.9,
    };
    for use_tiles in [false, true] {
        let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
            max_outstanding: 4,
            max_pending_bytes: 80,
        })
        .unwrap();
        let target = new(&mut runtime).target();
        let first = runtime
            .try_submit(viewport_command(target, region, 1.25, use_tiles))
            .unwrap();
        assert_eq!(
            runtime
                .try_submit(viewport_command(target, region, 1.25, use_tiles))
                .unwrap_err()
                .error,
            NativeRuntimeError::ByteCapacity
        );
        let completion = next(&mut runtime);
        assert_eq!(completion.request_id, first);
        #[cfg(feature = "native-skia")]
        {
            let Output::Frame { frame, .. } = completion.result.unwrap() else {
                panic!("rejected second request must not supersede the first");
            };
            assert_eq!((frame.origin_x, frame.origin_y), (-1, 0));
            assert_eq!((frame.pixels.width, frame.pixels.height), (5, 4));
            assert_eq!((frame.pixels.stride, frame.pixels.bytes.len()), (20, 80));
        }
        #[cfg(not(feature = "native-skia"))]
        assert_eq!(completion.result.unwrap_err().code, "featureDisabled");
        let after_receipt = runtime
            .try_submit(viewport_command(target, region, 1.25, use_tiles))
            .unwrap();
        assert_eq!(next(&mut runtime).request_id, after_receipt);
    }
}

#[test]
fn many_small_viewports_use_real_bytes_while_completion_credits_stay_bounded() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
        max_outstanding: 4,
        max_pending_bytes: 4 * 8 * 8 * 4,
    })
    .unwrap();
    let target = new(&mut runtime).target();
    let mut accepted = Vec::new();
    for _ in 0..4 {
        accepted.push(runtime.try_submit(render(target)).unwrap());
    }
    assert_eq!(
        runtime.try_submit(render(target)).unwrap_err().error,
        NativeRuntimeError::QueueFull
    );
    for (index, id) in accepted.into_iter().enumerate() {
        let completion = next(&mut runtime);
        assert_eq!(completion.request_id, id);
        if index < 3 {
            assert!(matches!(
                completion.result.unwrap(),
                Output::Skipped(NativeRuntimeSkipReason::SupersededRender)
            ));
        } else {
            #[cfg(feature = "native-skia")]
            assert!(matches!(completion.result.unwrap(), Output::Frame { .. }));
            #[cfg(not(feature = "native-skia"))]
            assert_eq!(completion.result.unwrap_err().code, "featureDisabled");
        }
    }
}

#[test]
fn invalid_render_geometry_is_rejected_before_superseding_a_valid_request() {
    let region = NativeRuntimeRegion {
        x: 0.0,
        y: 0.0,
        width: 8.0,
        height: 8.0,
    };
    for use_tiles in [false, true] {
        let mut runtime = runtime();
        let target = new(&mut runtime).target();
        let first = runtime.try_submit(render(target)).unwrap();
        for (invalid, scale) in [
            (
                NativeRuntimeRegion {
                    x: f64::NAN,
                    ..region
                },
                1.0,
            ),
            (
                NativeRuntimeRegion {
                    x: f64::INFINITY,
                    ..region
                },
                1.0,
            ),
            (
                NativeRuntimeRegion {
                    width: -1.0,
                    ..region
                },
                1.0,
            ),
            (
                NativeRuntimeRegion {
                    height: 0.0,
                    ..region
                },
                1.0,
            ),
            (
                NativeRuntimeRegion {
                    x: f64::MAX,
                    ..region
                },
                1.0,
            ),
            (
                NativeRuntimeRegion {
                    width: 16_385.0,
                    ..region
                },
                1.0,
            ),
            (
                NativeRuntimeRegion {
                    width: 4097.0,
                    height: 4096.0,
                    ..region
                },
                1.0,
            ),
            (region, 0.0),
            (region, -1.0),
            (region, 64.1),
            (region, f64::NAN),
        ] {
            assert_eq!(
                runtime
                    .try_submit(viewport_command(target, invalid, scale, use_tiles))
                    .unwrap_err()
                    .error,
                NativeRuntimeError::InvalidArgument
            );
        }
        let completion = next(&mut runtime);
        assert_eq!(completion.request_id, first);
        #[cfg(feature = "native-skia")]
        assert!(matches!(completion.result.unwrap(), Output::Frame { .. }));
        #[cfg(not(feature = "native-skia"))]
        assert_eq!(completion.result.unwrap_err().code, "featureDisabled");
    }
}
