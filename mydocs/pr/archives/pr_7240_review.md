---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-17
---

# PR #7240 검토

**보정 후 판정: 기존 실행 결함 해소, 최종 통합 검증 대기.** 아래 발견 사항은 보정 전 기록이다. 후단의 보정 결과·실제 증적을 함께 본다. 통합 전체 승인은 #7242·#7243 보류 사항과 최종 게이트 해소 전까지 보류한다.

- 원 PR: [#7240](https://github.com/edwardkim/rhwp/pull/7240), source `3276bb635d66e11e7fa16f5f83173e7a944f0e60`.
- 접수 시점: OPEN/non-draft/devel, CONFLICTING/DIRTY. [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/35212983067) 성공은 확인했으나 통합 후 결과와 다르다.

## 발견 사항

### P1 — 쪽 나눔 보존 주장이 HWPX 저장에서 깨짐

[src/document_core/commands/text_editing.rs](../../../src/document_core/commands/text_editing.rs)의
3899–3902줄은 `column_type=Column`, `raw_break_type |= 0x08`을 설정한다.
이미 명시적 Page인 문단은 메모리 raw=0x0c가 되지만
[HWPX writer](../../../src/serializer/hwpx/section.rs)의 `render_hp_p_open`은 enum만 소비한다.
그 결과 `pageBreak=0`, `columnBreak=1`로 저장돼 재열기 raw=0x08이 된다.
HWP adapter는 raw 비트를 보존해 같은 편집의 재열기 결과가 Page/raw=0x0c이다.

이는 테스트 보정을 마친 뒤에도 실제 저장으로 재현했다. 현재 원 PR 테스트는 저장 전
메모리 값만 검사하므로 4 PASS여도 이 유실을 검출하지 못한다.

해제 조건: page/column 두 축의 편집·HWPX/HWP 저장·재열기·실제 배치가 동일 계약을 소비하도록
보정하고, 두 명령 순서와 반복·구역 시작 경계를 포함한 제품 round-trip 검사로 입증한다.
raw 비트 assertion만 유지하는 것으로 해소하지 않는다.

### P1 — Studio 사용자 단 나눔을 속성 setter로 바꿔 첫 문단에서 나눔이 없어짐

`insert_column_break_native`는 CLI뿐 아니라 `wasm_api::insertColumnBreak`를 통해
Studio `page:column-break`(Ctrl+Shift+Enter)와 Studio HwpCtrl `BreakColumn`이 호출한다.
새 offset 0 조기 반환은 사용자 커서 이동·문단 분할까지 바꾼다.

기존 `samples/issue7218/outline_headings.hwpx`의 첫 문단에서 동일 Native 진입점을 호출했다:

| 코드 | 문단 수 | page_count | 반환 커서 |
| --- | --- | --- | --- |
| 기준 devel | 5→6 | 1→3 | paraIdx=1, charOffset=0 |
| 통합 코드 | 5→5 | 1→1 | paraIdx=0, charOffset=0 |

기준의 3쪽을 한컴 정답으로 승인한 것은 아니다. 새 코드에서는 한 단 문서 첫 문단의
사용자 나눔 자체가 없어졌음을 보여주는 제품 회귀 대조다. 저장본에 내용 문단의
columnBreak가 많다는 통계는 Ctrl+Shift+Enter/BreakColumn 명령 동작의 oracle가 아니다.
#7241에서 쪽 나눔의 저장 setter와 사용자 분할 명령을 분리한 이유와도 충돌한다.

해제 조건: CLI/MCP의 문단 앞 속성 setter와 사용자 편집 명령을 분리해 기존 명령을 보존하거나,
독립 한컴 BreakColumn 측정으로 제품 변경을 입증하고 실제 WASM/Studio 커서·반복·Undo/Redo·
저장 재열기를 검증한다. Windows Hwp 프로세스(PID 9932)가 이미 사용 중이라 이를 종료하지 않았고,
이번 한컴 COM 명령 측정은 **미실행**이다. 이전 BreakPage 1건을 BreakColumn 증거로 쓰지 않는다.

## 직접 실행 결과와 재현

- 원 column 경계 테스트: 3 PASS / 1 FAIL → setup 보정 후 4 PASS.
- Native 실제 입력 round-trip: before save Column/raw=0x0c → HWPX Column/raw=0x08,
  HWP Page/raw=0x0c. 구역 0·문단 3의 동일 텍스트를 대상으로 했다.
- 첫 문단 명령: 위 표의 기준/통합 차이를 같은 입력·메서드로 실행했다.
- 검증 입력은 기존 [outline_headings.hwpx](../../../samples/issue7218/outline_headings.hwpx)를
  재사용한다. SHA-256 `0e65077c16ae889497d03b77178bb8f1039358a7a609badbf99966b6b694e98a`,
  기준 devel 및 누적 head에 존재한다. HWPX/HWP export는 메모리 bytes로 재열었고 파일을 새로
  이름 변경해 커밋하지 않았다. PDF/시각 일치는 미검증이다.

```rust
use rhwp::document_core::DocumentCore;
fn main() {
    let b = std::fs::read("samples/issue7218/outline_headings.hwpx").unwrap();
    let mut d = DocumentCore::from_bytes(&b).unwrap();
    d.mark_page_break_at_paragraph_start_native(0, 3).unwrap();
    d.insert_column_break_native(0, 3, 0).unwrap();
    assert_eq!(d.document().sections[0].paragraphs[3].raw_break_type & 0x0c, 0x0c);
    let bytes = d.export_hwpx_native().unwrap();
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    // 현재 통합본에서는 0x08이므로 실패한다.
    assert_eq!(reopened.document().sections[0].paragraphs[3].raw_break_type & 0x0c, 0x0c);
}
```

rlib 연결 방법은 [#7239 재현 명령](pr_7239_review.md#직접-실행-결과)을 따른다.
CLI/MCP 설명도 아직 “지정 오프셋에서 분할”로 남아 있어 최종 의미 결정 뒤 정렬해야 한다.
이미 CLOSED인 #5019·#7218을 이번 검토가 새로 해결했다고 기록하지 않는다.

## 공통 조판·편집·입력 심사

| 항목 | 판정·근거 |
| --- | --- |
| 구현 근거·일반성 | 미충족: 저장 속성을 사용자 명령 동작의 근거로 일반화 |
| 측정·배치 일관성 | 미충족: raw bits → HWPX enum 소비에서 한 축 유실, HWP와 다름 |
| 분할·이어받기 | 문단 분할 경계 적용. first offset 0 사용자 명령의 문단/페이지/커서 변화 확인, 일반 표 continuation 비해당 |
| 줄 소속·점유 높이 | 기존 reflow/vpos/recompose 사용. 독립 한컴 단 나눔 배치와 직접 Visual Sweep은 미검증 |
| 사례·독립 기대값 | Native 기준 대조·저장 재열기 충족. 한컴 COM·fresh WASM·UI Undo/Redo 미검증 |
| 기준값 변경 | 비해당, assertion 완화 없음. setup은 #7241의 실제 setter로 정렬 |
| 주장·검증 범위 | 메모리 보존 통과와 파일 저장 유실을 분리. 원 CI 녹색을 통합 승인으로 승격하지 않음 |
| 검증 입력 커밋 | 충족, 기존 입력 hash와 Git blob 일치 확인. 동일 입력 중복 추가 없음 |

Studio command는 기존 snapshot/executeOperation/full-refresh 경로를 유지하지만,
core의 의미와 반환 커서가 바뀐다. 라우터가 존재한다는 사실만으로 실제 Undo/Redo를 통과했다고 쓰지 않는다.

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

## Merge 후 contributor PR comment 계획

현재 **머지 보류**이므로 merge/close 또는 승인 코멘트를 게시하지 않는다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)에
따른 직접 캡처는 아직 없고, 이전 PR의 PNG·수치를 이 head 증적으로 재사용하지 않는다.
blocker 수정 뒤 실제 Native/fresh WASM compare·standalone overlay·review, 페이지·후보 수·pixel/ink 지표와
사람의 판정을 이 기록에 추가하고 merge SHA에 고정한 대표 이미지 목록을 확정해야 한다.
그 뒤 사용자 승인·최신 head CI·실제 merge가 완료됐을 때만 한국어 감사와 범위/잔여 문제,
실제 CI URL·이미지를 UTF-8 파일+`--body-file`로 게시하고 재조회한다.

원격 source push·PR 생성·GitHub comment·merge·close는 이번 검토에서 수행하지 않았다.
reviewer 지정만 원격 metadata에 반영했다. 오늘할일은 최종 제출 준비 단계에서 작성한다.

## 메인터너 보정 회차 2 — 분석

CLI/MCP의 `offset=0`은 기존 문단 속성 설정으로 유지하되, Studio의 Ctrl+Shift+Enter는 기존 사용자 명령처럼 분할한다. 별도 column setter를 만들고 CLI에서만 연결한다. 쪽/단 직교 속성을 HWPX writer가 함께 저장하도록 보완하고, 양쪽 명령 순서·HWP/HWPX 재열기·시작 위치 사용자 명령을 검사한다. 원 PR의 속성 검사는 setter를 직접 검증하며 사용자 명령 계약과 혼동하지 않는다.

### 회차 2 결과보고

- 사용자 `insert_column_break_native`는 기존 분할 경로로 복원했다. CLI/MCP의 offset 0은 별도 `mark_column_break_at_paragraph_start_native`를 호출하며 봉투에 `paragraphDelta`/`columnBreakParagraph`를 제공한다.
- HWPX writer가 Page/Column enum 하나만 보지 않고 독립적인 raw 0x04/0x08을 함께 저장한다. 합성 페이지 경계는 저장하지 않는다. column setter는 기존 explicit Page 우선순위와 enum-only legacy 쪽 속성도 보존한다.
- 수정 전 새 반례: 시작 위치 사용자 명령 **5≠6 문단**, HWPX 두 속성 **08≠0c**로 각각 실패. 수정 후 코어 **7 passed**, 실제 CLI **4 passed**, 기존 page-break 대조 **11 passed**.
- CLI 첫 재실행은 suite 준비 전이라 0 tests였으며 증거에서 제외했다. `--prepare` 후 실제 4개 실행/통과를 확인했다.
- 합성 page 경계 테스트는 파서 계약과 같은 `raw=0, enum=Page, synthesized=true`로 준비했다. 명시 raw=4를 합성이라고 표기한 부적절한 초기 입력을 독립 계약에 맞게 정정했다.
- 최종 통합 lint·전체 회귀·Native/fresh WASM 및 사용자 경로 검증은 후속 회차에서 완료한다. 아직 최종 승인이 아니다.

## 최종 통합 후보와 시각 증거

- code head: `75a48488676d79a0357ba1cae6c863ac2120b668`, base `236a601da803b53429e9090eef652c661dd3bfe2`.
- Native SHA256: `46d87aedbeca44eb31a31ddccd2c6b7e8deebd4008bc2bbe98598af67bdc8ca9`.
- fresh WASM SHA256: `6488efc93f6635ef0fe193e09d7982cfd48231d99679007cd8d3116f61c2dbc5`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- Mac arm64/Rust 1.93.1, 별도 verify checkout의 source/test를 위 head와 바이트 대조했다. review target은 `target/pr7239-7240-review-20260917`. Docker 표준 경로 대신 host `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-final --no-opt`를 실행했다. wasm-opt 통과로 주장하지 않는다.
- `venv/bin/python scripts/visual_sweep.py --file-target <key> <입력> <PDF> --rhwp-bin <scratch>/rhwp-current --pages <아래 쪽> --dpi 96 --out <scratch>/native-sweep`, WASM은 `--wasm-pkg <scratch>/wasm-final` 추가. 최종 head로 재캡처한 compare·standalone overlay·review를 직접 판독했다.
- 전체 nextest·Native Skia 3종은 이번 후보에서 미실행이다. #7242 시각 보류 사유가 있어 지침의 작은 경계/영향 페이지 확인을 먼저 완료했고, 대규모 회귀를 통과 근거로 대신하지 않는다. 최종 승인·PR 제출 준비 완료가 아니다.

### 보정 후 판정

기존 두 보류 결함은 `df1b1de7f`로 해소했다. 첫 문단 사용자 명령은 분할과 다음 문단 커서를 복원했고, CLI offset 0은 속성 setter로 분리했다. 양 순서의 쪽/단 비트와 합성 쪽 경계의 비저장을 HWPX/HWP 재열기로 확인했다.

Chrome의 fresh WASM `insertColumnBreak`에서 첫 문단·일반 문단 시작·빈 문단·반복·중간 5개를 실행했다. 문단 +1(반복 +2), 텍스트 보존, 커서 +1/offset 0, HWPX/HWP 재열기 모두 통과했다. Studio가 호출하는 실제 WASM 진입점 검증이며 키보드 UI·Undo/Redo 자동화까지 실행한 것으로 세지 않는다. 사용자 명령 구현은 기준 devel 분할 경로를 복원했으며 snapshot/Undo 코드는 바꾸지 않았다.

편집 산출물 [column_start.hwpx](../../../tests/fixtures/pr7240_review/column_start.hwpx)는 기존 outline 입력의 구역0/문단3/offset0 CLI 명령으로 생성했다. SHA256 `b12bcdebd7d94bf1e31a5cc40e3ed932b0a8aaa89fca32113f55205392748152`. 새 단 속성 산출물이므로 기존 파일을 단순 이름 변경한 복제가 아니다. 한컴 변환 job `53f9caba-badb-4832-90e5-374a2320d677`, start→status succeeded→download, engine2020/Hancom11.0.0.9136, input_preprocess none, [PDF](../../../pdf/pr7240/column-start-2020.pdf) 2쪽, SHA256 `ce13a60d3c94151b759c8843d309204bdb62b59442f3cae543c7c11fc87d3cc1`.

Native/fresh WASM은 2쪽 구성이 PDF와 일치하고 대상 제목 앞 추가 빈 문단 없이 다음 쪽으로 이동한다. 재열기 파일을 기준 devel로 열어도 같은 SVG이므로 남은 개요 번호 `1.Second` 대 PDF `2. Second`, 글꼴/간격 차이를 이번 보정의 새 회귀로 세지 않는다. 전체 개요 번호 정합성까지 해소했다고 주장하지 않는다.

### 직접 판독한 PNG

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| column p1 | [compare](../assets/pr7240_review/native_column_compare_001.png) · [overlay](../assets/pr7240_review/native_column_overlay_001.png) · [review](../assets/pr7240_review/native_column_review_001.png) | [compare](../assets/pr7240_review/wasm_column_compare_001.png) · [overlay](../assets/pr7240_review/wasm_column_overlay_001.png) · [review](../assets/pr7240_review/wasm_column_review_001.png) |
| column p2 | [compare](../assets/pr7240_review/native_column_compare_002.png) · [overlay](../assets/pr7240_review/native_column_overlay_002.png) · [review](../assets/pr7240_review/native_column_review_002.png) | [compare](../assets/pr7240_review/wasm_column_compare_002.png) · [overlay](../assets/pr7240_review/wasm_column_overlay_002.png) · [review](../assets/pr7240_review/wasm_column_review_002.png) |

Merge 후 코멘트에는 위 **모든 영향 페이지**의 compare·overlay·review 링크를 실제 merge SHA의 raw URL로 치환한다. 대표 review만 넣고 standalone overlay를 빠뜨리지 않는다. 지금은 remote push/comment/merge를 수행하지 않았다.

### 최종 head 공통 검증 결과

- fmt check, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy(`-D warnings`), suite manifest base 비교: **모두 통과**.
- 최종 head focused 재실행: WMF fuzz **2**, golden **1**, column core **7**, column CLI **4**, 기존 page-break **11**, stored tail **3**, scaffold height **2** — **30 passed / 0 failed**. 각 원본에 `node scripts/run-rust-test.mjs <module>`를 test profile로 실행했다.
- Native/fresh WASM **각 9쪽** compare·standalone overlay·review, 총 18쪽 직접 확인. PNG 54개와 #7242 base 대조 PNG 6개를 개별 PR asset 경로에 보존했다. 총 60개이며 원 입력과 기준 PDF를 연결했다.
- test source·수치 baseline·허용치를 통과 목적으로 완화하지 않았다. source-side unit test 변경은 없어 unit-tier 비교는 비해당이다.
- 문서별 metadata 및 로컬 링크 검사, `git diff --check` 통과. 불필요한 log/JSON/TSV는 Git에 추가하지 않는다.

- 새 sample 3개(`stored-table-text-tail/native-8-0`, `issue7234/short_table_cell_row_height`, `issue7234/tall_table_cell_row_height`)의 hidden-text/injection/unicode 검사: **1 passed**. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 실제 대상을 지정해 실행했다.
