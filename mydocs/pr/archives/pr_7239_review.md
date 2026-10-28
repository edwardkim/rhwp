---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7239 검토

## 현재 최종 판정 — 2026-09-18

**승인 — 메인터너 보정이 포함된 통합 코드의 #7239 변경 범위.** 원 contributor head를
그대로 승인하거나 네 PR 전체의 병합을 승인한다는 뜻은 아니다. 아래 초기 발견 사항과
검증 대기 문구는 당시 기록이며, 현재 판정은 이 절을 따른다.

META_TEXTOUT의 TOP/BOTTOM 덧셈 overflow를 보정했다. overflow checks가 켜진
프로필에서 반례의 수정 전 실패·수정 후 통과와 정상 WMF/EMF golden 보존을 확인했다.
보정 commit은 `7521c08c8`다. 경계·시각 검증 후보 `75a484886`부터 최신 코드까지
WMF converter 제품 경로가 변경되지 않았음을 Git diff로 확인했다.
기존 수식 글꼴·표 선/글꼴 차이는 해결 범위 밖으로 기록한다.

### 최신 공통 검증과 증거 범위

- #7243의 이어받기 높이 보류 사유도 메인터너 보정으로 해소했다. 네 PR의 보정된 통합 코드에
  대한 개별 검토가 모두 승인 상태다. 원 contributor head를 그대로 승인한 것으로 표현하지 않는다.
