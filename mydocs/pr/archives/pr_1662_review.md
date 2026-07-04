# PR #1662 리뷰 기록

## 메타

| 항목 | 내용 |
|------|------|
| PR | #1662 |
| 제목 | C1b: 분산형(scatter) 차트 렌더링 (#1431 Track C) |
| 작성자 | johndoekim |
| base | devel |
| head | johndoekim:local/task1660 |
| 관련 이슈 | #1660, Refs #1431 |
| 규모 | 10 files, +791 / -17 |
| 최종 head SHA | `d9748845695257b5b46c4ab1197b498a05567763` |
| mergeable | merge 전 최종 확인값: `MERGEABLE`, `CLEAN` |
| CI 상태 | merge 전 최종 확인값: Build & Test, CodeQL, Render Diff/Canvas visual diff 통과 |
| merge commit | `922e69779c06afb937d2e7e3412dd366cda0489b` |
| 이슈 처리 | #1660 자동 close 실패 확인 후 수동 close/comment 완료. #1431은 상위 트래킹 이슈라 유지 |
| 처리 시각 | 2026-06-29 22:00 KST |

## 이슈 요약

#1660은 #1431 Track C의 하위 작업 C1b로, `samples/chart/분산형/`의 분산형 차트 5종이
`c:xVal`/`c:yVal` 미파싱과 `scatterChart` 미인식 때문에 "차트 (미지원)" placeholder로 렌더되던
문제를 다룬다. 목표는 픽셀 단위 parity가 아니라 5종 HWP/HWPX 10파일의 placeholder 제거와 합리적
분산형 렌더 커버리지 확보였다.

## 변경 범위

- `OoxmlChartType::Scatter`, `ScatterStyle`, `OoxmlSeries.x_values`, `OoxmlChart.scatter_style`을 추가했다.
- 파서가 `c:scatterChart`, `c:xVal`, `c:yVal`, `c:scatterStyle`을 인식하도록 확장했다.
- scatter의 X/Y `valAx`가 보조축으로 오분류되어 콤보 렌더 경로로 새지 않도록 축 분류 가드를 추가했다.
- 렌더러에 `render_scatter`를 추가해 2개 수치축, marker/line/lineMarker/smoothMarker 스타일, 소수 축 라벨을 렌더한다.
- `tests/issue_1431_scatter.rs`로 분산형 5종 × HWP/HWPX 10파일의 placeholder 회귀 가드를 추가했다.
- 구현 계획서, stage 문서, 최종 보고서를 `mydocs/**`에 포함했다.

## 검증

### GitHub Actions

최종 head `d9748845695257b5b46c4ab1197b498a05567763` 기준:

- Build & Test: success
- CodeQL preflight / Analyze rust / Analyze python / Analyze javascript-typescript / CodeQL: success
- Render Diff preflight / Canvas visual diff: success
- CI preflight: success
- WASM Build: skipped

### 로컬 확인

- 충돌 시뮬레이션: `upstream/devel` 기준 충돌 없음
- `git diff --check upstream/devel..HEAD`: 통과
- `cargo test --lib ooxml_chart::`: 통과, 32 passed
- `cargo test --test issue_1431_scatter`: 통과, 1 passed
- merge 후 `cargo test --test svg_snapshot`: 통과, 8 passed

## 리스크와 범위 밖

- 제목, 팔레트, 범례 위치, Y축 nice-scale headroom은 PR 보고서대로 C1c 공통 보정으로 이관한다.
- `smoothMarker`인 "곡선이있는"과 "곡선및표식" 샘플은 실제 chart XML이 동일해 같은 렌더로 처리한다.
- bubble, stock(HLC), 추세선, 세밀 스타일은 C2 이후 범위다.
- 이번 PR은 placeholder 제거와 합리적 렌더 커버리지가 목표이며, 픽셀 parity는 #1251 정책에 따라 1차 목표가 아니다.

## 최종 처리 결과

최종 head 기준 CI와 로컬 focused 검증이 통과했고, merge 전 `MERGEABLE` / `CLEAN` 상태를 확인했다.
#1662는 `922e69779c06afb937d2e7e3412dd366cda0489b`로 admin merge했다.

#1660은 `Refs`만 있어 자동 close되지 않았으므로 PR merge 후 수동 해결 코멘트를 남기고 close했다.
#1431은 상위 트래킹 이슈라 유지했다.
