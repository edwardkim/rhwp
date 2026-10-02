# PR #7377 self-review — #7333 꼬리말·도형·연결선

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#7377](https://github.com/edwardkim/rhwp/pull/7377) |
| 작성자·reviewer | `jangster77` collaborator self-review |
| base / head | `devel` / `task_m100_7333_footer_shapes_20260923` |
| 검토 head | `3a78dbe0c2322ef54e0a648ce0e5ff55f3cb4d8b` |
| 관련 이슈 | [#7333](https://github.com/edwardkim/rhwp/issues/7333), 잔여 기능 [#7375](https://github.com/edwardkim/rhwp/issues/7375) |
| 작성 시점 원격 상태 | `MERGEABLE`, `BLOCKED`; 최신 head CI 완료 전 참고값 |

이 PR은 `aaaaaa.hwp`의 꼬리말·도형·표 안 그림·연결선 배치를 한컴 2020 PDF와 대조해 보정한다.
8쪽을 포함한 연결선은 실제 선 경로를 넓은 컨테이너의 bbox보다 먼저 hit-test하고, 중첩 표의
`cellPath`와 선택 쪽을 보존한다. 클릭은 z-order를 바꾸지 않으며, 실제 드래그 뒤 Ctrl+Z와 Command+Z가
원래 offset으로 되돌린다.

## 입력과 기준

| 역할 | 저장소 경로 | SHA-256 | 확인 범위 |
| --- | --- | --- | --- |
| 원본 HWP | `samples/issue7333/aaaaaa.hwp` | `ab88150db2eb4652c945e6773ed40f61806782b6ca85694eb5f8c8ec390f96c5` | 50쪽 전체 |
| 한컴 기준 PDF | `pdf/issue7333/aaaaaa-2020.pdf` | `fbc0ff34b909e8824e775872ff47ac7d6c904d2cb74504fbe9e7a8411b2c597b` | 50쪽 전체 |
| RHWP export PDF | `pdf/issue7333/aaaaaa-rhwp-stage11.pdf` | `9278b2d9a65e291d42b96868200872fcddb3ea1f97461cb551dca5ddf088106b` | 50쪽 직접 PDF 대조 |

원본은 `hancom-office-2018` 저장 정보라 2020 MCP engine을 사용했다. 기준 PDF는 `Creator: Hwp 2020 0.0.0.0`,
PDF 1.4, 50쪽이며 형식 버전만으로 유효성을 제한하지 않았다. 세 파일은 모두 검토 head에 포함되어 있다.

## 완료한 검증

- `npm run e2e:issue-7333-line` 통과. headless Studio에서 8쪽 `para=141`, `control=1` 화살표를 실제 클릭하고,
  드래그 뒤 Ctrl+Z와 Command+Z 각각으로 원래 `horzOffset`·`vertOffset` 복귀를 확인했다.
- `node --test tests/picture-hit-policy.test.ts tests/undo-drag-click-routing.test.ts tests/shortcut-map.test.ts`:
  **23 passed**.
- `npm run build` 통과.
- 최종 Studio 선택 보정 전 renderer candidate의 전체 nextest는 **10,169 passed, 50 skipped**였다.
  최종 세 커밋은 Studio frontend/E2E만 변경했고, 작업지시자 지시에 따라 Rust 회귀를 다시 실행하지 않았다.
- Native 전체 visual sweep은 50/50쪽 완료, 누락 0, structural flag 0. 평균 pixel match 95.826%,
  2px 이웃 관용 내용 실루엣 99.402%였다. 사람 검토로 19·41·43쪽의 연결선 방향·marker와 표 셀 경로를 확인했다.

## 공통 조판 원칙 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족 | 저장 줄·footer·marker 경로는 실문서와 회귀 사례로 한정했고, 8쪽만의 분기를 두지 않았다. |
| 측정·배치 일관성 | 충족 | 저장 줄 소유·footer·SVG affine/marker의 생산·소비 경로를 함께 수정했다. |
| 분할·이어받기 계약 | 충족 | 전대역 표 뒤 흐름과 다음 문단을 14·22·23·33쪽 visual sweep으로 확인했다. |
| 줄 소속과 점유 높이 | 충족 | 40~47쪽 셀 그림이 control 소유 `LINE_SEG`를 따르게 회귀로 고정했다. |
| 사례와 증거의 독립성 | 충족 | HWP·한컴 PDF·Native/fresh WASM·RHWP PDF export를 구분해 보존했다. |
| 기준값 변경 | 충족 | off-canvas baseline 한 건은 한컴 기준 134쪽 직접 overlay로 확인했다. |
| 주장과 검증 범위 | 충족 | renderer visual sweep과 Studio 8쪽 실제 클릭·드래그·Undo E2E를 분리해 기록했다. |

## Visual Sweep 증적

PR 본문은 검토 head raw URL로 Native 및 fresh WASM 대표 review·overlay를 직접 표시한다. 최종 자산은
`mydocs/pr/assets/issue_7333_footer_shapes/`에 보존했으며, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.

- Native p19: `native_review_019.png`, `native_overlay_019.png`
- Native p41: `native_review_041.png`, `native_overlay_041.png`
- fresh WASM p14: `wasm_review_014.png`, `wasm_overlay_014.png`

## 잔여 범위

사각 도형의 곡률 지정은 RHWP가 저장·렌더링하지 않는 기능이다. 직각으로 표시되는 네모/원 숫자 차이를
이번 위치·선 방향 보정으로 숨기지 않고 [#7375](https://github.com/edwardkim/rhwp/issues/7375)로 분리했다.

## 최종 판정

**승인.** 현재 head의 코드·입력·시각 증적·Studio E2E를 확인했다. merge 전에는 최신 PR head의 CI 성공과
`MERGEABLE`/`CLEAN` 재확인 및 작업지시자 승인이 필요하다.

## Merge 후 contributor PR comment 계획

merge SHA와 CI URL이 확정된 뒤 #7377에 UTF-8 `--body-file`로 한국어 comment를 게시한다. comment에는
해결한 저장 줄·footer·연결선 계약, 실제 검증 범위, #7375 잔여 곡률 기능을 구분하고,
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.
다음 raw URL을 `<merge-commit-sha>`로 고정해 실제 이미지로 표시한 뒤 API 재조회로 본문·이미지·head를 확인한다.

- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7333_footer_shapes/native_review_019.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7333_footer_shapes/native_overlay_019.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7333_footer_shapes/native_review_041.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7333_footer_shapes/native_overlay_041.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7333_footer_shapes/wasm_review_014.png`
- `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7333_footer_shapes/wasm_overlay_014.png`
