# Task M100 #2278 구현계획서 v2 — 3D 투영 모델 (Stage 1R·2 재설계)

- 이슈: #2278
- 브랜치: `local/task2278`
- 작성일: 2026-07-16
- 수행계획서: `mydocs/plans/task_m100_2278_v2.md` (v2 — 투영 모델 채택)
- Stage 3(ofPie)은 `task_m100_2278_impl.md` 3단계 설계를 그대로 사용 (2D — 투영 무관)

## 구현 개요

Stage 1R에서 `c:view3D`/`c:gapWidth`/`c:gapDepth`를 파싱하고 시어 투영기
(`ShearProj`, rAngAx=1)를 신설해 3D 막대 4종과 방을 재작성한다. v1~v3의 눈대중
상수(깊이 0.30/클램프/누적 0.4)를 전부 폐기하고 카메라·gapWidth·gapDepth
유도값으로 일원화한다. Stage 2는 rAngAx=0 회전+원근으로 3차원원형의 타원비를
sin(rotX)에서 유도한다.

**두께 규칙의 유도 (v3 상수 폐기 근거)**: Excel gapWidth 의미 = 슬롯 내 막대군
바깥 여백을 막대 폭의 %로 지정 → `bar_w = slot / (n_eff + gapWidth/100)`
(묶음 n_eff=계열수, 누적 n_eff=1). 코퍼스 gapWidth=150 대입: 누적
`1/(1+1.5)=0.4`(v3 실측과 일치), 묶은 3계열 `3/(3+1.5)=0.667`(관측 ~0.7과
일치). 관측이 규칙의 결과였음 — 상수 대신 규칙을 구현한다. 2D 경로는 0.7
휴리스틱 동결(바이트 불변).

**한컴 스펙 근거 (한글문서파일형식_차트 rev1.2, 2014 — VtChart 오브젝트 모델,
작업지시자 제공 2026-07-16)**: 본 설계의 투영 분류가 한컴 엔진의 실제 모드와
대응함을 확인.

| 스펙 항목 | 내용 | 본 설계 대응 |
|---|---|---|
| 표 100 ProjectionType=1 | "2.5차원 — 깊이는 있지만 회전·상승해도 **XY 면이 변화하지 않는다**" | rAngAx=1 시어(`ShearProj`) — 앞면 직립 유지의 문헌 근거 |
| ProjectionType=0 | 관점 뷰(기본, 원근 소실) | rAngAx=0 회전+원근 |
| ProjectionType=2 | 무관점 3D | perspective=0 평행 투영 극한 |
| 표 60 View3D{Elevation, Rotation} | 상승·회전 각도 | rotX/rotY 해석 검증 |
| 표 43 Pie.ThicknessRatio | 3D 파이 높이 = **반지름의 백분율** | Stage 2 측벽 높이 `rx × 비율` 구조 |
| Plot.DepthToHeightRatio | 차트 깊이 = 차트 **높이**의 백분율 | 씬 깊이 D의 대안 유도(플롯 치수 기반) — 시각판정 교체점 |
| Plot.BarGap | 범주 내 막대 공간 | gapWidth 슬롯 규칙과 동계 |

> 스펙 PDF는 `output/poc/`(gitignore)에 있음 — 장기 참조 필요 시 `mydocs/tech/`
> 이관은 작업지시자 결정 사항(한컴 저작권 고지: 무수정 원본 배포 허용).

---

## Stage 1R — view3D 파싱 + 시어 투영 + 3D 막대·방 재작성

**대상**: `src/ooxml_chart/{mod,parser,renderer}.rs`,
`tests/issue_2278_chart_3d_ofpie.rs`

### (a) 모델 (`mod.rs`)

