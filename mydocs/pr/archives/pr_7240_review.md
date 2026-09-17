---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-17
---

# PR #7240 검토

**최종 판정: 머지 보류.** 아래 실행 결함을 해소하기 전에는 통합 PR 제출 준비 완료 또는 승인으로 판정하지 않는다.

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
