---
kind: review
status: completed
pr: 4263
issue: 2550
last_verified: 2026-08-08
---

# PR #4263 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4263](https://github.com/edwardkim/rhwp/pull/4263) |
| 작성자 | `kevin9327` |
| 제목 | `fix(security): BinData 압축 해제 상한 적용 — 저장·클립보드·렌더 deflate bomb 방어 (#2550)` |
| base / 원 candidate | `devel` / `6e5ee415679a440ca0ba59ff4b594a6ed2feab16` |
| 최종 로컬 head | `c4bfc73e2` (`6e5ee415` + 메인터너 보정 + 최신 `devel` merge) |
| 작성 시점 최신 devel | `59b31e5ce1d29aa6300e777bfa682ee123fa4f98` |
| 규모 | 18 파일, +691 / -52, 1 commit |
| 최종 병합 상태 | 최신 `devel`을 반영한 뒤 `src/renderer/layout.rs` import 충돌 해소 |
| 원격 CI | 원격 source head는 아직 원 candidate다. 최종 보정 head는 push 승인 전이므로 GitHub CI 미실행 |

검토 라우팅은 `maintainer 일반`이며, 보조 경로는 `접수·리뷰 기록`, `로컬 검증`,
`재작업·예외`다. 읽은 문서는 `pr_review_workflow.md`, `pr_review/README.md`,
`maintainer_general.md`, `intake_and_review.md`, `local_validation.md`,
`rework_and_exceptions.md`다.

## 변경 의도와 범위

PR은 HWP5 BinData의 지연 압축 해제가 저장·렌더·클립보드·질의 중 deflate bomb을
전량 materialize하는 문제를 막기 위해 `MAX_BIN_DATA_BYTES = 256MB` 상한과 bounded
load 경로를 추가한다. HWP5 저장은 원본 compressed stream을 보존하고, HWPX 저장과
렌더·클립보드·질의는 placeholder 또는 오류로 접는 계약을 제안한다.

변경 범위에는 parser/resolver, `BinDataBytes`, HWP/HWPX serializer, renderer,
clipboard, DocLang adapter와 신규 합성 bomb 회귀 시험이 포함된다. renderer와
serializer 경로를 바꾸지만, 정상 문서의 시각 fidelity를 바꾸는 PR은 아니므로
기준 PDF visual sweep은 판단 근거로 사용하지 않았다. 상한 초과 항목이 안전하게
placeholder가 되는지의 포맷별 회귀 시험이 필요한 범위다.

## 로컬 검증

검토 브랜치 `review/kevin9327-20260808`에서 candidate head를 고정했다. 최신 `devel`은
candidate의 조상이 아니며, 로컬 `merge-tree`에서 clipboard, rendering, parser,
serializer 및 renderer layout 파일의 충돌을 확인했다. source branch를 억지로
rebase하거나 merge하지 않았다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `git diff --check d3fb9de7..6e5ee415` | 통과 |
| `CARGO_TARGET_DIR=target/review-kevin9327-20260808 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2550_bin_data_decompression_bomb -- --nocapture` | 5/5 통과. HWP5 CFB 합성 bomb의 parse, HWP5 저장, render/query, clipboard, 정상 HWP 왕복만 검증한다. |
| `CARGO_TARGET_DIR=target/review-kevin9327-20260808 CARGO_INCREMENTAL=0 cargo test --profile release-test --lib parser::hwpx::reader::tests::test_compressed_entry_limited_read_rejects_before_materialization -- --nocapture` | 1/1 통과. HWPX reader의 bounded ZIP entry primitive만 검증한다. |

전체 `release-test`와 Native Skia는 초기 candidate 단계에서는 실행하지 않았다. candidate가 최신
`devel`과 충돌하고 아래의 보안 차단 결함이 먼저 해소되어야 했기 때문이다.

## 발견 사항

### P1: HWPX -> HWP 저장에서 상한 실패가 무제한 해제로 되돌아간다

`src/serializer/cfb_writer.rs:237-261`은 `load_limited(MAX_BIN_DATA_BYTES)`가 `None`이면
`load_raw()`를 시도하고, 그것도 `None`이면 `content.data.load()`를 호출한다.

그러나 HWPX lazy resolver는 `src/parser/hwpx/mod.rs:88-130`에서 `resolve_limited()`만
구현한다. ZIP 원본 stream을 HWP5 CFB stream으로 그대로 통과시킬 수 없으므로
`resolve_raw()`는 trait 기본값 `None`이다. 따라서 256MB 초과 HWPX BinData는 다음 경로를
탄다.