- 최신 제품 소스·binary hash·정확한 명령·시각 범위는
  [#7243 최종 실행·증적](pr_7243_review.md#최종-실행증적)을 따른다.
  전체 회귀 **10,023 passed / 50 skipped**, Native Skia lib **4,112 passed / 13 ignored**,
  그림 **2 passed**, 직접 PDF **4 passed**, fmt·Clippy·build·manifest를 모두 통과했다.
- #7239 WMF와 #7240 명령 경로는 이번 보정에서 변경하지 않았다. 아래 원 PR 시각 증거의
  캡처 후보와 이번 #7243 Native/fresh WASM 34쪽 캡처를 구분한다.
- 브랜치 `codex/pr7239-7240-review-20260917`, 고정 base `236a601da803b53429e9090eef652c661dd3bfe2`.
  제출할 최종 head의 원격 CI·mergeability는 별도 확인해야 하며 이번에 push·merge하지 않았다.

## 원 PR 접수 및 과거 회차 기록

- 원 PR: [#7239](https://github.com/edwardkim/rhwp/pull/7239), source `f5169220e8a165eb6c9344357d2d83e8a422ef52`.
- 접수 시점: OPEN/non-draft/devel, MERGEABLE/CLEAN. [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/35212978619) 성공은 확인했으나 통합 후 결과와 다르다.

## 발견 사항

### P1 — 동일 text_out 경로에 baseline 덧셈 넘침이 남음

[src/wmf/converter/svg/mod.rs](../../../src/wmf/converter/svg/mod.rs)의 `text_out`은 글꼴 높이의
`abs()`만 `saturating_abs()`로 바꾸고 `record.y_start + vertical_alignment_offset`은 i16 덧셈으로
남겨 두었다(검토 코드 1519–1525줄). 형제 `ext_text_out`의 같은 덧셈은 보정되어 있다.

102바이트 정상 구조 WMF(글꼴 높이 100, 기본 TOP 정렬, META_TEXTOUT y=32767)의 제품
`WMFConverter::run()`에서 `attempt to add with overflow`를 재현했다. `32767 + 80` 경계다.
기준 devel에서도 재현하므로 **신규 회귀가 아니라 이 PR이 수정한 동일 경로의 잔존 패닉**이다.
예약 fuzz가 검사하는 WMF converter의 입력 기반 패닉 방지를 이 경로까지 마쳐야 한다.

원 기여의 10 fixture는 수정 전 9개 패닉, 수정 후 10개 모두 무패닉을 직접 확인했다.
따라서 원 수정의 효과를 부정하지 않는다. 별도 TEXTOUT 입력은 그 10개에 없는 경계이며,
이를 전체 회귀 또는 release-test 통과로 덮어 승인하지 않는다.

해제 조건: TOP/BOTTOM baseline 계산을 일관된 안전 산술로 보정하고, 이 입력을 정식
회귀에 포함해 overflow checks on에서 수정 전 FAIL/수정 후 PASS를 확인한다. 정상 대조군의
좌표·획·clip 불변 및 최종 검증도 확인한다. 해결 범위 밖 main release/예약 workflow는 변경하지 않았다.

## 직접 실행 결과

| 실행 | 결과 |
| --- | --- |
| 원 WMF fixture 10개, 현재 통합 코드 | aggregate 1 PASS, 10개 무패닉 |
| 같은 fixture, 기준 devel WMF 코드 | aggregate FAIL, 9개 패닉; main_window_origin_abs만 무패닉 |
| 새 TEXTOUT 102byte 경계, 기준/통합 각각 | 둘 다 덧셈 overflow 패닉 |
| wmf_emf_goldens 정상 대조 | 1 PASS, baseline 갱신 없음 |

아래 입력은 단순 파일명 변경 사본이 아닌 새로운 최소 재현이다.

| 입력 | 역할·출처 | SHA-256 |
| --- | --- | --- |
| [textout_baseline_i16_max.wmf](../../../tests/fixtures/pr7239_review/textout_baseline_i16_max.wmf) | reviewer 합성, LOGFONT(100)+SELECTOBJECT+TEXTOUT(y32767)+EOF | `d3f7d24b08822908b197cf80db4f3e808f273f6353bb6903a2427b3334b43ee8` |

원 10개 입력은 `tests/fixtures/wmf_fuzz/`를 재사용한다. HWP/HWPX/PDF 입력은 이 PR의
실행에 사용하지 않았다. 한컴 oracle/PDF 시각 일치로 주장하지 않는다.

재현기(`probe.rs`로 저장, 위 test 프로필 빌드 뒤 해당 rlib와 연결):

```rust
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};
fn main() {
    let b = std::fs::read("tests/fixtures/pr7239_review/textout_baseline_i16_max.wmf").unwrap();
    let result = std::panic::catch_unwind(|| {
        WMFConverter::new(b.as_slice(), SVGPlayer::new()).run()
    });
    assert!(result.is_ok(), "META_TEXTOUT baseline overflow");
}
```

```sh
rustc --edition 2021 probe.rs \
  --extern rhwp=target/pr7239-7240-review-20260917/debug/deps/librhwp.rlib \
  -L dependency=target/pr7239-7240-review-20260917/debug/deps -o /tmp/pr7239-probe
/tmp/pr7239-probe
```

## 공통 조판 원칙·입력 심사

| 항목 | 판정·근거 |
| --- | --- |
| 구현 근거·일반성 | 부분 충족. 입력 기반 산술·길이 검증이며 문서 ID 분기 없음. text_out 동일 경계 누락은 위 미충족 |
| 측정·배치 일관성 | 일반 paragraph layout 비해당. WMF TOP/BOTTOM 좌표 산술의 형제 경로 불일치는 미충족 |
| 분할·이어받기·줄 소속·높이 | 비해당, 문서 pagination 소유 규칙 변경 없음 |
| 독립 사례 | 충족: 수정 전 9/10 패닉, 수정 후 0/10; 별도 작은 TEXTOUT 대조는 양쪽 실패 |
| 기준값 변경 | 비해당. golden/baseline 허용치 변경 없음 |
| 주장·검증 범위 | 패닉 잔존 미충족, Native/fresh WASM 시각 증거 미검증. 정상 golden 통과를 PDF 일치로 승격하지 않음 |
| 검증 입력 커밋 | 원 10개는 원 체리픽 commit, 새 102byte fixture는 이 검토 결과와 함께 보존. 기존 파일 복제 없음 |

## 접수·분석과 검토 범위

- 작성자 planet6897, reviewer jangster77 지정 완료. 기존 contributor이며 첫 기여 절차는 비해당.
- 경로: collaborator 외부 PR의 devel 기반 체리픽 통합 검토, 다수 PR·local validation·visual fixture 절차 적용.
- 기준 `upstream/devel`: `236a601da803b53429e9090eef652c661dd3bfe2`.
- 브랜치: `codex/pr7239-7240-review-20260917`, 누적 제품 코드 head: `36235b8ad1c9ee7edc5f2a2820a30b681493b39b`.
- #7239 → #7240의 고유 commit 순서로 적용했고 원 저자와 `cherry-pick -x` 계보를 유지했다.
  #7240에 포함된 `103ab177a`(#7238)는 #7241에서 보정·반영됐으므로 다시 적용하지 않았다.
- 텍스트 충돌 없음. 원 #7240의 GitHub DIRTY를 source branch force-push로 숨기지 않고,
  최신 devel 위 고유 변경만 적용했다. 아래 실행으로 찾은 **테스트의 의미 충돌**은 별도 보정했다.
- 분석 → 작은 경계 실행 → 결과보고 → 커밋 순서. 이번 결과는 원 source CI와 구별한 통합본 검사다.

## 통합 충돌 보정

`a_column_break_at_paragraph_start_keeps_the_other_break_axes`는 종전의
`insert_page_break_native(0, HEADING_PARA, 0)`를 속성 준비에 사용했다. #7241 이후 이 명령은
Studio의 사용자 문단 분할을 보존하므로 테스트는 실제로 다른 문단을 검사해 raw=0x08로 실패했다.
명시적 속성 setter `mark_page_break_at_paragraph_start_native`로 준비 방식을 수정했다.

- 보정 전: 4개 중 3 PASS / 1 FAIL, nextest exit 100.
- 보정 후: 4 PASS, exit 0. `raw & 0x04`, `raw & 0x08`, 문단 5개 assertion은 모두 유지했다.
- 이 수정은 테스트 setup만 고쳤고 제품의 `insert_column_break_native` 구현은 원 PR과 같다.
  저장 시 bit 유실이나 사용자 명령 변경까지 해결한 것으로 간주하지 않는다.
- 수정 직후 generated suite 갱신 전에는 0 tests/exit 4가 한 번 발생했다. 이를 통과로 세지 않고
  review worktree에서 `--prepare` 후 4개가 실제 실행됨을 확인했다. 파생 suite/manifest는 커밋하지 않는다.

## 실행 환경과 증거 경계

macOS arm64, Rust 1.93.1, `DEVELOPER_DIR=/Library/Developer/CommandLineTools`.
전용 target `target/pr7239-7240-review-20260917`, 검증 checkout
`/Users/tsjang/rhwp-pr7239-7240-verify-20260917`에서 실행했다. 기본/공유 target은 삭제하거나 재사용하지 않았다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
# 아래 이름마다 동일 전용 target/DEVELOPER_DIR 환경으로 순차 실행
node scripts/run-rust-test.mjs wmf_fuzz_integer_overflow -- --no-fail-fast
node scripts/run-rust-test.mjs wmf_emf_goldens -- --no-fail-fast
node scripts/run-rust-test.mjs issue_7218_column_break_at_paragraph_start -- --no-fail-fast
node scripts/run-rust-test.mjs insert_column_break_contract -- --no-fail-fast
```

WMF overflow 검사는 **기본 test 프로필(overflow checks on)** 이다. release-test 녹색으로 정수
넘침을 검출했다고 주장하지 않는다. 검증 전용 checkout에서 WMF와 text editing 제품 파일만
기준 devel로 되돌려 음성 대조를 실행한 뒤, 누적 head로 복원하고 최종 focused를 다시 실행했다.
주 작업공간 제품 코드는 이 대조로 변경하지 않았다.

**미실행:** 통합본 전체 nextest·Native Skia·Clippy 묶음·fresh WASM·Studio UI E2E·Visual Sweep.
작은 경계에서 아래 blocker가 확인됐으므로 고비용 최종 수용 검증에 앞서 기록했다.
기존 원 PR의 녹색 CI를 이 통합본의 전체 검증 성공으로 재사용하지 않는다.
시각 비교를 재개하면 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)의
Native/fresh WASM compare·standalone overlay·review를 새 head로 직접 산출·확인해야 한다.

임시 로그·probe 실행 파일은 `/private/tmp/rhwp-pr7239-7240-review`에만 두며 커밋하지 않는다.
아래 재현 코드는 제품 rlib를 직접 호출한다. 구현을 복사한 대역으로 제품 결과를 대신하지 않았다.


최종 focused 재실행은 원 WMF 입력 aggregate 1 + 정상 WMF/EMF golden 1 + 보정한 단 나눔 경계 4 +
CLI/MCP 단 나눔 계약 3 = **9 PASS**였다. 이는 별도 probe로 재현한 blocker가 해소됐다는 뜻이 아니다.
변경한 테스트 rustfmt와 `git diff --check`도 통과했다. source·test 변경 뒤 전체 lint/회귀를
완료한 제출 후보가 아니며 원격 push하지 않았다.


### 원 PR crash 입력의 Git blob 대조

| 경로 | SHA-256 |
| --- | --- |
| `tests/fixtures/wmf_fuzz/bitmap_line_past_pixel_data.wmf` | `ca75598b3b15469329017687e3bdd2abcdce329ba0fcbfd44964caada60031c0` |
| `tests/fixtures/wmf_fuzz/ellipse_rect_i16_extremes.wmf` | `73975fc620325cc3fa1c99033f6ccfe4b67c2b5682640c72b16b66ae17a4feb2` |
| `tests/fixtures/wmf_fuzz/emf_escape_data_size_u32_max.wmf` | `374073f2e29fd7a0cd1a3b34acaa0d8a0e1ae123b9ecb438ef9a63856821b055` |
| `tests/fixtures/wmf_fuzz/eps_size_below_header.wmf` | `392ac52230e441d87084dfbfe07d2b6606d16f2c9741be2cd498c0902131499f` |
| `tests/fixtures/wmf_fuzz/font_escapement_i16_min.wmf` | `975af3d941663c04dce8fa438c44bfb255bdda48bca15c09669c00c71a4c8c6f` |
| `tests/fixtures/wmf_fuzz/get_color_table_start_after_byte_count.wmf` | `b8402351e2719706dd3f82334975f1b4a111a7e0b5723410af9ee0a951d2739a` |
| `tests/fixtures/wmf_fuzz/main_window_origin_abs.wmf` | `96248fd4f2af1b02e3aec27241b968443699c21ee21a8d22012713b036dacc6b` |
| `tests/fixtures/wmf_fuzz/pen_dash_width_i16_extreme.wmf` | `fdc901d47b8d8d197012741267a9da005831e1425f216207f3dbf0998f5d56d8` |
| `tests/fixtures/wmf_fuzz/text_baseline_offset_i16_extreme.wmf` | `75c96cb74d8e11750d63642995e90a053a294a246968a8ebb106e1f51ac30f2b` |
| `tests/fixtures/wmf_fuzz/window_ext_i16_min.wmf` | `405eb986582e8f3e8b5329500f2f566c57bb28e68f3ec59648dde6b7f6ec9f3e` |

## Merge 후 contributor PR comment 계획

#7239과 #7243의 보정된 통합 코드 검토를 승인했다. 최종 head CI와 실제 merge 전에
merge/close 완료 코멘트를 게시하지 않는다. 실제 merge 뒤에는 아래 직접 판독한 PNG의
모든 영향 페이지에 대해 Native/fresh WASM compare·standalone overlay·review를
`https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7239_review/<파일명>`
형식으로 연결한다. 각 이미지의 실제 캡처 후보와 검증 범위를 함께 밝힌다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하고, 실제 merge SHA·CI URL·보정 이유·잔여 차이·기여 감사를 한국어 UTF-8 파일과
`--body-file`로 게시한 뒤 한국어·이미지 URL·실제 head를 재조회한다.

원격 source push·PR 생성·GitHub comment·merge·close는 이번 검토에서 수행하지 않았다.
reviewer 지정만 원격 metadata에 반영했다. 오늘할일은 최종 제출 준비 단계에서 작성한다.

## 메인터너 보정 회차 1 — 분석

승인된 후속 보정으로 META_TEXTOUT의 TOP/BOTTOM 덧셈을 형제 ExtTextOut과 같은
포화 덧셈으로 맞춘다. i16 범위 내 정상 연산은 같고, MIN/MAX에서 wrap 또는 panic을
막는다. 최소 WMF를 정식 회귀에서 읽어 TOP/MAX, BOTTOM/MIN, 정상 TOP/BOTTOM,
BASELINE/MAX를 검사한다. 결과를 확인한 뒤 이 회차를 커밋한다.

### 회차 1 결과보고

- `META_TEXTOUT`의 TOP/BOTTOM 기준점 덧셈을 `META_EXTTEXTOUT`와 같은 안전 산술로 통일했다.
- overflow-checks가 켜진 test profile에서 새 경계 테스트는 보정 전 `1519:20 attempt to add with overflow`로 실패했다. 보정 후 TOP/BOTTOM 양 극값, 정상 좌표와 BASELINE 대조를 모두 실행하고 글자 A의 SVG 보존까지 통과했다.
- 원 fuzz 10입력 aggregate와 새 경계 검사: **2 passed**. 기존 WMF/EMF golden: **1 passed**, baseline 변경 없음.
- 이 회차는 패닉 반례를 해결한 코드 후보다. 최종 통합 head의 lint·전체 회귀·Native/fresh WASM Visual Sweep은 후속 회차에서 수행하므로 아직 최종 승인이 아니다.

## 최종 통합 후보와 시각 증거

- code head: `75a48488676d79a0357ba1cae6c863ac2120b668`, base `236a601da803b53429e9090eef652c661dd3bfe2`.
- Native SHA256: `46d87aedbeca44eb31a31ddccd2c6b7e8deebd4008bc2bbe98598af67bdc8ca9`.
- fresh WASM SHA256: `6488efc93f6635ef0fe193e09d7982cfd48231d99679007cd8d3116f61c2dbc5`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- Mac arm64/Rust 1.93.1, 별도 verify checkout의 source/test를 위 head와 바이트 대조했다. review target은 `target/pr7239-7240-review-20260917`. Docker 표준 경로 대신 host `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-final --no-opt`를 실행했다. wasm-opt 통과로 주장하지 않는다.
- `venv/bin/python scripts/visual_sweep.py --file-target <key> <입력> <PDF> --rhwp-bin <scratch>/rhwp-current --pages <아래 쪽> --dpi 96 --out <scratch>/native-sweep`, WASM은 `--wasm-pkg <scratch>/wasm-final` 추가. 최종 head로 재캡처한 compare·standalone overlay·review를 직접 판독했다.
- 전체 nextest·Native Skia 3종은 이번 후보에서 미실행이다. #7242 시각 보류 사유가 있어 지침의 작은 경계/영향 페이지 확인을 먼저 완료했고, 대규모 회귀를 통과 근거로 대신하지 않는다. 최종 승인·PR 제출 준비 완료가 아니다.

### 보정 후 판정

기존 `META_TEXTOUT` 오버플로 보류 사유는 `7521c08c8`로 해소했다. 원 fuzz와 양 극값·정상 좌표 대조는 test profile에서 통과했고 golden은 바뀌지 않았다. 실물 transistor 문서는 base와 11쪽 SVG가 모두 바이트 동일하다. 영향 WMF 회로도가 있는 2·5쪽의 Native/WASM raster도 서로 동일하며 PDF와 회로·배선을 직접 대조했다. 기존 수식 글꼴·표 선/글꼴 차이는 남으며 이 PR의 패닉 방어를 문서 전체 PDF 일치로 보고하지 않는다.

### 직접 판독한 PNG

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| wmf p2 | [compare](../assets/pr7239_review/native_wmf_compare_002.png) · [overlay](../assets/pr7239_review/native_wmf_overlay_002.png) · [review](../assets/pr7239_review/native_wmf_review_002.png) | [compare](../assets/pr7239_review/wasm_wmf_compare_002.png) · [overlay](../assets/pr7239_review/wasm_wmf_overlay_002.png) · [review](../assets/pr7239_review/wasm_wmf_review_002.png) |
| wmf p5 | [compare](../assets/pr7239_review/native_wmf_compare_005.png) · [overlay](../assets/pr7239_review/native_wmf_overlay_005.png) · [review](../assets/pr7239_review/native_wmf_review_005.png) | [compare](../assets/pr7239_review/wasm_wmf_compare_005.png) · [overlay](../assets/pr7239_review/wasm_wmf_overlay_005.png) · [review](../assets/pr7239_review/wasm_wmf_review_005.png) |

Merge 후 코멘트에는 위 **모든 영향 페이지**의 compare·overlay·review 링크를 실제 merge SHA의 raw URL로 치환한다. 대표 review만 넣고 standalone overlay를 빠뜨리지 않는다. 지금은 remote push/comment/merge를 수행하지 않았다.

### 최종 head 공통 검증 결과

- fmt check, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy(`-D warnings`), suite manifest base 비교: **모두 통과**.
- 최종 head focused 재실행: WMF fuzz **2**, golden **1**, column core **7**, column CLI **4**, 기존 page-break **11**, stored tail **3**, scaffold height **2** — **30 passed / 0 failed**. 각 원본에 `node scripts/run-rust-test.mjs <module>`를 test profile로 실행했다.
- Native/fresh WASM **각 9쪽** compare·standalone overlay·review, 총 18쪽 직접 확인. PNG 54개와 #7242 base 대조 PNG 6개를 개별 PR asset 경로에 보존했다. 총 60개이며 원 입력과 기준 PDF를 연결했다.
- test source·수치 baseline·허용치를 통과 목적으로 완화하지 않았다. source-side unit test 변경은 없어 unit-tier 비교는 비해당이다.
- 문서별 metadata 및 로컬 링크 검사, `git diff --check` 통과. 불필요한 log/JSON/TSV는 Git에 추가하지 않는다.

- 새 sample 3개(`stored-table-text-tail/native-8-0`, `issue7234/short_table_cell_row_height`, `issue7234/tall_table_cell_row_height`)의 hidden-text/injection/unicode 검사: **1 passed**. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 실제 대상을 지정해 실행했다.
