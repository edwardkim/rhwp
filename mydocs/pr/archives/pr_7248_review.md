---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7248 검토

## 최종 판정

**승인** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

잘못된 정렬값·배열 길이 검증, 기본 justify 유지, left/center/right/justify 전달과 저장 왕복 6건이 통과했다. #7245와 충돌한 builder 인자와 ID 할당을 함께 보존했다.

정렬 옵션 범위의 검토를 승인한다. 통합 보류는 해소됐으며 최종 head CI 통과는 별도 필수다.

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
| 원 PR | [#7248](https://github.com/edwardkim/rhwp/pull/7248) — 기능: scaffold 표에 셀 정렬 cell_align 을 연다 (#7232) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `050394e006d4e838d10c4c2d0e05220ad7846b66` |
| 규모 | 11 files, +458 / -9 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `050394e006d4e838d10c4c2d0e05220ad7846b66` | `c9f407ff4b2f5a218adc30c24d26e9e0aff0f2e3` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35271719822) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35271719800) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35271719444) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35271719763) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35271719836)

## 범위·조판 계약 검토

관련 이슈: [#7232](https://github.com/edwardkim/rhwp/issues/7232). scaffold table에 단일 또는 열별 cell_align을 추가하고 문단 모양을 생성한다.

JSON cell_align → 검증된 CellAlign → 열별 para_shape_id → 셀 문단 저장·렌더로 전달한다. ID 할당과 스타일 생성의 책임을 분리했다.

주요 소비 경로: [src/scaffold/builder.rs](../../../src/scaffold/builder.rs), [src/scaffold/mod.rs](../../../src/scaffold/mod.rs), [src/scaffold/schema.rs](../../../src/scaffold/schema.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 통합 제품의 `issue_7232_scaffold_cell_align`: **6 tests run: 6 passed, 193 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

left/justify 각 1쪽을 한컴 PDF와 대조했다. 정렬 선택이 반영되고 표 외곽·행 구성은 유지된다. 폰트 폭과 원점의 기존 차이는 전체 일치로 표현하지 않는다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 최초 검토 Visual Sweep 증적

CLI Native SVG와 fresh WASM SVG를 각각 Chrome webfont 경로로 캡처했다. 이것은 Native Skia raster나 Studio CanvasKit 화면 캡처가 아니다. `fidelity_compare --text-only --export-all-svg --layout-ledger` 전쪽 원장을 산출해 후보를 확인했다. 아래 자동 flag와 ink-match는 보조 지표이며 시각 승인 그 자체가 아니다.

| 입력 | 쪽 | Native flag / 평균 ink-match | WASM flag / 평균 ink-match |
| --- | --- | --- | --- |
| cell_left | 1 | 0 / 9.82% | 0 / 9.82% |
| cell_justify | 1 | 0 / 9.97% | 0 / 9.97% |

| 증적 | Native | fresh WASM |
| --- | --- | --- |
| cell_left p1 | [compare](../assets/pr7248_review/native_cell_left_compare_001.png) · [overlay](../assets/pr7248_review/native_cell_left_overlay_001.png) · [review](../assets/pr7248_review/native_cell_left_review_001.png) | [compare](../assets/pr7248_review/wasm_cell_left_compare_001.png) · [overlay](../assets/pr7248_review/wasm_cell_left_overlay_001.png) · [review](../assets/pr7248_review/wasm_cell_left_review_001.png) |
| cell_justify p1 | [compare](../assets/pr7248_review/native_cell_justify_compare_001.png) · [overlay](../assets/pr7248_review/native_cell_justify_overlay_001.png) · [review](../assets/pr7248_review/native_cell_justify_review_001.png) | [compare](../assets/pr7248_review/wasm_cell_justify_compare_001.png) · [overlay](../assets/pr7248_review/wasm_cell_justify_overlay_001.png) · [review](../assets/pr7248_review/wasm_cell_justify_review_001.png) |

## Merge 후 contributor PR comment 계획

승인된 통합 PR의 최종 head CI와 실제 merge를 확인한 뒤 원 PR·관련 이슈에 한국어로
merge SHA·CI URL·수정 계약·실제 검증 범위·남은 차이를 기록하고 기여에 감사한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.

- `mydocs/pr/assets/pr7248_review/`: `native_cell_left_review_001.png`, `native_cell_left_overlay_001.png`, `wasm_cell_left_review_001.png`, `wasm_cell_left_overlay_001.png`, `native_cell_justify_review_001.png`, `native_cell_justify_overlay_001.png`, `wasm_cell_justify_review_001.png`, `wasm_cell_justify_overlay_001.png`를 실제 이미지로 포함한다.

이미지는 `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<위-경로>`로 표시한다.
UTF-8 파일과 `--body-file`로 게시하고 한국어·실제 head·이미지 URL을 API와 HTTP로 재확인한다.

## 이슈·다음 단계

#7232의 정렬 지정 수단 추가 범위는 종료 후보. 자동으로 모든 좁은 열의 문자를 재배치하는 기능은 아니다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
