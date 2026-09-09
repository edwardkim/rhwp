# Task M100 #6788 — 1단계 코어 구간별 적용 완료보고서

- Issue: [#6788](https://github.com/edwardkim/rhwp/issues/6788)
- 작성일: 2026-09-06
- 상태: **1단계 focused 검증 완료 — 2단계 승인 대기**
- 계획: [수행계획서](../plans/task_m100_6788.md), [구현계획서](../plans/task_m100_6788_impl.md)
- 브랜치: `codex/6788-preserve-mixed-char-format`
- 기준 devel: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`
- 계획 commit: `6b0ee5d55`

## 1. 결과와 한계

코어 속성 적용은 이제 선택 시작의 모양 하나를 전체에 덮어쓰지 않고, 기존 구간마다 지정한
속성만 병합한다. 본문·일반 셀·중첩 셀·머리말/꼬리말에서 형광펜을 적용해도 기존 글자색 및
미지정 글꼴·크기·굵기 등을 보존한다. 선택 밖 모양과 문단 끝 경계도 유지한다.

**Studio Undo/Redo는 아직 수정하지 않았다.** 문단당 before/after ID 하나를 기록하는 문제가
남아 있으므로 Firefox 사용자 증상의 전체 해결이나 배포 가능 상태로 판정하지 않는다.
새 WASM·실제 브라우저/Firefox 검증과 최종 전체 검증도 후속 단계다.

## 2. 변경 내용

| 파일 | 변경 |
| --- | --- |
| `src/model/paragraph.rs` | 원본 ID 수집과 범위 변환이 동일한 문자→UTF-16 경계를 공유한다. `map_char_shape_range`에서 기존 겹침 구간별 ID를 변환하고 범위 밖을 복원한다. |
| `src/model/document.rs` | 원본 ID별 mods 병합 결과를 매핑한다. 같은 원본 ID는 한 번만 처리하고 기존 동일 모양을 재사용한다. |
| `src/document_core/commands/formatting.rs` | 본문·flat 셀·by-path 중첩 셀을 원본 구간별 병합으로 연결한다. |
| `src/document_core/commands/header_footer_ops.rs` | 머리말/꼬리말에도 동일 구간별 병합을 적용한다. |
| `tests/cases/issue_6788_mixed_char_format.rs` | 실제 blank 초기화 및 편집 API 기반 회귀 11개를 추가한다. |

명시적 단일 ID 교체용 `apply_char_shape_range`의 기존 의미는 유지한다. 기존 범위 분할·복원
로직을 공통화하고, 속성 적용 호출만 각 원본 ID를 새 ID로 변환하도록 바꿨다.
빈/유효하지 않은 범위에서는 불필요한 새 글자 모양을 만들지 않는다.

raw stream 무효화·dirty·paint-only 판정·reflow·pagination 후처리는 기존 호출 위치를 유지했다.
구간마다 재조판하지 않으며 renderer/layout 정책과 공개 WASM API는 변경하지 않았다.

## 3. 검증

별도 review worktree `/private/tmp/rhwp-highlight-analysis.KTpzss/devel`에서 파생 suite를 준비했다.
검증 target은 `/Users/melee/Documents/projects/forks/rhwp/target/pr-review`로 고정하고 Cargo를
순차 실행했다. 검증한 원본 4개와 회귀 파일은 작업 브랜치 파일과 `cmp`로 일치 확인했다.
파생 suite·manifest는 커밋 대상이 아니다.

| 검증 | 결과 |
| --- | --- |
| 기준 devel + 새 #6788 테스트 (수정 전) | **1 통과, 10 실패**, ignored 0. 단일 ID 교체 대조군만 통과하여 결함을 검출했다. |
| 1단계 수정 + 동일 #6788 테스트 | **11 통과, 0 실패**, ignored 0. |
| 기존 `document_core::commands::formatting` lib 테스트 | **13 통과**, ignored 0. paint-only·셀 폭·중첩 셀 reflow 포함. |
| 기존 `issue_4121_header_footer_text_selection` | **7 통과**, ignored 0. 선택 범위 서식 등 머리말/꼬리말 계약 유지. |
| review worktree manifest `--check` | 통과. 1170 source, 28 suite + 20 exception = 48 integration target. |
| `cargo fmt --all -- --check` 및 새 test 개별 `rustfmt --check` | 통과. |
| `git diff --check` 및 root/review source 일치 | 통과. |

핵심 실행 명령 (review worktree):

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs --cargo-test issue_6788_mixed_char_format -- \
  --target-dir /Users/melee/Documents/projects/forks/rhwp/target/pr-review
cargo test --locked --lib \
  --target-dir /Users/melee/Documents/projects/forks/rhwp/target/pr-review \
  document_core::commands::formatting
node scripts/run-rust-test.mjs --cargo-test issue_4121_header_footer_text_selection -- \
  --target-dir /Users/melee/Documents/projects/forks/rhwp/target/pr-review
node scripts/rust-test-suite-manifest.mjs --check
```

새 회귀가 확인한 항목:

- 검정/보라/검정에 노랑 형광펜, 반복 적용 시 모양 재사용 및 line segments 유지.
- 혼합 글꼴·크기·굵기·기울임, 굵기/크기 변경 시 기존 색상 유지.
- 부분 선택·단일 구간·선택 밖 보존, 보조 평면 문자와 선행 제어문자의 UTF-16 offset.
- 빈/역전/시작 범위 초과, 끝 범위 초과의 기존 끝까지 적용 의미, 끝 모양 sentinel 보존.
- 본문, 일반 셀 두 진입 경로, 중첩 셀과 바깥 셀 무변경, 머리말·꼬리말.
- 형광펜 적용 후 HWP 및 HWPX 저장·재적재 시 글자색/형광펜 유지.

source-side `#[cfg(test)]`는 변경하지 않았다. 전체 Clippy·전체 integration·Native Skia 및
fresh WASM/Studio/browser 검증은 이 단계에서 실행하지 않았고, 계획의 후속 게이트를 따른다.
기존 진단용 Node WASM은 수정 전 산출물이므로 1단계 수정 검증 증적으로 사용하지 않았다.

## 4. 계획 대비와 다음 승인

1단계 범위 내에서 완료했다. 임시 전체 문서 snapshot이나 단일 ID fallback을 추가하지 않았다.
다음 단계는 선택 범위의 모양 구간 목록 조회/복원 API와 Studio history의 before/after 목록
저장·복원이다. 새 WASM과 실제 `ApplyCharFormatCommand`/`CommandHistory`로 적용→Undo→Redo를
검증해야 한다.

이 보고서와 변경을 로컬 커밋으로 마감하고 **2단계 착수 승인 후** 진행한다.
remote push·PR 생성·merge·이슈 종료는 수행하지 않았다.