```rust
/// `c:view3D` 3D 카메라 파라미터 (C2b #2278 v2). 투영 알고리즘은 ECMA-376
/// 비정의(구현 재량) — rAngAx=1은 시어, rAngAx=0은 회전+원근으로 해석.
#[derive(Debug, Clone, PartialEq)]
pub struct View3D {
    pub rot_x: f64,       // c:rotX (도, -90..90). 기본 15
    pub rot_y: f64,       // c:rotY (도, 0..360). 기본 20
    pub perspective: f64, // c:perspective (0..240). 기본 30 — rAngAx=1이면 미적용
    pub r_ang_ax: bool,   // c:rAngAx 직각 축. 기본 true
    pub h_percent: f64,   // c:hPercent. 기본 100 — 현 단계 미사용(기록만)
    pub depth_percent: f64, // c:depthPercent. 기본 100 — 막대 깊이/폭 비
}
impl Default for View3D { /* 15/20/30/true/100/100 */ }

pub struct OoxmlChart {
    …
    /// 3D plot의 c:view3D. bar3D/pie3D/line3D에서 Some(기본값으로 초기화 후
    /// 자식 요소가 덮어씀). (C2b #2278 v2)
    pub view3d: Option<View3D>,
    /// bar/bar3D plot의 c:gapWidth — 3D 두께 규칙 slot/(n+gap/100)용.
    /// stock의 up_down_gap_width와 별도 필드(상호 오염 방지). 기본 150.
    pub bar_gap_width: Option<f64>,
    /// bar3D plot의 c:gapDepth — 방 깊이 = 막대깊이×(1+gap/100). 기본 150.
    pub gap_depth: Option<f64>,
}
```

### (b) 파서 (`parser.rs`)

**⚠ 문서 순서 제약 (설계 리뷰 확정, 치명)**: ECMA-376 CT_Chart 시퀀스상
`c:view3D`는 `c:plotArea`보다 **앞**에 온다 (코퍼스 바이트 실측: view3D@908 <
plotArea@1059 < bar3DChart@1082). 따라서 view3d 초기화를 plot arm에 두면
rotX 등 6개 값이 전량 무음 폐기된다 — 막대 코퍼스 값(15/20/30/rAngAx=1)이
기본값과 동일해 막대에서는 무증상, **원형(rotX=30/rAngAx=0)에서만 발현**하는
은폐형. 초기화는 반드시 `view3D` 요소 자체에서:

```rust
b"view3D" => {
    // c:plotArea보다 먼저 온다(ECMA CT_Chart 시퀀스) — 여기서 초기화해야
    // 자식 rotX/rotY/…가 유실되지 않음. (C2b #2278 v2 설계 리뷰)
    chart.view3d = Some(View3D::default());
}
b"rotX" => { if let (Some(v3), Some(v)) = (chart.view3d.as_mut(), attr_f64(e)) { v3.rot_x = v; } }
// rotY/perspective/hPercent/depthPercent 동형, rAngAx는 val=="0"/"false" → false
```

- 3D plot arm(`bar3DChart`/`pie3DChart`/`line3DChart`)에는
  `chart.view3d.get_or_insert_with(View3D::default);` — view3D 요소가 없는
  3D 차트의 폴백(덮어쓰기 금지).
- `gapWidth` arm 확장(기존 Stock 게이트 유지 + 분기 추가):
  `Some(Column | Bar) => chart.bar_gap_width = Some(v)`
- `gapDepth` arm 신규: `cur_plot_type == Some(Column|Bar)` 게이트 →
  `chart.gap_depth = Some(v)`
- (헬퍼) `attr_f64(e)` = `attr_val(e, "val").and_then(parse)` — 기존 패턴 추출.
- View3D 기본값 15/20/30/true는 XSD 기본이 아니라 **Office(MS-OE376) 관례** —
  주석에 출처 명시.

### (c) 시어 투영기 + 막대·방 재작성 (`renderer.rs`)

**`ShearProj`** — rAngAx=1(직각 축): 앞면(z=0)은 직립 사각형 유지, 깊이만
화면 벡터 `(+sin(rotY), −sin(rotX))·D`로 밀림. 투영 bbox를 플롯 rect에
비등방 fit:

