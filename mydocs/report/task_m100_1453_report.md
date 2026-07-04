# Task M100 #1453 최종 결과보고서 — 3D막대·3D원형·ofPie 라우팅 + 막대 누적 보정

- 이슈: #1453 (상위 트래킹 #1431 Track C 첫 작업)
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1453`
- 작성일: 2026-06-21

## 1. 개요

OOXML 차트 27종 중 미지원 14종에서 **데이터가 이미 추출되던 7종**(3D막대4·3D원형1·ofPie2)을
요소명 라우팅만으로 렌더 전환했다(Part A). 시각판정 중 `render_bars`가 `c:grouping`을 무시해
누적/백프로 막대가 grouped로 왜곡됨을 발견, **막대 누적 보정**(Part B)을 작업지시자 승인으로 편입했다.

## 2. Part A — 요소명 라우팅 (7종)

- `src/ooxml_chart/parser.rs` `handle_start`: `bar3DChart→Column`(barDir로 Bar 세분),
  `pie3DChart`·`ofPieChart→Pie`. `handle_end` plot-종료 arm 확장. **렌더러·enum 무변경.**
- 결과: "차트 (미지원)" placeholder → 기존 막대/원형 렌더러로 **2D 근사** 렌더.
  (3D 입체·ofPie 보조플롯은 C2 후속.)

## 3. Part B — 막대 누적(stacked/percentStacked) 보정 (6종)

- `mod.rs`: `BarGrouping`(Clustered/Stacked/PercentStacked) enum + `OoxmlChart.grouping` 필드.
- `parser.rs`: `c:grouping` 파싱(막대 plot 한정, line은 C1d).
- `renderer.rs`: `render_bars` 누적/백프로 분기 + 값축(stacked=합계, percent=0~100%) +
  `render_value_grid` `percent` 파라미터. `category_positive_sum` 헬퍼.
- 대상 6종: 3D누적세로/가로 + **기존 2D 누적세로/가로·백프로세로/가로**(grouped 오렌더였던 것 동시 정합).

## 4. 검증

| 항목 | 결과 |
|------|------|
| 파서 단위 테스트 | 13건 (라우팅4 + grouping4 + 기존5) 통과 |
| 렌더러 기하 테스트 | 3건 (stacked=단일x / clustered=시리즈별x / percent=%축) 통과 |
| 통합 회귀 (`issue_1453`) | 라우팅 14파일 + 누적 6종, fallback 0 / percentStacked만 100%축 |
| 기존 차트 테스트 | `issue_1156`·`issue_1251` 회귀 없음 |
| 전체 스위트 | **2477 passed; 0 failed** (RC=0) |
| clippy `--all-targets` | 무경고 |
| 시각판정 | `output/poc/chart_c1a/` rhwp↔한컴2022 PDF — **데이터·기하·누적 정합**(작업지시자 확인). 잔여 차이=스타일 4갭(C1c) |

## 5. 작업 중 발견·등록

- **C1d (라인 누적 렌더)** — `render_line`도 `c:grouping` 미인식 → 누적꺽은선 3종 독립선 오렌더.
  #1431 Track C에 **신규 항목 등록**. (막대 누적과 대칭, 후속.)
- **#1456 (studio 캔버스 rawSvg 첫로드 공백)** — studio 캔버스 경로에서 차트=rawSvg op의
  비동기 디코드 재렌더 안전망이 rawSvg를 미커버(#1181 불완전 수정). **C1a 로직 회귀 아님**
  (native/SVG 경로 정상). **별도 이슈 등록.** → 차트 시각검증은 SVG 경로(`output/poc`)로 수행.

## 6. 인도물

- 계획: `mydocs/plans/task_m100_1453.md`, `_impl.md`
- 단계 보고: `mydocs/working/task_m100_1453_stage{1..6}.md`
- 최종 보고: 본 문서
- 소스: `src/ooxml_chart/{parser,renderer,mod}.rs`, `tests/issue_1453_chart_3d_ofpie_routing.rs`
- 시각판정: `output/poc/chart_c1a/` (+ `contactsheet.html`, gitignore)

## 7. 잔여/후속 (Track C)

- C1b scatter(5) · C1c 스타일 4갭 · **C1d 라인 누적**(신규) · C2 stock+3D입체+ofPie보조플롯 · **#1456 studio 캔버스 수정**

## 8. 커밋 (`local/task1453`)

계획서 → 1단계 라우팅 → 2단계 회귀테스트 → 3단계 시각판정 → 계획확장 → 4단계 grouping파싱
→ 5단계 누적렌더 → 6단계 누적가드. (`94932818` … `62f8c9c9`)
