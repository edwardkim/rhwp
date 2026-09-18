---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7255 검토

## 최종 판정

**승인** — 2026-09-18 통합 검토.

대상 1쪽의 누적 위쪽 밀림이 개선되고 독립 PDF 위치를 검사하는 focused가 통과했다. #7253의 첫 노드 검사와 충돌하나, 빈 TextLine이 있다는 것만으로 제품 오류라고 판단하지 않는다.

대상 빈 슬롯 보존 변경은 승인한다. #7253 통합 검사 충돌과 다른 보류 해결 및 최종 head CI 전 merge하지 않는다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7255](https://github.com/edwardkim/rhwp/pull/7255) — fix: 감싼 칸의 빈 문단이 저장 슬롯만큼 자리를 지킨다 (#6925) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `3380ada6d92ad2bd818d9427ddcaf793fa54cc0e` |
| 규모 | 5 files, +125 / -1 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `3380ada6d92ad2bd818d9427ddcaf793fa54cc0e` | `075d11d16ebea23d58f0f63313692a98827c05a6` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35294778853) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35294778902) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35294778653) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35294778878) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35294778826) · [CI 6](https://github.com/edwardkim/rhwp/actions/runs/35294778703)

## 범위·조판 계약 검토

관련 이슈: [#6925](https://github.com/edwardkim/rhwp/issues/6925). HWP5 wrapper의 저장 전진이 빈 문단 슬롯과 일치하면 슬롯 높이를 보존한다.

stored_advance와 line_height+line_spacing의 일치 → stored_slot_exact → 빈 문단 유닛 높이 → 후속 내용 배치. 기존 접힌 슬롯과 HWPX 경로를 구분한다. 통합 #7243 중첩 패딩 대조군 2건도 통과했다.

주요 소비 경로: [src/renderer/layout/table_layout.rs](../../../src/renderer/layout/table_layout.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 실행한 검증과 한계

- 통합 제품의 `issue_6925_empty_cell_paragraph_keeps_stored_slot`: **1 test run: 1 passed, 194 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

1·2쪽을 PDF와 대조했다. 1쪽 제목·본문·표의 누적 위치가 base보다 개선됐다. 약 4px의 공통 원점 차이와 글꼴 차이는 남아 있다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| empty_slot | 1, 2 | 0 / 18.22% | 0 / 18.22% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| empty_slot p1 | [compare](../assets/pr7255_review/native_empty_slot_compare_001.png) · [overlay](../assets/pr7255_review/native_empty_slot_overlay_001.png) · [review](../assets/pr7255_review/native_empty_slot_review_001.png) | [compare](../assets/pr7255_review/wasm_empty_slot_compare_001.png) · [overlay](../assets/pr7255_review/wasm_empty_slot_overlay_001.png) · [review](../assets/pr7255_review/wasm_empty_slot_review_001.png) |
| empty_slot p2 | [compare](../assets/pr7255_review/native_empty_slot_compare_002.png) · [overlay](../assets/pr7255_review/native_empty_slot_overlay_002.png) · [review](../assets/pr7255_review/native_empty_slot_review_002.png) | [compare](../assets/pr7255_review/wasm_empty_slot_compare_002.png) · [overlay](../assets/pr7255_review/wasm_empty_slot_overlay_002.png) · [review](../assets/pr7255_review/wasm_empty_slot_review_002.png) |

기존 devel 제품 대조: [base empty_slot p1](../assets/pr7255_review/base_empty_slot_review_001.png).

## Merge 후 contributor PR comment 계획

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- empty_slot: 위 p1, p2의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7255_review/wasm_empty_slot_overlay_001.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#6925 전체 종료는 남은 공통 원점·다른 입력의 누적 오차 확인 후 판단한다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