```rust
/// rAngAx=1 시어 투영 사전계산. 씬 = 앞면 플롯평면 × 깊이 [0..D].
/// fit: 앞면+깊이 오프셋의 bbox를 (px,py,pw,ph)에 비등방 스케일로 맞춤.
struct ShearProj {
    fx: f64, fy: f64, fw: f64, fh: f64, // fit 후 앞면 rect (모든 2D 배치 재사용 기준)
    dxf: f64, dyf: f64,                 // fit 후 z=D 화면 오프셋 (+우/+상)
}
fn shear_proj(view: &View3D, px: f64, py: f64, pw: f64, ph: f64, depth: f64) -> ShearProj {
    // 음수 시어 성분(rotX<0 하향, sin(rotY)<0 좌향)은 코퍼스 부재 + 페인트
    // 순서(아래→위/왼→오른쪽 은면 제거)와 상충 — 0 클램프 방어 근사.
    // 실샘플 확보 시 순회 방향 반전과 함께 확장. (C2b #2278 v2 설계 리뷰)
    let ox = depth * view.rot_y.to_radians().sin().max(0.0);
    let oy = depth * view.rot_x.to_radians().sin().max(0.0);
    let sx = pw / (pw + ox).max(1e-9);
    let sy = ph / (ph + oy).max(1e-9);
    ShearProj {
        fx: px,                 // ox ≥ 0 → 앞면은 좌측 고정
        fy: py + oy * sy,       // oy ≥ 0 → 앞면은 하단 고정(화면 y-down)
        fw: pw * sx,
        fh: ph * sy,
        dxf: ox * sx,
        dyf: oy * sy,
    }
}
```

시어 성분 클램프로 `push_bar_3d`는 **dx, dy ≥ 0 전제**(음수 분기 불필요 —
`debug_assert!(dx >= 0.0 && dy >= 0.0)`), 방 바닥·격자·라벨도 동일 전제로 단순화.

