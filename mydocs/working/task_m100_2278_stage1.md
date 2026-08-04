# Task M100 #2278 Stage 1 완료보고서 — 3D 막대 압출 (shade + push_bar_3d)

- 이슈: #2278 "C2b: 3D 입체·ofPie 보조플롯 렌더"
- 브랜치: `local/task2278`
- 구현계획서: `mydocs/plans/task_m100_2278_impl.md` 1단계
- 작성일: 2026-07-16

## 1. 정답지 정밀 판독 (구현 전 실측)

`pdf/chart/{세로막대형,가로막대형}/3차원{묶은,누적}*-2022.pdf` 4장 판독 결과.
이슈 #2278 사전 실측(2026-07-15)과 전 항목 정합.

| 항목 | 판독 결과 |
|---|---|
| 면 구성 | 정면 원색 / **우측(끝) 면 어둡게** / **윗면 밝게** — 4종 공통 |
| 압출 방향 | 우상(up-right) 사선, 약 45° |
| 압출 깊이 | 막대 두께의 약 40~50% (두꺼운 누적 막대도 깊이는 비례 상한 있음 — `(두께*0.45).clamp(3,9)` 근사와 부합) |
| 묶은형 가림 | 이웃 막대가 앞선 막대의 압출면을 부분 가림 — 길이(높이) 차이만큼만 노출. 단순 페인트 순서(세로: 왼→오른쪽, 가로: 아래→위)가 만드는 가림과 동일 형상 |
| 누적형 노출 | **세그먼트마다 자기 색의 측면(세로)/윗면(가로) 노출**, 캡(세로 윗면/가로 끝면)은 최종 세그먼트만 — "모든 세그먼트 3면 방출 + 페인트 순서 은면 제거" 설계와 일치 |
| 축 | 묶은세로/가로 0~5 step 1 무헤드룸, 누적세로 0~20 step 5(+1 step), 누적가로 0~14 step 2 — 기존 #1882 앵커와 완전 일치(변경 없음 재확인) |
| 범례 | 묶은세로·누적가로 정순 / 누적세로 역순 / 묶은가로 역순 — C2a #2277 stage3 규칙과 일치(변경 없음) |
| 기타 | 한컴은 플롯 배경에 3D 방(뒷벽·바닥 사선) 표현 있음 — 이슈 범위 외(막대 압출만), 미반영 |

음영 계수는 저해상도 판독 한계로 근사 시작: `BAR3D_TOP_SHADE = +0.25`(흰색 방향),
`BAR3D_SIDE_SHADE = -0.25`(검정 방향) — 시각판정에서 보정.

## 2. 구현 내용

`src/ooxml_chart/renderer.rs`만 변경 (파서·모델 무접촉).

1. **`shade(rgb, factor)`** — 채널별 선형 보간(+ = 흰색 방향, − = 검정 방향),
   상위(알파) 바이트 보존, factor ±1.0 클램프.
2. **`BAR3D_TOP_SHADE`/`BAR3D_SIDE_SHADE`** 상수 (±0.25 근사, 시각판정 보정 대상).
3. **`push_bar_3d(svg, x, y, w, h, depth, color)`** — 우상 45° 압출 3면:
   - top 평행사변형 `hwp-bar3d-top` (shade +): `(x,y) (x+d,y-d) (x+w+d,y-d) (x+w,y)`
   - right 평행사변형 `hwp-bar3d-side` (shade −): `(x+w,y) (x+w+d,y-d) (x+w+d,y+h-d) (x+w,y+h)`
   - front rect 원색 (무클래스 — 2D와 형태 통일)
   - `w/h ≤ 0` 조기 반환 — 누적 0값 세그먼트가 이웃 캡을 재도색하는 것 방지
   - 우상 압출에서 보이는 면은 세로/가로 막대가 동일 → 방향 플래그 없이 단일
     함수로 4조합(묶음/누적 × 세로/가로) 커버
