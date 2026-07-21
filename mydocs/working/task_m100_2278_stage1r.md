# Task M100 #2278 Stage 1R 완료보고서 — view3D 파싱 + 시어 투영 3D 막대·방

- 이슈: #2278 "C2b: 3D 입체·ofPie 보조플롯 렌더"
- 브랜치: `local/task2278`
- 계획: `task_m100_2278_v2.md` (수행 v2) / `task_m100_2278_impl_v2.md` (구현 v2)
- 선행: Stage 1 v1~v3 (오블리크 고정 근사 — 본 단계에서 투영 모델로 대체)
- 작성일: 2026-07-17

## 1. 구현 내용

### 모델 (`mod.rs`)

- `View3D { rot_x, rot_y, perspective, r_ang_ax, h_percent, depth_percent }` 신설
  — 기본값 15/20/30/true/100/100 (XSD 기본이 아닌 Office(MS-OE376) 관례, 주석 명시)
- `OoxmlChart.view3d: Option<View3D>` / `bar_gap_width` / `gap_depth` 필드 추가
- `depth_percent`는 "막대 깊이/폭 %"로 해석 — ECMA("차트 폭 대비 %")와 다른
  **의도적 편차** 주석 명시 (설계 리뷰 반영)
- 모듈 doc·`is_3d` 스테일 주석 갱신

### 파서 (`parser.rs`)

- **`b"view3D"` arm에서 초기화** — `c:view3D`가 `c:plotArea`보다 앞이라(바이트
  실측 908 < 1059) plot arm 초기화는 값 전량 폐기(설계 리뷰 치명 발견). 3D plot
  arm(`bar3DChart`/`pie3DChart`/`line3DChart`)은 `get_or_insert_with` 폴백만
- 자식 arm 6개(rotX/rotY/perspective/rAngAx/hPercent/depthPercent) +
  `attr_f64` 헬퍼
- `gapWidth` arm 확장(Stock 게이트 유지 + Column/Bar → `bar_gap_width`),
  `gapDepth` arm 신규(Column/Bar 게이트)

### 렌더러 (`renderer.rs`)

- **`ShearProj`/`shear_proj`**: rAngAx=1 시어 투영(한컴 차트 스펙 rev1.2 표 100
  ProjectionType=1 "2.5차원"과 동계) — 화면 깊이 벡터 `(+sin(rotY), −sin(rotX))·D`,
  비등방 fit으로 플롯 rect에 맞춤. **음수 시어 성분은 0 클램프**(페인트 순서
  은면 제거와 상충 — 정의역 방어, 설계 리뷰 반영)
- **`render_bars_3d`** 신설 — 3D 전용 경로 분리(2D 루프 무접촉·바이트 불변):
  - 두께 = `slot/(n_eff + gapWidth/100)` — v1~v3 눈대중 상수(누적 0.4·묶은 0.7·
    깊이 클램프)의 유도 원형으로 전부 대체. 2D는 0.7 휴리스틱 동결
  - 막대 깊이 = `bar_w × depthPercent/100`, 방 깊이 = `× (1+gapDepth/100)`
  - **깊이 센터링**: z ∈ [(D−b)/2, (D+b)/2] — gapDepth 여백 앞뒤 분할
    (Excel/한컴 관례). `d_scene ≤ ε` 퇴화 NaN 가드
  - 3D 묶음은 2D의 0.95 폭 계수 미적용(gapWidth 규칙이 간격 담당)
- **`push_bar_3d`** 벡터 서명 `(dx, dy)` — dx,dy ≥ 0 전제(debug_assert),
  압출 퇴화(<0.01) 시 front만
