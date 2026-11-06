# PR #8658 검토 — 표 칸 그림의 겹침 방지와 분할 행 소유

## 최종 판정

**머지 보류.** 원 사례 전 4쪽과 CENTER 대조군은 후보 Native에서 모두 raw 100%다. 다만 원본의 칸 높이만 그림 6장의 높이 합과 같게 한 경계에서 raw **83.83%**를 재현했다. 표 아래 괘선과 뒤 `END` 문단이 독립 한컴 Print와 어긋나며, 90% gate에 미달한다. base 45.59%보다 개선됐지만 기존 미해결을 면제하지 않는다. WASM Canvas 진단도 같은 경계에서 **83.81%**다. 최소 글꼴 집중 회귀 3건은 통과했으며, 원격 댓글·push·merge는 아직 승인받거나 실행하지 않았다.

## 접수와 검토 범위

- [원 PR #8658](https://github.com/edwardkim/rhwp/pull/8658), 기여자 `hyunmyungJaneLee`, 관련 [이슈 #8576](https://github.com/edwardkim/rhwp/issues/8576).
- 원 head: `218c8856ad0a338fd1820c43e62b40dbcdcb86bb`, fork `kidsnote/rhwp`, branch `task/8576-cell-square-no-overlap`.
- 고정한 최신 base: `08963f6825956984d38b0d9392f44777db7107ab`.
- 로컬 branch: `review/pr-8658-cell-square-no-overlap`. 원 기여 SHA를 보존한 검토 merge: `759f35f2fbccc2682356dc22cb31b1068b57e178`, tree `072cf20016469acdb8d332049fb4cfb589323632`. merge-tree와 실제 병합 모두 충돌이 없었다. 이 로컬 commit은 원격에 push하지 않았다.
- reviewer `edwardkim` 배정을 API로 확인했다. GitHub의 FIRST_TIME_CONTRIBUTOR 표시는 앞서 병합한 #8585·#8515·#8599·#8608·#8631·#8656의 실제 수용 이력과 구분한다.
- base route: `maintainer_general`. 보조 경로: `intake_and_review`, `local_validation`, `visual_fixture_evidence`, `post_merge`. 새 review의 판정 위치는 `review_template`을 따랐다.

## 선행 해결과 현재 재현

이슈 #8576의 본문과 원 PR의 본문·diff·리뷰·댓글·CI를 확인했다. 주장은 같은 문단의 allowOverlap=0 Square 그림을 아래로 쌓고, 분할 행의 저장 줄 없는 그림 문단을 해당 유닛 소유 조각에 그리는 것이다. 기여자의 비공개 코퍼스 집계는 참고 자료이며 독립 검증으로 사용하지 않는다.

직전 [통합 PR #8683](https://github.com/edwardkim/rhwp/pull/8683), merge `c80f355faa2bd810eda0724ec606027e7bbc019f`에 포함된 원 [#8599](https://github.com/edwardkim/rhwp/pull/8599), 원 commit `3221125b0b`와 [실제 소비 경로 기록](assets/hyunmyungjanelee-20261106/consumer-path-review.md)을 확인했다. 그 수정은 컷 없는 온전한 행의 control-only TAC 도형과 제한된 그림 원점을 다루며, 컷 있는 행의 그림 유닛 소유를 해결한 것으로 확대하지 않는다. #8579의 기존 뒤 본문 겹침에 대한 앞선 수용 예외는 이번 PR의 새 차이나 미검증을 면제하지 않는다.

최신 base를 실제 Native Skia 바이너리로 빌드하고 원본 3개를 같은 글꼴·DPI·profile로 실행했다.

| 원본 | 독립 Print | base Native 전쪽 raw | 실제 관측 |
| --- | --- | --- | --- |
| `cell-square-six-pictures-35x25.hwpx` | 한컴 2024, 1쪽 | 60.48% | 6장 모두 같은 (119.0, 137.9) 원점 |
| `cell-square-six-pictures-60x30-grows.hwpx` | 한컴 2024, 1쪽 | 34.98% | 6장 모두 같은 (119.0, 137.9) 원점 |
| `split-row-picture-only-para.hwp` | 한컴 2020, 2쪽 | 85.72% / 100.00% | 전체 2쪽의 그림 0장. Print는 1쪽에 2장 |

점수는 96dpi·색상 경계 보정 전 2px raw 실루엣 보조값이며 시각 완료 판정이 아니다. base의 미달을 후보 통과나 회귀 허용 근거로 사용하지 않는다.

## 후보의 개선과 재현된 보류 사유

후보 Native는 원 입력 3개 전 4쪽과 cell subList `vertAlign=TOP`만 `CENTER`로 바꾼 대조군 1쪽에서 모두 raw 100%다. 분할 행은 Print와 같은 1쪽 칸에 그림 2장을 복원했다. CENTER 대조군의 통과를 통해 정렬 경로의 우려 일부를 해소했지만 모든 배치의 일반성을 입증한 것으로 확대하지 않는다.

공개 원본의 `cellSz.height`만 `46772 → 42522`로 바꾼 `tests/fixtures/pr8658/cell-square-six-pictures-exact-fit.hwpx`를 추가 대조했다. `42522`는 원본 그림 6장의 선언 높이 합이며 그림·앵커·오프셋·여백·후속 본문은 바꾸지 않았다. 독립 한컴 2024 Print는 1쪽에서 표를 늘리고 뒤 `END`를 그 아래에 둔다. base와 후보는 같은 567.0px 높이의 표로 남는다. Print 괘선은 y=136.0107→706.1080px이고 후보 render tree는 y=136.0, h=567.0px다. 원본 안 여백은 위·아래 각각 141HU다. 이 경계의 실제 WASM Canvas도 83.81%이며 같은 괘선·후속 본문 차이를 보였다. 약 3.8px를 강제로 더하는 해법으로 정하지 않고, 내용 점유와 여백 및 후단 선언 높이 수용 경로를 추적해야 한다.

실행 결과는 base Native **45.59% → 후보 Native 83.83%**다. 원 PR 본문에 남긴 ‘쌓인 높이가 선언 높이와 같을 때 한컴이 칸을 더 늘리는 미지원’과 같은 범위이며 새 회귀 검출로 부르지 않는다. Native review/overlay와 같은 영역의 2배 확대에서 아래 괘선·다음 본문의 위치 차이를 직접 확인했다. 앞선 #8599의 기존 차이 예외를 이 경계에 옮겨 적용하지 않는다.

[보정·검증 계획](pr_8658_review_impl.md)에 승인 후 수행할 범위를 제시한다. 현재 reviewer가 원 head를 직접 고치거나 새 회귀를 추가한 상태는 아니다.

## 기준 출력과 검증 입력

원 PR에 커밋된 입력 3개와 웹한글기안기 Print PDF 3개를 보존한다. HWPX의 마지막 저장 제품 metadata는 2024, HWP의 metadata는 2010이다. 이 값은 합성 입력의 원 작성 제품 증명이 아니므로 생성 방식과 구분한다.

데스크톱 보완 Print는 다음과 같다. MCP start → status succeeded → download의 bytes·SHA-256 검증 및 실제 pdfinfo를 확인했다. endpoint·인증 정보는 기록하지 않는다.

- `pdf/issue8576/cell-square-six-pictures-35x25-2024.pdf`: 한컴 13.0.0.3901, engine 2024, 1쪽, 11,077 bytes, SHA-256 `5b13909893f3e1e7c49d531c75bceb8068832edf31a31d4d9af218e6047f2443`.
- `pdf/issue8576/cell-square-six-pictures-60x30-grows-2024.pdf`: 같은 한컴 2024, 1쪽, 11,774 bytes, SHA-256 `8ba22b151ae96ca9907d55125aa008813a5b52db50788279c3eff0cd10146713`.
- `pdf/issue8576/split-row-picture-only-para-2020.pdf`: 한컴 11.0.0.9136, engine 2020, 2쪽, 16,253 bytes, SHA-256 `dc16aa6065987b2e60a336b3a1309499652328dcbd383f3956a6ba6e2cfa7f8b`.

2024의 `pdf_print_method=0 / frame_print_to_pdf_ex_one_up`, 2020의 `pdf_print_method=0 / hancom2020_pdf_driver_one_up`을 확인했다. 기존 웹 출력과 Desktop 출력을 같은 자료라고 부르지 않는다. 원 글꼴 `hamchod-r.ttf`·`hamchob-r.ttf`를 Native에 명시했고 파일 자체는 커밋하지 않는다.

## 공통 조판 원칙 검토

| 항목 | 현재 근거·확인할 경계 | 상태 |
| --- | --- | --- |
| 원 증상·선행 해결 | #8576, #8599 → #8683 → 현재 base의 실제 3입력 재현 | 원 증상 확인 |
| 측정·실제 배치의 공통 결과 | `cell_no_overlap_square_stack`은 offset 상자의 HU 계산으로 `shifts/bottom`을 만들지만, paint의 `NoOverlapSquarePlacer::place`는 실제 x/y에서 별도로 겹침을 재계산한다. shifts 소비·정렬·반례를 대조 중 | 코드 검토상 우려 |
| 칸 세로 정렬 | 원본의 cell subList `vertAlign=TOP`만 `CENTER`로 바꾼 공개 합성 대조군을 독립 Print 대비 Native/Canvas 모두 100% | 해당 대조군 충족 |
| 분할·이어받기 | 빈 줄 소유와 control 유닛 소유의 합성, 실제 그림별 소유·요구/예약/배치 높이·마지막 유닛 후 본문 보존을 대조 중 | 미검증 |
| #8599 및 최신 devel 기능 보존 | 기존 원점 보정과 현재 partial 그림의 분기·소비 결과 비교 필요 | 미검증 |
| 신규 회귀의 시각 선행 조건 | 원 PR 3개 테스트의 최신 Native/fresh WASM 전쪽 증거·첫머리 측정 근거 규약 확인 필요 | 미검증 |
| backend·저장·자원 | 실제 Native/fresh WASM 및 필요한 경계 검증을 완료하기 전 비해당으로 바꾸지 않음 | 미검증 |
| baseline/golden | 원 PR에는 기존 baseline/golden 변경이 없음 | 비해당 |

## 실행과 CI

원 head `218c8856ad…`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/42148796576), CodeQL·Render Diff·Adapter·Proptest가 success다. 이는 새 base의 로컬 통합 후보 검증을 대신하지 않는다.

base Native 빌드와 3개 입력의 전쪽 Native Sweep를 실제로 완료했다. 후보 Native 빌드·전쪽 6쪽 비교와 실제 Canvas 진단·최소 글꼴 집중 회귀 3건을 완료했다. [Native 근거](assets/pr8658/native-review.md), [Canvas 진단·빌드 제한·집중 실행](assets/pr8658/wasm-diagnostic-and-focused.md), [검증 입력 10개 commit 일치](assets/pr8658/validated-inputs.tsv)를 따른다. 최상위 WASM 빌드 명령은 143으로 종료해 정식 빌드 완료로 처리하지 않았다. 전체 회귀·lint·Native 공식 3종·편람·base fresh WASM 등 수용 후보 제출용 최종 게이트는 보정 승인 뒤 완료해야 한다. 원격 조치는 아직 하지 않았다.

## Merge 후 contributor PR comment 계획

현재 최종 판정은 선언 높이 경계가 재현된 머지 보류다. Native와 Canvas 진단 증거는 확보했지만 90% 미달을 그대로 수용하거나 없는 최종 빌드 결과로 게시 계획을 채우지 않는다. 보정 후보의 전체 페이지 Native/fresh WASM TSV와 각 원 재현물의 review/overlay를 확보한 뒤 [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)에 맞춰 이 절을 보완한다. 댓글·push·PR 생성·merge는 별도 원격 승인 뒤 수행한다.
