# Task M100 #2277 단계별 완료보고서 — 3단계: 범례 순서 규칙 + 묶은가로 슬롯 반전

- 이슈: #2277 (C2a stock HLC 렌더 + 2D fidelity 정합)
- 브랜치: `local/task2277`
- 단계: 3/5
- 작성일: 2026-07-15

## 핵심 성과

C1c(#1882)에서 **"관찰 상충"으로 이관됐던 범례 순서 역전이 상충이 아니라 규칙**임을
28종 전수 실측으로 확정하고 구현했다:

> **역순 = (세로 값축 && 누적/백프로) || (가로막대 && 묶음)** — 계열이 플롯에서
> 세로 방향으로 배열되는 차트는 우측 세로 범례를 시각적 상→하 순서와 일치시킨다.
> 3D는 2D와 동일 규칙. pie(카테고리)/scatter/stock/콤보/이중축 = 정순.

부수 발견 반영: 묶은가로는 **플롯 슬롯 내 계열 배치도 계열1이 맨 아래**(현행은 맨 위)
— 범례만 뒤집으면 플롯과 어긋나므로 슬롯 반전을 같은 커밋에 포함.

## 정답지 PDF 28종 전수 실측표 (2026-07-15)

병렬 실측(파일당 독립 판독, 색상 픽셀 실측 기반). order: 범례 위→아래가 계열 1→N이면
정순(fwd), N→1이면 역순(rev).

| 종류 | order | 범례 위→아래 | 종류 | order | 범례 위→아래 |
|---|---|---|---|---|---|
| 묶은세로 | fwd | 1→2→3 | 꺽은선형 | fwd | 1→2→3 |
| **누적세로** | **rev** | 3→2→1 | **누적꺽은선형** | **rev** | 3→2→1 |
| **백프로누적세로** | **rev** | 3→2→1 | **백프로누적꺽은선형** | **rev** | 3→2→1 |
| 3D묶은세로 | fwd | 1→2→3 | 표식꺽은선형 | fwd | 1→2→3 |
| **3D누적세로** | **rev** | 3→2→1 | **표식누적꺽은선형** | **rev** | 3→2→1 |
| **묶은가로** | **rev** | 3→2→1 | 분산형 5종 전부 | fwd | Y1→Y2 |
| 누적가로 | fwd | 1→2→3 | 원형 5종 전부 | fwd | 1분기→4분기 |
| 백프로누적가로 | fwd | 1→2→3 | 고가저가종가 | fwd | 고가→저가→종가 |
| **3D묶은가로** | **rev** | 3→2→1 | 시가고가저가종가 | fwd | 시가→…→종가 |
| 3D누적가로 | fwd | 1→2→3 | 특이케이스(1계열) | single | 계열 1 |

**역순 8종 / 정순 19종 / 단일 1종 — 규칙 예외 0.** 예측했던 미확정 귀속
(백프로세로=역순·백프로가로=정순·3D=2D 동일)도 전건 실측 적중.

플롯 배치 교차 관찰 (규칙의 기하학적 근거): 누적류 스택 아래→위 = 계열1→3
(위=계열3=범례 첫 줄), 묶은가로 슬롯 위→아래 = 계열3→1, 묶은세로 슬롯 왼→오른쪽 =
계열1→3(가로 배열 → 정순), 누적가로 스택 왼→오른쪽 = 계열1→3(가로 배열 → 정순).

부수 확보 (4단계 근거 선실측): 무표식 라인 범례 스와치 = 짧은 굵은 **선분**(사각형
아님), 표식 라인 = **선분+마커 글리프**(◆■▲, 플롯 마커와 동일), 분산형 = 선분+글리프
또는 글리프, stock = 고가/저가/시가 **스와치 없음** + 종가만 글리프(▲/×) — 기존
실측과 일치, 전 파일 재확인됨.

## 구현 내용 (`src/ooxml_chart/renderer.rs`)

- **`legend_order_reversed(chart)` 단일 결정 함수** (규칙 표를 doc 주석으로 명문화):
  `legend_pos==Right && !콤보 && !이중축` 게이트 후 Column=누적/백프로,
  Bar=묶음, Line=line_grouping 누적/백프로일 때 true. 하단 가로 범례는 코퍼스
  미실측(전 샘플 legendPos=r)이라 현행 정순 유지 — 게이트에 명시.
- **`legend_items` 비-pie 분기**: 색 매핑(palette(si) 원 인덱스) 후 `items.reverse()` —
  하단/우측 범례 렌더 함수 양쪽이 이 함수를 소비하므로 변경 지점 1곳.
- **`render_bars` 묶은가로 슬롯 반전**: `bar_w * si` → `bar_w * (ser_count-1-si)` —
  계열1이 슬롯 맨 아래(실측 정합). 누적·세로 경로 무변경. 단일 계열(특이케이스)은
  no-op.

## 테스트 (RED→GREEN)

| 테스트 | 단언 | 종별 |
|--------|------|------|
| `test_legend_order_rule_table` | 9케이스 진리표 (Column/Bar/Line × 3 grouping) | 신규(유닛) |
| `test_legend_order_3d_same_as_2d` | 3D누적세로·3D묶은가로 역순 | 신규(유닛) |
| `test_legend_order_forward_for_stock_and_bottom_legend` | stock 정순 + 하단 범례 규칙 미적용 | 신규(유닛) |
| `test_legend_order_combo_forward` | 콤보 정순 고정 | 신규(유닛) |
| `test_hbar_clustered_slot_series1_at_bottom` | 묶은가로 슬롯: 파랑(계열1) y > 회색(계열3) y | 신규(유닛) |
| `tests/issue_2277_legend_order.rs` (2 테스트) | 역순 대표 4종 + 정순 대표 4종 × hwp/hwpx = 16파일 — 범례 라벨 y좌표 비교 | 신규(통합) |

RED (구현 전): `test_legend_order_rule_table`·`_3d_same_as_2d`·
`_hbar_clustered_slot`·통합 역순 테스트가 정확한 사유(전부 정순 렌더)로 실패
→ 84 passed; 3 failed (유닛) + 통합 1 failed. 정순 케이스는 무회귀 핀으로 선통과.

GREEN: `cargo test --lib ooxml_chart` **87/87**, 차트 통합 6스위트(신규 legend_order
포함) 전부 통과.

## 검증 결과

- [x] 유닛 87/87 + 차트 통합 6스위트 (issue_2277_stock/2129/1431_scatter/1882/1453 무회귀)
- [x] `issue_1882::chart_legend_on_right`(묶은세로 — 정순 유지·x좌표) 무회귀 확인
- [x] **전체 `cargo test` 전수 (클린 로그 실행): 265스위트 전부 ok — 3,178 passed /
  0 failed, cargo exit 0**
- [x] `cargo clippy --all-targets -- -D warnings` 무경고 (exit 0)
- [x] fmt: 수정 파일 rustfmt, `cargo fmt --check` 지적 0건

## 다음 단계

4단계 — 범례 스와치 글리프: `legend_items` 반환에 `SwatchKind` 도입
(Square/LineOnly/LineGlyph/GlyphOnly/Blank), `push_legend_swatch` 확장(—◆—),
별도 클래스 `hwp-legend-glyph`, stock 시/고/저 빈 스와치. 본 단계 전수 실측에서
스와치 형태 근거 확보 완료.