- **`render_value_grid_3d`** 재작성 — proj 기반: 뒷벽(z=D)·바닥·눈금별 바닥
  조그(`<line>` 2개, polyline 미사용 — room 테스트 어휘 유지)·앞면 축선.
  라벨 문자열·포맷 불변(#1882), 위치는 fit 후 앞면 rect 기준
- `render_bars`는 축 범위 확정 후 `is_3d`면 `render_bars_3d`로 조기 분기 —
  2D 경로는 stage1 이전 형태로 원복(방출 코드 무분기)

### 효과 (한컴 정합 관점)

- 값축 격자의 "기울어진" 인상 = 바닥 조그(방 깊이 2.5×막대 깊이) 자동 재현
- 가로/세로·묶음/누적별 두께가 gapWidth 규칙에서 자동 유도 — 계열별 상수 소멸
- 임의 카메라(rotX/rotY/gapDepth/depthPercent 변형)에 연속적으로 동작

## 2. TDD 증적

- 파서: 신규 3종 선작성 → 컴파일 RED → 구현 → 36/36 GREEN.
  `test_parse_view3d_fields`는 **문서 순서(view3D가 plotArea 앞) + 비기본값
  (원형 코퍼스 rAngAx=0/rotX=30/rotY=0)** 픽스처 — plot arm 초기화 퇴행 시
  기본값≠실측으로 즉시 RED (공허 통과 차단, 설계 리뷰 반영)
- 렌더러: 신규 6종 + 처분(depth_clamp **삭제**, stacked_thinner_bar
  **thickness_from_gap_width로 흡수**) 적용. 초회 4 FAIL은 `{:.2}` SVG 좌표
  반올림 대비 과도한 허용오차(1e-6)가 원인 — 반올림 현실(±0.005/좌표)에 맞게
  1e-3~2e-2로 보정 후 GREEN

| 신규 테스트 | 검증 |
|---|---|
| `test_parse_view3d_fields` | 문서 순서 + 비기본값 6필드 |
| `test_parse_view3d_defaults_when_absent` | view3D 부재 3D → Office 관례 기본값, 2D는 None |
| `test_parse_bar_gap_width_and_depth` | bar3D gapWidth/gapDepth + stock 필드 무오염 |
| `test_bar3d_shear_direction` | fit 역산(pw=fw+dxf) pre-fit 성분비 == sin15/sin20 + 막대 압출 ∥ 방 깊이 |
| `test_bar3d_room_depth_ratio` | dxf/bdx == 1+gapDepth/100 (150→2.5, 300→4.0) |
| `test_bar3d_thickness_from_gap_width` | 누적 0.4 / 묶은 1/4.5 / gapWidth300 0.25 / 2D 대조군 0.7 |
| `test_bar3d_bars_depth_centered` | 센터링 오프셋 == (dyf−bdy)/2 |
| `test_bar3d_faces_within_plot` | 4조합 전 면 좌표 bbox 내 (fit 스모크) |
| `test_bar3d_degenerate_cameras` | rotX=0/90/−15·rotY=0/200°·depthPercent=0 — 무패닉·무NaN·front 존재 |

유지(투영 무관): 면 카운트 3종·room 2종·axis_3d 라벨 앵커 2종.

## 3. 검증 결과

- ooxml_chart 유닛: **110 passed / 0 failed** (fmt 후 재확인)
- 통합·앵커: issue_2278(1)/1453(2)/1882(4)/2277 3종(10)/2129(3)/1431_scatter(1)
  — **전부 통과** (면 12쌍·방 1회·#1882 라벨이 투영 전환에도 불변 입증)
- `cargo clippy --all-targets -- -D warnings`: 무경고. fmt 수정 파일만
- `cargo test` 전체: **3,231 passed / 0 failed** (269 바이너리 — 로그:
  `$TMPDIR/c2b_stage1r_full_test.log`)
- 시각판정 산출물: `output/poc/chart_c2b/{hwp,hwpx}/` 8종 재산출 —
  전 샘플 room 1·top 12·NaN 0 확인

## 4. 위험·잔여

- 시어 배율·fit이 시각적으로 과/부족하면 보정은 **투영 모델 파라미터로만**
  (계열별 상수 금지 — v2 가드레일)
- rAngAx=0 막대는 시어 폴백(코퍼스 부재, 주석 문서화) — 진짜 회전 투영은
  Stage 2 원형에서 도입
- 음수각 카메라는 클램프 방어 — 실샘플 확보 시 페인트 순서 반전과 함께 확장