```text
HWPX BinData bomb
  -> load_limited(256MB) = None
  -> load_raw() = None
  -> content.data.load()
  -> HwpxReader::read_file_bytes()가 ZIP entry 전체 압축 해제
```

즉 HWPX를 HWP로 저장하는 지원 경로에서 OOM 방어가 사라진다. `None`은 이미 메모리에
존재하는 안전한 `Loaded` 값뿐 아니라 HWPX lazy container도 뜻하므로, 현재 fallback의
전제 자체가 성립하지 않는다. 이 PR의 목적이 저장 경로 deflate bomb 방어이므로 병합을
막는 결함이다.

### P1: HWPX DocLang/외부 이미지 질의가 bounded load 전에 무제한 해제를 호출한다

`src/doclang/adapter/resources.rs:53-62`는 bounded `load_limited()` 전에
`!c.data.is_empty()`로 필터한다. `BinDataResolver`의 기본 `resolved_is_empty()`는
`resolve(key).is_empty()`를 호출한다(`src/model/bin_data.rs:135-143`). HWPX resolver는
이 메서드를 재정의하지 않았으므로 ZIP entry 전체가 먼저 해제된다.

같은 기본 경로는 `Document::external_image_loaded()`의
`src/model/document.rs:320-332`에도 남아 있다. HWP5 resolver에만 추가된
`resolved_len()`/`resolved_is_empty()`는 HWPX에는 적용되지 않는다. 따라서 HWPX bomb은
DocLang 이미지 질의나 외부 이미지 상태 확인에서도 상한을 우회할 수 있다.

## 메인터너 보정

1. `src/serializer/cfb_writer.rs`에서 `load_limited()` 실패 뒤 무제한 `load()`로 되돌아가던
   fallback을 제거했다. HWP5 raw stream은 저장 형식이 일치할 때만 통과시키고, 원 raw 표현이 없는
   HWPX lazy BinData와 상한 초과 값은 빈 placeholder stream으로 저장한다.
2. `HwpxReader::file_size_limited()`와 `HwpxBinResolver`의 `resolved_len()`/
   `resolved_is_empty()`를 추가했다. ZIP central directory의 비압축 길이를 상한과 비교하므로
   DocLang 및 외부 이미지 상태 질의가 entry 전체를 해제하지 않는다.
3. `src/doclang/adapter/resources.rs`의 사전 `is_empty()` 호출을 제거하고,
   `load_limited(MAX_BIN_DATA_BYTES)` 결과만으로 이미지 bytes를 만들게 했다.
4. 실제 HWPX ZIP entry에 256MB 초과의 압축 가능한 zero payload를 기록하는 회귀 시험을 추가했다.
   query, HWPX -> HWP, HWPX -> HWPX 모두 `None` 또는 빈 placeholder로 끝나는지를 검증한다.
5. 최신 `devel` 반영 시 `src/renderer/layout.rs`의 import 충돌은 contributor의
   `find_bin_data_bytes`와 devel의 outline-numbering imports를 모두 보존해 해소했다. 동작 로직을
   임의로 선택하거나 삭제하지 않았다.

## 최종 로컬 검증

모든 명령은 `CARGO_TARGET_DIR=target/review-kevin9327-20260808`,
`CARGO_INCREMENTAL=0`에서 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo test --profile release-test --test issue_2550_bin_data_decompression_bomb -- --nocapture` | 6/6 통과. HWP5와 실제 ZIP 형식 HWPX bomb의 query·두 저장 경로를 포함한다. |
| `cargo test --profile release-test --lib parser::hwpx::reader::tests::test_compressed_entry_limited_read_rejects_before_materialization -- --nocapture` | 1/1 통과 |
| `cargo test --profile release-test --lib doclang::adapter::resources::tests -- --nocapture` | 8/8 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --profile release-test --tests` | 종료 코드 0. 전체 integration suite 통과. 느린 기존 baseline인 `overflow_cell_lines_do_not_grow`도 280.98초에 통과했다. |

Native Skia는 이번 변경이 BinData 보안 경계와 serializer/resolver에 한정되고, Rust 전체
release-test와 Clippy가 최종 merge head에서 통과했으므로 중복 실행하지 않았다. 원격 source branch에
최종 head를 push한 뒤에는 해당 head의 GitHub CI 상태와 mergeability를 다시 확인해야 한다.

## 결론

**로컬 수용.** 초기 P1 두 건을 메인터너 보정으로 차단했고, 실제 ZIP 형식 HWPX bomb을 포함한
회귀와 전체 release-test가 최종 merge head에서 통과했다. 아직 최종 head는 원격 source branch에
push하지 않았으므로, 사용자 승인 뒤 원격 CI와 mergeability를 재확인한 다음에만 병합 판단을 한다.
