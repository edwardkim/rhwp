---
kind: review
status: active
---

# PR #7260 재검토 — 변경 요청

최종 판정: **머지 보류**. 기존 batch 성능 지적은 해결됐다. 남은 보류 근거는 PR이 주장한 편집·undo·RowBreak 원점 보존의 수용 증거이며, 한컴과의 모든 렌더 차이를 이 PR의 신규 회귀로 판정한 것은 아니다.

## 대상과 처리 경로

| 항목 | 값 |
| --- | --- |
| PR / 이슈 | [#7260](https://github.com/edwardkim/rhwp/pull/7260) / [#6639](https://github.com/edwardkim/rhwp/issues/6639) |
| 기여자 / reviewer | @lpaiu-cs / @postmelee |
| 코드 검토 SHA | `e5135e3f19ff26222254f1474adc87d2af04e870` |
| 증적 commit | `e1ac81c621f89a7c63ef974898eea958e1e6d046` |
| 비교 base | `505661360e9a2d596f55300d0cb0c5222f0e14b4` (`upstream/devel`) |
| 코드 변경 규모 | contributor 2 commits, 10 files |
| 원격 상태 참고값 | Ready, 기존 CHANGES_REQUESTED, maintainerCanModify=true |
| 처리 경로 | 사용자 지시로 현재 contributor branch `fix/6639-cell-vpos` 위에 증적·검토 기록만 추가 |

주 작업공간의 사용자 변경을 보존하기 위해 기존 review worktree를 사용했다. contributor commit을 rewrite하지 않았고 reviewer의 제품 source·Rust test·baseline 변경은 없다. metadata 재요청·merge·issue close는 이번 범위가 아니다. 증적 추가 후에는 새 문서 sample이 포함되므로 review-only fast-pass를 가정하지 않는다. 최신 CI는 merge 전에 따로 확인한다.

## 기존 요청의 해결 확인

실제 줄 흐름·문단 앞뒤 간격이 달라진 문단만 dirty로 표시하고 batch 종료에 한 번 처리한다. 정렬만 변경하거나 같은 값을 재적용하면 vpos 재계산을 예약하지 않는다. 셀 전체를 setter마다 순회하던 추가 O(N²) 작업과 중복 스타일 갱신을 제거했다. 성능 개선 배수 자체는 재계측하지 않았다.

소비 경로는 `apply_para_format_in_cell_native` / `set_cell_para_shape_id_native` → paragraph/core pending 표시 → `end_batch_native`의 `flush_cell_format_vpos` → 현재 vpos로 조각 구분 → `apply_cell_vpos_ladder` → 기존 dirty/rebuild/pagination 경로다. `rebuild_section`, snapshot 저장, mutable document 접근에서도 pending을 처리한다. 본 리뷰는 helper 존재만이 아니라 현재 좌표로 경계를 다시 구분하는 소비 조건을 확인했다.

## 기여자에게 요청할 보완

### 1. 같은 입력의 편집·복원 결과로 잔여 차이의 범위를 설명

[140% 직접 비교](../assets/pr_7260_edited140_native_wasm_review.png)에서 한컴보다 추가 줄바꿈과 큰 표 높이가 남는다. 원본 160% 비교에서도 차이가 있어 새 회귀로 단정할 수 없다. 초기 PR head와 현 head의 140% SVG가 같다는 사실은 이번 성능 보정의 무변화를 보여 주지만, PR 도입 전과의 비교를 대신하지 않는다.

기여자는 제공된 원본에서 160% → 140% → 원래 모양 ID 복원을 실행하고, PR 도입 전 코드와 수정 후 코드의 같은 페이지·셀을 비교해 달라. 후속 문단 시작점, 마지막 줄 끝점, 셀/표 외곽과 뒤 내용 위치를 함께 확인한다. 보이는 차이가 기존 줄 구성·폰트·입력 캐시 문제인지, 본 vpos/높이 갱신에 영향을 주는지 설명해야 한다. 변경 경로의 위반이면 수정과 수정 전 FAIL/후 PASS 회귀를 추가한다. 독립된 기존 문제라면 근거와 남는 제한을 PR 본문에 명시하며 전체 renderer 수정을 요구하지 않는다.

현재 reviewer의 원본 문단 vpos 복원 검사는 PASS지만, 모양 ID 복원 후 전체 SVG는 최초 화면과 다르다. 표 바탕 높이는 1009.12px에서 1025.3067px로 바뀐다. 이는 Studio의 실제 undo stack 검증과도 구분해야 한다. “좌표 일부 복원”을 “화면 전체 undo 일치”로 확대하지 말고 PR의 undo 주장 범위와 검사를 맞춰 달라.

### 2. 반복 편집으로 vpos 역행이 사라지는 RowBreak 경계

[`flush_cell_format_vpos`](../../../src/document_core/commands/text_editing.rs)는 매 flush마다 **현재** 인접 문단 첫 줄 vpos의 역행으로 조각을 나눈다. PR 테스트의 `stored_cell`을 사용하면 다음 합성 계약 실패가 재현된다.

| 단계 | 첫 줄 vpos |
| --- | --- |
| 최초 160% | `[100,1700,1600,3200]` |
| batch 140% 적용 | `[100,1500,1600,3000]` |
| 원래 shape ID 복원 기대 | `[100,1700,1600,3200]` |
| 실제 복원 | `[100,1700,3300,4900]` |

첫 축소로 좌표 역행이 사라지면 다음 flush에서 두 번째 조각 원점을 잃는다. 이 입력은 수동 좌표를 쓴 합성이며 정상 한컴 저장본에서 유효한 초기 상태인지는 **미검증**이다. 실제 문서 회귀라고 쓰지 않는다. 기여자는 해당 초기 상태의 유효성·지원 계약을 독립 근거로 확인해 달라. 유효하면 조각 경계/원점을 반복 편집·undo에서도 유지하는 수정과 정식 회귀를 추가한다. 유효하지 않다면 어떤 저장 규칙으로 배제되는지 설명하고, 유효한 경계 사례로 현재 보존 주장을 검증해 달라. 수치 임계값을 더 붙이는 문서별 보정은 권하지 않는다.

재현은 기존 `tests/cases/issue_6639_cell_para_format_vpos.rs`의 helper를 그대로 사용하는 아래 테스트다. reviewer는 별도 임시 example에서 실행했으며 이번 commit에 Rust test를 추가하지 않았다.

```rust
#[test]
fn review_rowbreak_origin_survives_shrink_then_undo() {
    let positions = [100, 1700, 1600, 3200];
    let (mut core, para, ctrl) = stored_cell(&positions);
    let ids = cell_shape_ids(&core, para, ctrl);
    core.begin_batch_native().unwrap();
    for idx in 0..4 {
        core.apply_para_format_in_cell_native(
            0, para, ctrl, 0, idx, r#"{"lineSpacing":140}"#,
        ).unwrap();
    }
    core.end_batch_native().unwrap();
    assert_eq!(cell_vpos(&core, para, ctrl), [100, 1500, 1600, 3000]);
    core.begin_batch_native().unwrap();
    for (idx, id) in ids.into_iter().enumerate() {
        core.set_cell_para_shape_id_native(0, para, ctrl, 0, idx, id).unwrap();
    }
    core.end_batch_native().unwrap();
    assert_eq!(cell_vpos(&core, para, ctrl), positions);
}
```

## 검증과 증적

[입력 README](../../../samples/issue6639/README.md)에 원본·파생 입력 4개와 PDF 3개의 저장소 경로, 출처, 생성 절차, byte 수, SHA-1/SHA-256, 한컴 job 및 engine 정보를 모았다. 실제 실행 파일과 증적 commit의 7개 Git blob을 SHA-256으로 대조해 모두 일치를 확인했다. 원본 SHA-256은 `983df661a2316881457ee4604c3084895bd4f6b350df4953c6c53cca8c162801`이다.

| 검사 | 결과 및 한계 |
| --- | --- |
| #6639 focused | 8/8 PASS |
| #4118 batch 동등성 | 1/1 PASS |
| 실제 원본 native 편집 | 140% 마지막 문단 `[32760,34020]`, 텍스트 입력/삭제 후 유지, 원래 vpos 복원 PASS |
| 합성 추가 계약 | 위 RowBreak shrink/복원 1개 FAIL; 실제 지원 입력 유효성은 미검증 |
| fresh WASM | 동일 코드 SHA에서 새로 빌드하고 실제 Chromium 실행. 10문단 140%, 1쪽, Native SVG byte equality 및 텍스트 왕복 PASS |
| 화면 전체 복원 | 원본 SVG와 달라 미충족 관측. 새 PR 회귀 여부 및 원인은 미검증 |
| 원본 Native/fresh WASM sweep | 각각 1쪽 compare·overlay·review 직접 확인. 자동 flagged 0이지만 시각 차이는 남음 |
| 140% 실제 편집 비교 | canonical webfont 및 compare/overlay/review helper로 비교. 편집 render-tree heuristic은 미실행 |
| 파생 입력 대조 | 160% 한컴 대조 PDF raster가 원본 PDF와 동일. README 재생성 코드의 두 HWPX가 검증 입력과 byte 동일 |
| 새 fixture 검사 | 신규 4개 샘플 대상 injection/보안 2개 PASS, IR 전수 왕복 래칫 82.072초 PASS(총 3/3). layout-anomaly는 4개 모두 모든 신호 0. 기존 자동 수집 범위에 포함되며 baseline 추가·완화는 불필요했다. |
| baseline/golden | 변경 없음 |

코드 SHA의 [CI 35820580415](https://github.com/edwardkim/rhwp/actions/runs/35820580415)와 [Render Diff 35820580055](https://github.com/edwardkim/rhwp/actions/runs/35820580055) 성공을 기존 코드 검토에 사용했다. CI WASM Build job은 skipped였으므로 reviewer가 별도 빌드했다. 이 결과를 새 fixture commit의 최신 전체 CI 통과로 쓰지 않는다. reviewer가 새 Rust source/test/helper를 수정하지 않아 새 Rust lint 묶음은 실행하지 않았으며, 새 sample 검사 결과는 별도로 기록했다.

재실행 명령(검토용 별도 target을 사용할 것):

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs issue_6639_cell_para_format_vpos -- --cargo-profile release-test --target-dir target/pr-review
node scripts/run-rust-test.mjs issue_4118_cell_format_batch_deferral -- --cargo-profile release-test --target-dir target/pr-review
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir /tmp/pr7260-fresh-wasm --dev --no-opt
RHWP_BIN=target/pr-review/release-test/rhwp venv/bin/python tools/fidelity_compare/fidelity_compare.py 0 0 --source samples/issue6639/rhwp-table-cell-minimal-repro.hwp --reference-pdf pdf/issue6639/issue6639-original-160-2020.pdf --label original160 --reference-grade Hancom2020 --text-only --export-all-svg --layout-ledger --out-dir /tmp/pr7260-fidelity
venv/bin/python scripts/visual_sweep.py --file-target original160 samples/issue6639/rhwp-table-cell-minimal-repro.hwp pdf/issue6639/issue6639-original-160-2020.pdf --rhwp-bin target/pr-review/release-test/rhwp --pages 1 --dpi 96 --out /tmp/pr7260-native
venv/bin/python scripts/visual_sweep.py --file-target original160 samples/issue6639/rhwp-table-cell-minimal-repro.hwp pdf/issue6639/issue6639-original-160-2020.pdf --rhwp-bin target/pr-review/release-test/rhwp --wasm-pkg /tmp/pr7260-fresh-wasm --pages 1 --dpi 96 --out /tmp/pr7260-wasm
```

140% 편집은 **원본 HWP**를 열고 `beginBatch()` → 대상 `(0,0,2,31)`의 문단 index 0..9에 `applyParaFormatInCell(..., '{"lineSpacing":140,"lineSpacingType":"Percent"}')` → `endBatch()` → `renderPageSvg(0)` 순서로 실행했다. 별도로 원래 shape ID들을 저장해 batch로 `setCellParaShapeId`를 호출했다. 이 경로는 Studio undo 버튼을 누른 검증이 아니다. 140% 기준 HWPX를 rhwp에서 단순히 열어 렌더한 것을 원본의 실시간 편집 결과로 대신하지 않았다.

- fresh JS SHA-256: `a75560f1a0619ccc206444ec9da320b057375f11773611e90b41ce3d9c218175`
- fresh WASM SHA-256: `1b0278cfe54d61d044efd162b3b96a0536161cfdd737bbeccf0bf02e110ede88`
- 140% Native/WASM SVG SHA-256: `fa0ff3a16e57fbc06ed5c64528b93597cba9e65e5f869601bcbec5447c56758a`
- 임시 원장·로그·SVG: `/private/tmp/pr7260-rereview-evidence` (재현 절차·입력과 대표 PNG는 저장소에 보존)

| 직접 확인한 PNG | pixel match | 내용 중심 proxy |
| --- | --- | --- |
| [원본 Native](../assets/pr_7260_original160_native_review.png) | 91.15192% | 8.54662% |
| [원본 fresh WASM](../assets/pr_7260_original160_wasm_review.png) | 91.15012% | 8.54504% |
| [140% 편집 Native/WASM](../assets/pr_7260_edited140_native_wasm_review.png) | 93.04333% | 14.46969% |

## 조판 원칙 대조

| 항목 | 판정과 근거 |
| --- | --- |
| 원인·범위·일반성 | 기존 반복 순회 해결 충족. 저장 조각 경계의 반복 편집 유효성은 미검증 |
| 측정·배치 소비 | setter → pending → flush → ladder → 재배치 경로 확인. 최종 표 외곽의 기준 정합 및 복원 범위는 미검증 |
| 분할·이어받기 | 컷/예산 알고리즘 변경은 비해당. RowBreak 조각 원점 보존 주장에는 위 반례가 적용되나 정상 입력 여부 미검증 |
| 저장 정보 유효성 | 원본 저장본과 명시적 변형 HWPX를 구분. 대조군 성공을 원본 rhwp 일치로 승격하지 않음 |
| 독립 기대값 | 한컴 PDF를 확보. 합성 원점 기대는 최초 좌표·복원 계약이며 한컴 일치로 보지 않음 |
| 기준값 변경 | 비해당. 허용치 완화 없음 |
| 입력 commit 동일성 | 충족. 증적 commit의 7개 HWP/HWPX/PDF blob과 실제 검증 입력 hash 일치 |
| 완료·수용 증거 | 미검증 범위가 남아 머지 보류. 기존 코드 결함 해결과 필수 증거 부족을 구분 |

## 보류 해제와 원격 조치

기여자가 위 두 보완에 답하고, 필요한 코드·정식 회귀 수정 또는 독립적인 범위 설명을 제출하면 그 최종 head에서 재판정한다. 보완한 PDF/원본은 reviewer가 책임지고 보존하므로 기여자에게 MCP 접근을 요구하지 않는다. 이번 요청은 증적 push와 Request changes review이며 approve·merge·issue close는 포함하지 않는다. 새 sample을 포함한 최신 head의 CI 결과는 아직 미확정이다.

## Merge 후 contributor PR comment 계획

현재 머지 보류이므로 merge comment를 게시하지 않는다. 해제 후 최종 head에서 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)에 따라 source SHA·입력·페이지·compare/overlay/review와 잔여 차이를 갱신한다. merge가 실제 완료되면 대표 PNG를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/<file>.png` 형식으로 고정하고 결과·기여자 credit을 포함한 본문을 UTF-8 파일의 `--body-file`로 게시한 뒤 API로 확인한다. 미래 CI·merge·issue 종료를 완료로 기록하지 않는다.