핵심 재사용 통찰: **fit 후 앞면 rect(fx,fy,fw,fh)에서 기존 2D 배치 수식을
그대로 실행**하고, 깊이는 상수 벡터 `(dxf, −dyf)`로 압출 — 축 계산(#1882)은
플롯 rect 기준 그대로, 배치·라벨만 앞면 rect로 치환.

**`render_bars` 3D 경로**:

```rust
if chart.is_3d {
    let view = chart.view3d.clone().unwrap_or_default();
    let gap_w = chart.bar_gap_width.unwrap_or(150.0).max(0.0);
    let n_eff = if stacked { 1.0 } else { ser_count as f64 };
    // fit 전 좌표로 두께·깊이 산출 (순환 없음)
    let slot0 = (if horizontal { ph } else { pw }) / cat_count as f64;
    let bar_w0 = slot0 / (n_eff + gap_w / 100.0);
    let b_depth = bar_w0 * view.depth_percent / 100.0;
    let d_scene = b_depth * (1.0 + chart.gap_depth.unwrap_or(150.0).max(0.0) / 100.0);
    let proj = shear_proj(&view, px, py, pw, ph, d_scene);
    // 막대는 깊이 방향 센터링: z ∈ [z0, z0+b], z0 = (D−b)/2 — Excel/한컴 관례
    // (gapDepth 여백을 앞뒤로 분할; 설계 리뷰 확정. 앞바닥 스트립이 보이는
    // 한컴 렌더와 정합 — 시각판정 확인 항목).
    // d_scene ≤ ε 퇴화(depthPercent=0 등) → 비율 0/0 NaN 방지 가드.
    let (bdx0, bdy0, bdx, bdy) = if d_scene > 1e-9 {
        let z0 = (d_scene - b_depth) / 2.0 / d_scene; // 시작 깊이 비율
        let zb = b_depth / d_scene;                   // 막대 깊이 비율
        (proj.dxf * z0, proj.dyf * z0, proj.dxf * zb, proj.dyf * zb)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };
    // 이후: 방 → 막대 배치(앞면 rect 기준: cat_span = fh|fw / cat_count,
    // bar_w = cat_span/(n_eff+gap_w/100), 묶음 그룹폭 = n_eff*bar_w;
    // 각 막대의 화면 원점에 (bdx0, −bdy0) 센터링 오프셋 가산) →
    // push_bar_3d(…, bdx, bdy, rgb) → render_category_labels(앞면 rect)
} else { /* 기존 2D — 무변경 */ }
```

- 2D와 3D의 배치 루프는 slot 수식만 다르므로(0.7 vs gapWidth 규칙) 3D 전용
  배치 루프를 분리(2D 루프 무접촉 — 바이트 불변 최우선). 누적/묶음 ×
  가로/세로 4조합의 페인트 순서는 v1과 동일(은면 제거) — 시어 클램프(dx,dy≥0)가
  이 순서의 유효 전제임을 주석 명기.
- **3D 묶음 막대는 2D의 `* 0.95` 폭 축소 계수 미적용** — 간격은 gapWidth
  규칙이 담당(계수 중복 방지, 명시적 결정).
- 3D front 면은 rAngAx=1에서 axis-aligned — **`<rect>`로 방출 유지**(기존
  테스트 헬퍼 `data_bar_xs`·front rect 파싱과 호환). `w/h ≤ 0` 조기 반환
  가드 유지(0값 세그먼트).

**`push_bar_3d` 재서명** — `(dx, dy)` 벡터화 + 부호 대응:

```rust
/// 3D 막대 면 방출 — 압출 벡터 (dx, -dy). dy>0: top 노출(윗면 밝게),
/// dy<0: bottom 노출. dx>0: right, dx<0: left (측면 어둡게).
/// |dx|,|dy| < 0.01 → front만(퇴화 카메라 무패닉). (C2b #2278 v2)
fn push_bar_3d(svg: &mut String, x: f64, y: f64, w: f64, h: f64, dx: f64, dy: f64, color: u32)
```

top(y면) 평행사변형: dy≥0이면 y 모서리 기준 `(x,y)(x+dx,y-dy)(x+w+dx,y-dy)(x+w,y)`,
dy<0이면 y+h 모서리. side(x면): dx≥0이면 x+w 모서리, dx<0이면 x 모서리 —
쉐이드는 `BAR3D_TOP_SHADE`/`BAR3D_SIDE_SHADE` 유지. 클래스
`hwp-bar3d-top`/`hwp-bar3d-side` 유지(테스트·통합 앵커 보존).

**`render_value_grid_3d` 재서명** — proj 기반 방:

- 뒷벽 rect = 앞면 rect + `(dxf, −dyf)` 평행이동 (axis-aligned 유지)
- 바닥 = 앞면 하단 모서리 ↔ 뒷벽 하단 모서리 사변형 (`#f2f2f2`)
- 값축 격자 = **바닥 조그**(앞 눈금 → 깊이 D) + **뒷벽 선** — 각각 별도
  **`<line>` 요소 2개로 방출**(`<polyline>`/`<path>` 미사용 — 기존 room
  테스트의 `<line` 계수·y1==y2 수평선 탐지 어휘 유지, 설계 리뷰 반영).
  조그가 d_scene(막대 깊이의 2.5배) 길이라 한컴의 "기울어진 격자" 자동 재현
- 라벨 = 앞면 rect 기준 기존 수식(문자열·포맷 불변 — #1882). 기존 코드의
  "라벨 위치 2D와 동일" 스테일 주석 2곳은 "문자열 동일·위치는 앞면 rect
  기준"으로 갱신
- `hwp-bar3d-room` 그룹 유지

**폐기**: `d3`/`bar_frac` 3D 분기(0.30·2.5~9 클램프·0.4 상수),
`depth3d` 잔재 — 유도값으로 대체.

**rAngAx=0 막대(코퍼스 부재)**: 시어 폴백(kx=sin(rotY), ky=sin(rotX)) + 주석
문서화 — 진짜 회전 투영은 Stage 2의 원형(rAngAx=0 실샘플)에서 구현·검증 후
필요 시 후속에서 막대에 확장.

### (d) 테스트

파서 단위:
- `test_parse_view3d_fields` — **문서 순서 필수 재현**: 픽스처는 코퍼스와
  동일하게 `view3D`가 plot 요소보다 **앞**에 오고, 값은 **기본값과 다른
  원형 코퍼스 값**(rAngAx=0/rotX=30/rotY=0)을 사용 — 막대 값(15/20)으로
  쓰면 초기화 순서 버그가 Some(default)==실측으로 공허 통과(설계 리뷰 확정)
- `test_parse_view3d_defaults` — view3D 요소 없는 bar3D → 15/20/30/true/100/100
  (plot arm `get_or_insert_with` 폴백 검증)
- `test_parse_bar_gap_width_and_depth` — bar3D의 gapWidth/gapDepth 저장 +
  stock `up_down_gap_width` 무간섭(기존 테스트 재확인)

렌더러 단위 — **기존 3D 테스트 9종 처분 표 (설계 리뷰 반영)**:

| 기존 테스트 | 처분 | 사유/대체 |
|---|---|---|
| `test_bar3d_clustered_faces_both_orientations` | 유지 | 클래스 카운트 — 투영 무관 |
| `test_bar3d_stacked_all_segments_extrude` | 유지 | 카운트+페인트 순서 핀 |
| `test_bar3d_zero_segment_skipped` | 유지 | 0값 스킵 — 투영 무관 |
| `test_bar3d_depth_clamp` | **삭제** | 클램프 규칙 자체 폐기 → `test_bar3d_room_depth_ratio`+`test_bar3d_thickness_from_gap_width`가 대체 |
| `test_bar3d_room_only_when_3d` | 유지 | `<line>` 어휘 유지 전제(위 (c) 명시) |
| `test_bar3d_room_grid_on_back_wall` | 재작성 | 단언 형태 유지, 좌표를 fit 기준(fx+dxf)으로 |
| `test_bar3d_stacked_thinner_bar` | **흡수** | `test_bar3d_thickness_from_gap_width`로 대체(0.4 수치 동일 + gapWidth=300 변형 + 2D 대조군 0.7 유지) |
| `test_axis_3d_clustered_no_headroom` | 무수정 유지 | 라벨 문자열 앵커 |
| `test_axis_3d_stacked_vertical_extra_headroom` | 무수정 유지 | 라벨 문자열 앵커 |

신규:
- `test_bar3d_shear_direction` — top 폴리곤 압출 벡터 `dy/dx` ==
  `(sin15°/sin20°) × (sy/sx)` — **비등방 fit 배율을 테스트 내에서 재계산**해
  기대값 구성(±1e-6). 순수 sin비 ±1e-3 단언은 fit과 모순(설계 리뷰 확정)
- `test_bar3d_thickness_from_gap_width` — 누적 3D 두께/슬롯 = 1/(1+1.5)=0.4,
  묶은 3계열 그룹/슬롯 = 3/4.5, gapWidth=300 → 0.25. 2D 대조군 0.7
- `test_bar3d_faces_within_plot` — 모든 `hwp-bar3d-*` 좌표가 플롯 rect
  내부(±0.5) — fit 검증
- `test_bar3d_room_depth_ratio` — 뒷벽 오프셋(dxf)/막대 압출(bdx) ==
  1 + gapDepth/100 (기본 2.5 — 센터링과 무관하게 성립: dxf/bdx = D/b)
- `test_bar3d_bars_depth_centered` — 막대 front 원점이 앞면 rect 대비
  `(bdx0, −bdy0)` 오프셋(센터링) 반영
- `test_bar3d_degenerate_cameras` — rotX=0·rotY=0·rotX=90·**rotY=200°(sin<0
  → 클램프 0)·rotX=−15(클램프 0)**·depthPercent=0(NaN 가드) — 무패닉 + front 방출

통합 (`issue_2278_chart_3d_ofpie.rs`): 기존 단언 유지(면 12쌍·방 1회·#1882
라벨) — 투영 전환에도 불변이어야 함.

**완료 기준**: 유닛+차트 통합·앵커 GREEN → `cargo test` 전체(단계 게이트,
`$TMPDIR/c2b_stage1r_full_test.log`) + clippy → 8종 재산출 →
`task_m100_2278_stage1r.md` + 커밋 → **시각판정** (보정은 모델 파라미터로만).

## Stage 2 — 3D 원형 (rAngAx=0 회전+원근)

**대상**: `renderer.rs` (`render_pie_3d` 신설), 통합 테스트 확장

(a) 정밀 판독: `pdf/chart/원형/3차원원형-2022.pdf` — 타원비(예측 sin30°+원근
≈ 0.5~0.55)·측벽 높이·중심 위치 실측으로 유도식 검증.

(b) 타원비 유도: 원판(y=0 평면, 반지름 r)을 rotX로 기울여 원근 투영 —
`ry/rx = sin(rot_x)`, perspective>0이면 근소 보정(원근 나눗셈: 카메라 거리
`d0 = r / tan(fov/2)`, `fov = perspective/2 도` 해석 — 캘리브레이션 1점).
`rotY`는 시작각 오프셋. 측벽 높이 = `rx × PIE_THICKNESS_RATIO` — 한컴 스펙
`Pie.ThicknessRatio`(반지름의 백분율) 구조 채택, 비율값은 정답지 판독으로
캘리브레이션 (hPercent 반영 여지 기록).

(c) 렌더: v1 계획의 측벽 알고리즘 유지(하반부 θ∈(0,π) 클립·벽 선행·
`hwp-pie3d-wall`/`hwp-pie3d-top`) — 타원비만 상수 0.55 → `sin(rot_x)` 유도.
훅: Pie 분기 `is_3d` → `render_pie_3d(svg, chart, view, …)`.

(d) 테스트: `test_pie3d_ellipse_ratio_follows_rotx` — rotX=30 → ry/rx≈0.5,
rotX=60 픽스처 → ≈0.866 (유도 확인). 벽 클립·순서·2D 가드는 v1 설계 그대로.
통합: 3차원원형 wall≥1·top==4.

**완료 기준**: v1 Stage 2와 동일 + 시각판정.

## Stage 3 — ofPie + 팔레트 #5 (v1 구현계획서 그대로)

`task_m100_2278_impl.md` 3단계 설계 무변경 적용 (2D — 투영 무관).

---

## 변경 파일 예상

| 파일 | 변경 |
|---|---|
| `src/ooxml_chart/mod.rs` | `View3D` 신설, `view3d`/`bar_gap_width`/`gap_depth` 필드 (1R) |
| `src/ooxml_chart/parser.rs` | view3D arm 6 + gapWidth 확장 + gapDepth (1R) |
| `src/ooxml_chart/renderer.rs` | `ShearProj`·`push_bar_3d`(dx,dy)·`render_value_grid_3d` 재작성, 3D 배치 루프 분리, v3 상수 폐기 (1R) / `render_pie_3d` (2) / ofPie·팔레트 (3) |
| `tests/issue_2278_chart_3d_ofpie.rs` | 유지 + stage2/3 파트 추가 |
| `mydocs/working/task_m100_2278_stage{1r,2,3}.md` | 단계 보고서 |

## 위험 / 주의

- **화면 y-down 부호**: 시어 (+dx, −dy)에서 dy>0 = 화면 위 — top/side 면
  선택·방 오프셋·격자 조그 전부 동일 부호 규약 사용. 부호 단위 테스트
  (`shear_direction`, `room_depth_ratio`)로 핀. 음수 성분은 클램프로 정의역
  차단(위 (c)) — 페인트 순서 반전 미구현 상태에서 음수각 유입 방지.
- **depthPercent 해석은 의도적 편차**: ECMA 정의는 "차트 폭 대비 %"이나 본
  설계는 "막대 폭 대비 %"로 사용 — 막대 스케일 비례 유지 목적의 캘리브레이션
  결정(스펙 유도 아님). 코드 주석·보고서에 편차 명시 (설계 리뷰 반영).
- **2D 바이트 불변**: 3D 배치 루프를 2D 루프에서 분리 — 2D 경로 코드 무접촉.
  대조군 테스트(0.7·무면·무방) 유지.
- **#1882**: 축 계산 입력(플롯 rect) 불변, 라벨 문자열 불변. 라벨 위치는
  앞면 rect로 이동하나 앵커는 문자열만 검사 — issue_1882 재실행으로 입증.
- **fit 비등방**: 시어 각이 미세 왜곡되나(sx≠sy) 코퍼스 카메라에선 오차 작음.
  시각판정에서 거슬리면 등방 fit(min(sx,sy)+정렬)으로 전환 — 모델 내 교체점.
- **gapWidth 공유 요소**: Stock 게이트 유지 + Column/Bar 분기 추가 — 기존
  stock 테스트로 무간섭 입증.
- **perspective 해석**(Stage 2): `fov = perspective/2 도`는 관례 기반 근사 —
  rotX 유도 타원비가 지배 항이라 잔차 작음. 판독으로 확정.
- 통합 face-count 12쌍 앵커는 투영 전환과 무관하게 유지되어야 함 — 전환
  직후 즉시 재실행.
- 기능 변경만 — fmt 수정 파일 한정, 저장 경로 무접점.
