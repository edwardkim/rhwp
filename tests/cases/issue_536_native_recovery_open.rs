//! 관계형: recovery 가져오기·worker 문서 교체와 미저장 checkpoint의 보존을 검사한다.
//! 주석 갱신: 2026-11-07T16:41:01+09:00; 직전 파일 커밋: 2026-11-07T07:40:46+09:00 (21fac65baabdb550e597e96ea676d913de054304).
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
//! Explicit recovery imports retain an unsaved checkpoint, including worker replacement.

#![cfg(not(target_arch = "wasm32"))]

use std::thread;
use std::time::{Duration, Instant};

use rhwp::native_host::runtime::{
    NativeRuntime, NativeRuntimeCommand as Command, NativeRuntimeConfig, NativeRuntimeInfo,
    NativeRuntimeOutput as Output,
};
use rhwp::native_host::save::NativeSaveFormat;
use rhwp::native_host::NativeDocumentSession;
use rhwp::DocumentCore;

fn payload() -> Vec<u8> {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "복구 😀").unwrap();
    core.export_hwpx_native().unwrap()
}

fn run(runtime: &mut NativeRuntime, command: Command) -> Result<Output, String> {
    let request = runtime.try_submit(command).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(completion) = runtime.try_recv().unwrap() {
            assert_eq!(completion.request_id, request);
            return completion.result.map_err(|error| error.message);
        }
        assert!(Instant::now() < deadline, "recovery worker deadline");
        thread::sleep(Duration::from_millis(2));
    }
}

fn info(output: Output) -> NativeRuntimeInfo {
    match output {
        Output::Info(info) => info,
        other => panic!("expected document info, got {other:?}"),
    }
}

#[test]
fn recovery_import_starts_dirty_without_manufacturing_an_edit() {
    let bytes = payload();
    assert!(!NativeDocumentSession::open_bytes(&bytes, None)
        .unwrap()
        .is_dirty());
    let mut restored = NativeDocumentSession::open_recovery_bytes(&bytes).unwrap();
    assert_eq!(restored.revision(), 0);
    assert!(restored.is_dirty());
    let prepared = restored.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    let cancelled = restored.cancel_save(prepared.token()).unwrap();
    assert_eq!(cancelled.saved_revision, None);
    assert!(cancelled.dirty);
    let prepared = restored.prepare_save(NativeSaveFormat::Hwpx).unwrap();
    let saved = restored.acknowledge_save(prepared.token()).unwrap();
    assert_eq!(saved.saved_revision, Some(0));
    assert!(!saved.dirty);
}

#[test]
fn failed_and_stale_recovery_replacements_preserve_the_existing_document() {
    let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap();
    let original = info(run(&mut runtime, Command::New { replace: None }).unwrap());
    assert!(run(
        &mut runtime,
        Command::OpenRecovery {
            replace: None,
            bytes: payload()
        }
    )
    .is_err());
    assert_eq!(info(run(&mut runtime, Command::Info {}).unwrap()), original);
    assert!(run(
        &mut runtime,
        Command::OpenRecovery {
            replace: Some(original.target()),
            bytes: b"corrupt recovery".to_vec(),
        }
    )
    .is_err());
    assert_eq!(info(run(&mut runtime, Command::Info {}).unwrap()), original);
    let restored = info(
        run(
            &mut runtime,
            Command::OpenRecovery {
                replace: Some(original.target()),
                bytes: payload(),
            },
        )
        .unwrap(),
    );
    assert_ne!(restored.document_id, original.document_id);
    assert_eq!(restored.revision, 0);
    assert!(restored.dirty);
    assert!(!restored.editing);
    assert!(!restored.can_undo);
    assert!(run(
        &mut runtime,
        Command::OpenRecovery {
            replace: Some(original.target()),
            bytes: payload(),
        }
    )
    .is_err());
    assert_eq!(info(run(&mut runtime, Command::Info {}).unwrap()), restored);
}