4. **`render_bars` 통합** — `depth3d = |t| (t*0.45).clamp(3.0, 9.0)` 클로저 +
   rect 방출 4개소(stacked 가로/세로, clustered 가로/세로)를
   `if chart.is_3d { push_bar_3d(...) } else { 기존 rect }`로 분기.
   - 3D 분기 색은 `ser.color.unwrap_or_else(|| palette(si))` (u32) — 기존
     `series_color` String은 2D 분기 전용 유지 → **2D 출력 바이트 불변**
   - 축/range 블록·`render_value_grid`·`render_category_labels` **무접촉**
     (#1882 앵커 보호)
   - 누적 은면 제거는 기존 페인트 순서(아래→위/왼→오른쪽)가 담당 — 캡 노출
     분기 로직 불필요. 루프 순서 변경 금지 주석 명기

## 3. TDD 증적

- **RED**: 유닛 5종 + 통합 1종 선작성 → `cargo test --lib` 컴파일 에러
  (E0425 `shade` 미정의) 확인.
- **GREEN**: 구현 후 전부 통과.

신규 테스트:

| 테스트 | 검증 |
|---|---|
| `test_shade_lighten_darken` (유닛) | ±0.25 채널 수치(0x6183D7 → 0x89A2E1/0x4962A1), ±1.0 클램프, factor 0 항등, 알파 보존 |
| `test_bar3d_clustered_faces_both_orientations` (유닛) | Column·Bar 각 top/side 6개(2cat×3ser), 2D는 `hwp-bar3d-` 부재 |
| `test_bar3d_stacked_all_segments_extrude` (유닛) | 누적 top/side 각 6개 + 페인트 순서 핀(계열1 side 색이 계열3보다 선행) |
| `test_bar3d_zero_segment_skipped` (유닛) | 0값 세그먼트 면 무방출 (top 5개) |
| `test_bar3d_depth_clamp` (유닛) | top 폴리곤 x-delta — 넓은 플롯 9.0 / 좁은 플롯 3.0 |
| `bar3d_charts_emit_extrusion_faces_with_stable_axis` (통합, `tests/issue_2278_chart_3d_ofpie.rs` 신설) | 4종 × {hwpx,hwp}: top==side==12(3계열×4카테고리) + #1882 축 라벨 앵커(묶은 `>5<`유/`>6<`무, 누적세로 `>20<`, 누적가로 `>14<`/`>20<`무) |

## 4. 검증 결과

- 렌더러·파서 유닛(`cargo test --lib ooxml_chart`): **100 passed / 0 failed**
- 앵커 통합 재실행: issue_1453(2) / issue_1882(4) / issue_2277_stock(3) /
  issue_2277_legend_order(3) / issue_2277_mini_chart_axis(1) / issue_2278(1) —
  **전부 통과**
- `cargo test` 전체: **3,221 passed / 0 failed** (269 바이너리, ignored 1 —
  로그: `$TMPDIR/c2b_stage1_full_test.log`, 작업지시자 로그 관례 2026-07-16 채택)
- `cargo clippy --all-targets -- -D warnings`: 무경고
- 시각판정 산출물: `output/poc/chart_c2b/{hwp,hwpx}/` — 3D 막대 4종 × 2 = 8 SVG.
  자체 점검: 전 샘플 top/side 면 12/12, 표본 폴리곤 d=8.4(클램프 내),
  면 색 `#89a2e1`/`#4962a1` = shade(파랑, ±0.25) 정확

## 5. 위험 점검

- `data_bar_xs` 유닛 헬퍼(front rect 계수)는 2D 픽스처만 사용 — 3D front rect
  유입 없음(전체 유닛 통과로 확인).
- 페이지 전역 도형/WMF polygon과의 계수 충돌 — 통합 테스트를 `hwp-bar3d-*`
  클래스 기준으로 작성해 회피.
- top face의 플롯 상단 침범(묶은 무헤드룸 max값) — depth ≤ 9px, 시각판정에서
  확인 예정.

## 6. 전체 테스트 수치

```
$ grep -c "test result:" $TMPDIR/c2b_stage1_full_test.log   → 269
$ grep -oE "[0-9]+ passed" ... | awk '{s+=$1} END{print s}' → 3221 passed
$ grep -oE "[0-9]+ failed" ... | awk '{s+=$1} END{print s}' → 0 failed
$ grep -E "FAILED|failures:|error\[" ...                    → (없음)
```

C2a 종료 시점 3,191 대비 +30 (C2a 이후 devel 유입 + 본 단계 신규 6).
