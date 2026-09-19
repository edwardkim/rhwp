---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7249 검토

## 최종 판정

**승인** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

독립 한컴 HWP/HWPX 변환본과 비교하는 4건이 통과했다. 길이 -1의 두 워드와 above/below 슬롯을 대조했다. 실제 미주가 표시되는 HWP3 문서의 최종 조판은 미검증이다.

저장 레코드 변환 범위는 승인한다. 미주 배치 및 다른 구분선 길이 종류의 미검증을 그대로 남긴다.

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다.
[통합 PR #7264](https://github.com/edwardkim/rhwp/pull/7264)의 code candidate
`8228249fcfdf04cb7c47af9b3f5d5447a645d3b9` CI를 모두 확인했다.
[원격 CI 증적](pr_7244_review_impl.md#통합-pr-7264-code-candidate-ci)의 동일 PR 실행이며,
같은 PR의 trailing review·오늘할일 head aggregate와 mergeability를 확인한 뒤 merge한다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7249](https://github.com/edwardkim/rhwp/pull/7249) — 수정: HWP3→HWP5 저장의 각주·미주 모양을 한/글 변환본과 맞춘다 (#7174) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `7497b8e59d557e84d0d5cd1ee05e1a9ab77bba8b` |
| 규모 | 5 files, +297 / -63 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `7497b8e59d557e84d0d5cd1ee05e1a9ab77bba8b` | `fe2d59443a9f5aac93be5c6c21ae5650dc5f7d5d` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35276155560) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35276155659) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35276155087) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35276155527) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35276155547)

## 범위·조판 계약 검토

관련 이슈: [#7174](https://github.com/edwardkim/rhwp/issues/7174). HWP3 각주 구분선 sentinel과 미주 기본 여백을 HWP5 저장 슬롯에 정확히 매핑한다.

HWP3 문서정보 → FootnoteShape 두 워드 및 HWP5 여백 슬롯 → serializer → reparsed HWP/HWPX를 확인했다. 렌더 결과만 맞추는 값 조정이 아니라 저장 형식 계약을 고친다.

주요 소비 경로: [src/parser/hwp3/mod.rs](../../../src/parser/hwp3/mod.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 통합 제품의 `issue_7174_hwp3_note_shape_contract`: **4 tests run: 4 passed, 209 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

SO-SUEOP 1~3쪽은 일반 출력 대조군이다. 각주/미주의 시각 일치 증거로 사용하지 않는다. 핵심 근거는 커밋된 Hancom 2020 변환 HWP/HWPX의 이름 있는 속성 및 바이너리 필드다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 검토 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| hwp3_note | 1, 2, 3 | 0 / 17.91% | 0 / 17.91% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| hwp3_note p1 | [compare](../assets/pr7249_review/native_hwp3_note_compare_001.png) · [overlay](../assets/pr7249_review/native_hwp3_note_overlay_001.png) · [review](../assets/pr7249_review/native_hwp3_note_review_001.png) | [compare](../assets/pr7249_review/wasm_hwp3_note_compare_001.png) · [overlay](../assets/pr7249_review/wasm_hwp3_note_overlay_001.png) · [review](../assets/pr7249_review/wasm_hwp3_note_review_001.png) |
| hwp3_note p2 | [compare](../assets/pr7249_review/native_hwp3_note_compare_002.png) · [overlay](../assets/pr7249_review/native_hwp3_note_overlay_002.png) · [review](../assets/pr7249_review/native_hwp3_note_review_002.png) | [compare](../assets/pr7249_review/wasm_hwp3_note_compare_002.png) · [overlay](../assets/pr7249_review/wasm_hwp3_note_overlay_002.png) · [review](../assets/pr7249_review/wasm_hwp3_note_review_002.png) |
| hwp3_note p3 | [compare](../assets/pr7249_review/native_hwp3_note_compare_003.png) · [overlay](../assets/pr7249_review/native_hwp3_note_overlay_003.png) · [review](../assets/pr7249_review/native_hwp3_note_review_003.png) | [compare](../assets/pr7249_review/wasm_hwp3_note_compare_003.png) · [overlay](../assets/pr7249_review/wasm_hwp3_note_overlay_003.png) · [review](../assets/pr7249_review/wasm_hwp3_note_review_003.png) |

## Merge 후 contributor PR comment 계획

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7249_review/`: `native_hwp3_note_review_001.png`, `native_hwp3_note_overlay_001.png`, `wasm_hwp3_note_review_001.png`, `wasm_hwp3_note_overlay_001.png`, `native_hwp3_note_review_002.png`, `native_hwp3_note_overlay_002.png`, `wasm_hwp3_note_review_002.png`, `wasm_hwp3_note_overlay_002.png`, `native_hwp3_note_review_003.png`, `native_hwp3_note_overlay_003.png`, `wasm_hwp3_note_review_003.png`, `wasm_hwp3_note_overlay_003.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#7174 전체에는 번호 장식·본문 텍스트 축도 있다. 이 PR의 sentinel/여백 검사만으로 전체 이슈를 닫지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
