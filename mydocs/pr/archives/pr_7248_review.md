---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7248 검토

## 최종 판정

**승인** — 2026-09-18 통합 검토.

잘못된 정렬값·배열 길이 검증, 기본 justify 유지, left/center/right/justify 전달과 저장 왕복 6건이 통과했다. #7245와 충돌한 builder 인자와 ID 할당을 함께 보존했다.

정렬 옵션 범위의 검토를 승인한다. 통합 보류 및 최종 head CI 통과는 별도 필수다.

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
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
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

## 실행한 검증과 한계

- 통합 제품의 `issue_7232_scaffold_cell_align`: **6 tests run: 6 passed, 193 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

left/justify 각 1쪽을 한컴 PDF와 대조했다. 정렬 선택이 반영되고 표 외곽·행 구성은 유지된다. 폰트 폭과 원점의 기존 차이는 전체 일치로 표현하지 않는다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## Visual Sweep 증적

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

현재 게시·merge 승인으로 간주하지 않는다. 보류 해제와 최종 head CI 및 통합 merge 이후 실제 merge SHA·통합 PR·CI URL·수정 계약·실제 검증 범위·잔여 차이를 한국어로 설명하고 기여에 감사한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.

- cell_left: 위 p1의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.
- cell_justify: 위 p1의 Native/WASM review와 **각 standalone overlay**를 코멘트에 실제 이미지로 포함한다.

URL 형식 예: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr7248_review/wasm_cell_left_overlay_001.png`. `<merge-commit-sha>`는 게시 전 실제 값으로 치환한다. UTF-8 파일과 `gh ... --body-file`로 게시하고 API 재조회로 한국어·실제 head·모든 이미지 URL을 확인한다.

## 이슈·다음 단계

#7232의 정렬 지정 수단 추가 범위는 종료 후보. 자동으로 모든 좁은 열의 문자를 재배치하는 기능은 아니다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
