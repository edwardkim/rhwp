# PR #1457 리뷰 — 3D막대·3D원형·ofPie 차트 라우팅 + 막대 누적 보정 (C1a)

- PR: https://github.com/edwardkim/rhwp/pull/1457
- 제목: `Task #1453: 3D막대·3D원형·ofPie 차트 라우팅 + 막대 누적 보정 (C1a)`
- 작성자: johndoekim (collaborator, 누적 15 PR)
- 연결 이슈: #1453 (closes), 트래킹 #1431 Track C
- base ← head: `devel` ← `johndoekim:task1453-chart-3d-ofpie-stacking`
- 작성일: 2026-06-22 (작성 시점 참고값: mergeable MERGEABLE / CLEAN)

## 1. 요약 판단

OOXML 차트 27종 중 데이터가 이미 추출되던 7종(3D막대4·3D원형1·ofPie2)을 요소명 라우팅으로
렌더 전환하고, 시각판정 중 발견한 막대 누적(stacked/percentStacked) 미반영을 함께 보정한 PR.
코드 품질이 높고 범위가 차트 모듈에 격리되어 있으며, 단위/통합 테스트가 충실하다.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/ooxml_chart/mod.rs` | `BarGrouping` enum(Clustered/Stacked/PercentStacked, Default=Clustered) + `OoxmlChart.grouping` 필드 |
| `src/ooxml_chart/parser.rs` | `bar3DChart`(barDir 재사용)·`pie3DChart`·`ofPieChart` 라우팅 + `c:grouping` 파싱(막대 plot 한정) + 단위 테스트 |
| `src/ooxml_chart/renderer.rs` | `render_bars` 누적/백분율 분기 + 값축 범위(stacked=합계, percent=0~100) |
| `tests/issue_1453_chart_3d_ofpie_routing.rs` | 통합 회귀 (라우팅 14파일·누적 6종) |
| `mydocs/plans·working·report/task_m100_1453*` | Hyper-Waterfall 문서 9건 |

## 3. 코드 검토

- **라우팅(Part A)**: 렌더러·enum 무변경. `bar3DChart`는 기존 `barDir` 핸들러를 재사용해
  Column↔Bar 후처리 확정. 3D 입체감·ofPie 보조플롯은 후속(C2)로 주석 명시. 깔끔한 최소 변경.
- **누적(Part B)**: `c:grouping`을 막대 plot에만 채택(line은 C1d 후속으로 분리). 렌더러 분기가
  음수 클램프(`.max(0.0)`), 양수 합(`category_positive_sum`), percent 0-나눗셈 가드 포함. 견고.
- **범위 격리**: `src/ooxml_chart/` 3파일 + 통합 테스트만. HWP3 룰·공통 모듈 침범 없음.

## 4. 사전 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build & Test / CodeQL / Analyze rust·js·python) | 전부 pass (작성 시점) |
| devel 충돌 시뮬레이션 (`merge-tree`) | 0건 |
| `cargo test --lib ooxml_chart` | 21 passed / 0 failed |
| `cargo test --test issue_1453_chart_3d_ofpie_routing` | 2 passed / 0 failed |
| 전체 `cargo test --profile release-test --tests` | lib 1902 passed / 0 failed, 통합 전부 0 failed |
| `cargo fmt --check` (수정 소스) | diff 없음 |
| `cargo clippy --lib` | 0 warning |

## 5. 시각 판정 관련

PR 설명은 macOS / 한컴 2022 PDF(`pdf/chart/`) 기준 데이터·기하·누적 정합을 서술. 비교 PNG는
`output/poc/`(gitignore)라 PR 미포함이나, 입력(`samples/chart/`)·정답지(`pdf/chart/`)가 저장소에
있어 재현 가능. 메모리 룰 `feedback_self_verification_not_hancom` 정합 — **머지 게이트는 자동
테스트이며, 한컴 편집기 직접 시각 판정은 Windows 환경에서 별도 확인 권장**(잔여 스타일 4갭은
후속 C1c로 분리 명시).

## 6. 판단

코드/테스트 품질 우수, 범위 격리, CI·로컬 검증 통과, 충돌 0건. merge 권장.
잔여 시각 정합(스타일 4갭)·3D 입체감·line 누적은 후속(C1c/C2/C1d)으로 명확히 분리되어 본 PR
범위가 적절하다.
