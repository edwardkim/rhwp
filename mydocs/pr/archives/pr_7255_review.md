---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7255 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

대상 1쪽의 누적 위쪽 밀림이 개선되고 독립 PDF 위치를 검사하는 focused가 통과했다. #7253의 첫 노드 검사 충돌은 빈 슬롯을 보존하는 공유 좌표 보정으로 해소했다.

대상 빈 슬롯 보존 변경과 기존 입력 재사용 보정을 수용한다. 최종 원격 head CI 전 merge하지 않는다.

## 메인터너 보정: 기존 입력·기준 PDF 재사용

최종 입력 점검에서 신규 `samples/issue6925/148751598_paragraph_spacing_drift.hwp`가
기존 [samples/issue6924/148751598-briefing.hwp](../../../samples/issue6924/148751598-briefing.hwp)와
동일 Git blob임을 확인했다. 검사를 기존 경로로 통일하고 중복 HWP 및 그 경로만을 위한
IR baseline 두 행을 제거한다. 기존 경로의 검사와 baseline은 보존한다.
HWP SHA-256은 `03c93b021e01652b1ca5ba3a4a301decf9da33484af7d088987327efcb59e610`로 같다.

기준도 이미 검증된 [pdf/148751598-briefing-2020.pdf](../../../pdf/148751598-briefing-2020.pdf)를
재사용한다. Creator `Hwp 2022 0.0.0.0`, PDF 1.6, 6쪽,
SHA-256 `98ce52ec0a6ed25ba73070b22c743cb72d129113456ae3019f417d1b53ee9dd3`.
PDF 버전·Creator 연도로 배제하지 않고 동일 원문과 정상 출력임을 확인했다.
새 이름의 추가 PDF는 최종 diff에서 제거한다. 아래 최초 비교는 당시 PDF 기록이며
최종 comment는 다음 기존 기준의 최신 compare·review·overlay를 사용한다.

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 1 | [compare](../assets/pr7255_review/maintainer_dedup_20260918/native_compare_001.png) · [overlay](../assets/pr7255_review/maintainer_dedup_20260918/native_overlay_001.png) · [review](../assets/pr7255_review/maintainer_dedup_20260918/native_review_001.png) | [compare](../assets/pr7255_review/maintainer_dedup_20260918/wasm_compare_001.png) · [overlay](../assets/pr7255_review/maintainer_dedup_20260918/wasm_overlay_001.png) · [review](../assets/pr7255_review/maintainer_dedup_20260918/wasm_review_001.png) |
| 2 | [compare](../assets/pr7255_review/maintainer_dedup_20260918/native_compare_002.png) · [overlay](../assets/pr7255_review/maintainer_dedup_20260918/native_overlay_002.png) · [review](../assets/pr7255_review/maintainer_dedup_20260918/native_review_002.png) | [compare](../assets/pr7255_review/maintainer_dedup_20260918/wasm_compare_002.png) · [overlay](../assets/pr7255_review/maintainer_dedup_20260918/wasm_overlay_002.png) · [review](../assets/pr7255_review/maintainer_dedup_20260918/wasm_review_002.png) |

재사용 기준의 p1·p2를 직접 판독했고 Native/WASM 2/2 PNG가 동일하다. 빈 슬롯 보존의 개선과 기존 글꼴·공통 원점 차이를 구분한다. 입력 경로 변경 후 #6925 1/1, 기존 #6924 대조 2/2, IR baseline 4/4 통과(exit 0). IR 실행의 한 검사에는 nextest `leaky` 표지가 있었으며 실패로 종료되지 않았다. 불필요한 중복 입력을 제거했고 기존 경로의 검사는 유지했다.

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다. 통합 원격 최종 head의
CI·mergeability 확인은 아직 남아 있으며 완료 전 merge하지 않는다.

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
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
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

## 최초 검토 검증과 한계

- 통합 제품의 `issue_6925_empty_cell_paragraph_keeps_stored_slot`: **1 test run: 1 passed, 194 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

1·2쪽을 PDF와 대조했다. 1쪽 제목·본문·표의 누적 위치가 base보다 개선됐다. 약 4px의 공통 원점 차이와 글꼴 차이는 남아 있다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 검토 Visual Sweep 증적

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

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7255_review/maintainer_dedup_20260918/`: `native_review_001.png`, `native_overlay_001.png`, `wasm_review_001.png`, `wasm_overlay_001.png`, `native_review_002.png`, `native_overlay_002.png`, `wasm_review_002.png`, `wasm_overlay_002.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#6925 전체 종료는 남은 공통 원점·다른 입력의 누적 오차 확인 후 판단한다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
