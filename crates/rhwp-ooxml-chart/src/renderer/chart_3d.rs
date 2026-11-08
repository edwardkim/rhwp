//! 3D 막대의 장면·축·면을 같은 좌표계에서 구성한다.

use super::*;

const BAR3D_TOP_SHADE: f64 = -0.25;
const BAR3D_SIDE_SHADE: f64 = -0.5;

/// 투영 전 데이터 면과 글줄 예약 후 viewport를 구분한다.
/// 범주 글줄 회전은 viewport만 줄이며 같은 데이터 면의 종횡비를 바꾸지 않는다.
pub(super) struct SceneFrame {
    world_width: f64,
    world_height: f64,
    viewport: [f64; 4],
    category_left_advance: Option<(f64, usize)>,
    category_count: usize,
}

impl SceneFrame {
    pub(super) fn new(
        chart: &OoxmlChart,
        [width, height]: [f64; 2],
        [plot_width, plot_height]: [f64; 2],
        [title_height, legend_height, legend_width]: [f64; 3],
        has_title: bool,
        category: CategoryLabelLayout,
        scales: cartesian_layout::ChartScales,
        measure: &dyn Fn(&str, f64, &str) -> f64,
    ) -> Option<Self> {
        let view = chart.view3d.as_ref()?;
        // 계열 깊이 축·원근·반대쪽 카메라는 기존 경로를 보존하며 자동 2.5D와 혼동하지 않는다.
        if !chart.is_3d
            || !matches!(
                chart.chart_type,
                OoxmlChartType::Column | OoxmlChartType::Bar
            )
            || !view.r_ang_ax
            || view.rot_x.to_radians().sin() < 0.0
            || view.rot_y.to_radians().sin() < 0.0
            || chart
                .axes
                .iter()
                .any(|axis| axis.kind == crate::AxisKind::Series)
        {
            return None;
        }
        let horizontal = chart.chart_type == OoxmlChartType::Bar;
        let category_count = chart.categories.len().max(
            chart
                .series
                .iter()
                .map(|series| series.values.len())
                .max()
                .unwrap_or(0),
        );
        if category_count == 0 {
            return None;
        }
        let font = label_font_px(chart);
        let number_width = scales.primary.label_width(chart, 0, measure);
        let world_title = if has_title {
            FRAME_PAD * 2.0 + title_font_px(chart) * crate::legend_layout::TEXT_LINE_EM + font * 0.5
        } else {
            title_height
        };
        let (world_width, world_height, viewport) = if horizontal {
            // 가로 값축의 마지막 숫자 반 폭과 세로 값축용 반 줄은 표시 여백이다.
            // 이를 데이터 면에서도 빼면 긴 가로 차트의 종횡비와 등방 fit이 함께 줄어든다.
            // 실제 숫자·범주 공간은 CartesianLayout과 viewport가 계속 예약한다.
            (
                plot_width + number_width * 0.5,
                plot_height + font * 0.5,
                [
                    TICK_GAP * 0.5,
                    TICK_GAP * 0.5,
                    plot_width - TICK_GAP * 2.0,
                    plot_height - TICK_GAP * 0.5 - ROOM_TICK_DOWN,
                ],
            )
        } else {
            (
                width - legend_width - number_width - FRAME_PAD * 4.0,
                height - world_title - legend_height - cartesian_layout::category_text_bottom(font),
                [
                    TICK_GAP,
                    TICK_GAP * 0.5,
                    plot_width - TICK_GAP - FRAME_PAD,
                    plot_height - TICK_GAP,
                ],
            )
        };
        if world_width <= 0.0 || world_height <= 0.0 || viewport[2] <= 0.0 || viewport[3] <= 0.0 {
            return None;
        }
        Some(Self {
            world_width,
            world_height,
            viewport,
            category_count,
            category_left_advance: if !horizontal && category.rotation_degrees > 0.0 {
                Some((
                    category.maximum_advance * category.rotation_degrees.to_radians().cos(),
                    category_count,
                ))
            } else {
                None
            },
        })
    }

    fn depth(&self, horizontal: bool) -> f64 {
        let span = if horizontal {
            self.world_height
        } else {
            self.world_width
        };
        span / self.category_count as f64 * 0.75
    }

    pub(super) fn projected_size(&self, chart: &OoxmlChart) -> [f64; 2] {
        let view = chart
            .view3d
            .as_ref()
            .expect("3D 장면은 카메라를 확인한 뒤 생성한다");
        let projection = self.project(
            view,
            0.0,
            0.0,
            self.depth(chart.chart_type == OoxmlChartType::Bar),
        );
        [projection.fw, projection.fh]
    }

    fn project(&self, view: &View3D, x: f64, y: f64, depth: f64) -> ShearProj {
        let ox = depth * view.rot_y.to_radians().sin();
        let oy = depth * view.rot_x.to_radians().sin();
        let [vx, vy, vw, vh] = self.viewport;
        let mut scale = (vw / (self.world_width + ox)).min(vh / (self.world_height + oy));
        let mut left_overhang = 0.0;
        if let Some((advance, count)) = self.category_left_advance {
            let first_center = self.world_width / (count as f64 * 2.0);
            // 끝 정렬한 회전 글줄의 시작은 첫 범주 중심보다 왼쪽으로 나간다.
            // 고정 글자 폭과 투영되는 데이터 폭을 분리해 같은 fit에서 수용한다.
            if advance > first_center * scale {
                let label_fit = (vw - advance) / (self.world_width + ox - first_center);
                if label_fit > 0.0 {
                    scale = scale.min(label_fit);
                    left_overhang = (advance - first_center * scale).max(0.0);
                }
            }
        }
        ShearProj {
            fx: x + vx + (vw - (self.world_width + ox) * scale + left_overhang) * 0.5,
            fy: y + vy + (vh - (self.world_height + oy) * scale) * 0.5 + oy * scale,
            fw: self.world_width * scale,
            fh: self.world_height * scale,
            dxf: ox * scale,
            dyf: oy * scale,
        }
    }
}

/// rAngAx=1(직각 축 — 한컴 차트 스펙 rev1.2 표 100 ProjectionType=1
/// "2.5차원: 회전·상승해도 XY 면 불변"과 동계) 시어 투영 사전계산.
/// 씬 = 앞면 플롯평면 × 깊이 [0..D], 화면 깊이 벡터 = (+sin(rotY), −sin(rotX))·D.
/// 투영 bbox를 플롯 rect에 비등방 fit — 앞면은 좌하 고정, 뒷벽이 우상으로.
/// rAngAx=0 막대는 코퍼스 부재 — 동일 시어 폴백(근사, 진짜 회전 투영은 Stage 2
/// 원형에서 rot_x 유도로 도입). (C2b #2278 v2)
struct ShearProj {
    /// fit 후 앞면(z=0) rect — 3D 배치 수식의 기준
    fx: f64,
    fy: f64,
    fw: f64,
    fh: f64,
    /// fit 후 z=D 화면 오프셋 (+우/+상)
    dxf: f64,
    dyf: f64,
}

fn shear_proj(view: &View3D, px: f64, py: f64, pw: f64, ph: f64, depth: f64) -> ShearProj {
    // 음수 시어 성분(rotX<0 하향, sin(rotY)<0 좌향)은 코퍼스 부재 + 페인트
    // 순서(아래→위/왼→오른쪽 은면 제거)와 상충 — 0 클램프 방어 근사.
    // 실샘플 확보 시 순회 방향 반전과 함께 확장. (v2 설계 리뷰 확정)
    let ox = (depth * view.rot_y.to_radians().sin()).max(0.0);
    let oy = (depth * view.rot_x.to_radians().sin()).max(0.0);
    let sx = pw / (pw + ox).max(1e-9);
    let sy = ph / (ph + oy).max(1e-9);
    ShearProj {
        fx: px,           // ox ≥ 0 → 앞면 좌측 고정
        fy: py + oy * sy, // oy ≥ 0 → 앞면 하단 고정 (화면 y-down: 상단 여백 = oy·sy)
        fw: pw * sx,
        fh: ph * sy,
        dxf: ox * sx,
        dyf: oy * sy,
    }
}

/// 3D 막대 1개(또는 누적 세그먼트 1개) — 압출 벡터 (+dx, −dy) 3면:
/// 위·옆 평행사변형(앞면보다 어둡게)과 앞 사각형(원색)을 그린다.
/// dx, dy ≥ 0 전제(shear_proj 클램프 — 음수 성분은 페인트 순서 은면 제거와
/// 상충하여 정의역 밖). 은면 제거는 호출측 페인트 순서(누적: 아래→위/왼→오른쪽)가
/// 담당하므로 루프 순서 변경 금지. w/h ≤ 0(0값 세그먼트)이면 무방출 — 누적에서
/// 이웃 세그먼트의 캡 재도색 방지. 압출 퇴화(dx,dy < 0.01)면 front만. (C2b #2278 v2)
fn push_bar_3d(svg: &mut String, x: f64, y: f64, w: f64, h: f64, dx: f64, dy: f64, color: u32) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    debug_assert!(dx >= 0.0 && dy >= 0.0, "시어 성분은 클램프로 비음수");
    if dx > 0.01 || dy > 0.01 {
        svg.push_str(&format!(
            "<polygon class=\"hwp-bar3d-top\" points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\"/>\n",
            x, y, x + dx, y - dy, x + w + dx, y - dy, x + w, y,
            color_hex(shade(color, BAR3D_TOP_SHADE))
        ));
        svg.push_str(&format!(
            "<polygon class=\"hwp-bar3d-side\" points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"{}\"/>\n",
            x + w, y, x + w + dx, y - dy, x + w + dx, y + h - dy, x + w, y + h,
            color_hex(shade(color, BAR3D_SIDE_SHADE))
        ));
    }
    svg.push_str(&format!(
        "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/>\n",
        x,
        y,
        w,
        h,
        color_hex(color)
    ));
}

/// 3D 방 선 스타일 — 정답지 임베드(2702px) 픽셀 실측: 축선·조그·격자·틱 전부
/// #808080 균일(gray 126~148)·0.75pt=1px. 2D의 축/격자 명암 구분(#e8e8e8)과
/// 다른 3D 전용 실측 규칙 (시각판정 피드백 2026-07-19).
const ROOM_LINE_STYLE: &str = "stroke=\"#808080\" stroke-width=\"1\"";
/// 값·카테고리 좌측 틱 길이(pt) — 실측 44/38px ÷ 8.34px/pt ≈ 5.3/4.6
const ROOM_TICK_LEFT: f64 = 5.0 * PX_PER_PT;
/// 하단 틱 길이(pt) — 실측 31~37px ≈ 3.7~4.4
const ROOM_TICK_DOWN: f64 = 4.0 * PX_PER_PT;

/// 3D 방 + 값축 격자·라벨 — 시어 투영 기반: 뒷벽(z=D, 격자 포함) + 눈금별
/// 바닥 조그 + 바닥 평행사변형 + 값·카테고리 틱. 격자는 `<line>` 2개(조그+뒷벽)로
/// 방출(`<polyline>` 미사용 — room 테스트 어휘 유지). 라벨 문자열·포맷은
/// render_value_grid와 동일(#1882 라벨 앵커) — 위치는 fit 후 앞면 rect 기준.
/// 한컴 실측(2026-07-19): 벽 테두리·바닥 채움 없음, 바닥 외곽선·틱은 격자와
/// 동일 스타일. 2D 경로는 이 함수를 쓰지 않는다. (C2b #2278 v2)
#[allow(clippy::too_many_arguments)]
fn render_value_grid_3d(
    svg: &mut String,
    font: f64,
    proj: &ShearProj,
    vmin: f64,
    vmax: f64,
    step: f64,
    format_code: Option<&str>,
    horizontal: bool,
    percent: bool,
    cat_count: usize,
) {
    let (fx, fy, fw, fh) = (proj.fx, proj.fy, proj.fw, proj.fh);
    let (dxf, dyf) = (proj.dxf, proj.dyf);
    // 방 표면: 뒷벽(흰 면, 무테두리 — 한컴: 벽 모서리선 없음) → 바닥(흰 면 +
    // #808080 외곽선; 뒷모서리·좌측 대각은 0 눈금 격자/조그와 겹침) → 앞면 좌측 축선
    svg.push_str(&format!(
        "<g class=\"hwp-bar3d-room\">\n<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"#ffffff\"/>\n",
        fx + dxf,
        fy - dyf,
        fw,
        fh
    ));
    svg.push_str(&format!(
        "<polygon points=\"{:.2},{:.2} {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}\" fill=\"#ffffff\" {ROOM_LINE_STYLE}/>\n",
        fx,
        fy + fh,
        fx + dxf,
        fy + fh - dyf,
        fx + fw + dxf,
        fy + fh - dyf,
        fx + fw,
        fy + fh
    ));
    svg.push_str(&format!(
        "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
        fx,
        fy,
        fx,
        fy + fh
    ));

    // 라벨 문자열 규칙은 render_value_grid와 동일 (#1882 라벨 앵커 — 위치는
    // 앞면 rect 기준)
    let decimal = (step - step.round()).abs() > 1e-9;
    let label = |v: f64| format_grid_value(v, format_code, percent, decimal);
    let span = (vmax - vmin).max(1e-9);
    let step = if step > 0.0 { step } else { span / 5.0 };
    let grid_lines = ((span / step) + 1e-9).floor().max(0.0) as usize;
    for i in 0..=grid_lines {
        let t = (step * i as f64) / span;
        let v = vmin + step * i as f64;
        if horizontal {
            let gx = fx + fw * t;
            // 바닥 조그(앞 눈금 → 뒷벽, 깊이 D) + 뒷벽 세로 격자 + 하단 값 틱
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                gx,
                fy + fh,
                gx + dxf,
                fy + fh - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                gx + dxf,
                fy + fh - dyf,
                gx + dxf,
                fy - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                gx,
                fy + fh,
                gx,
                fy + fh + ROOM_TICK_DOWN
            ));
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"inherit\" font-size=\"{:.2}\" fill=\"{}\" text-anchor=\"middle\" dominant-baseline=\"text-after-edge\">{}</text>\n",
                gx,
                // 눈금 끝·라벨 간격·글줄 상자를 차례로 예약한다.
                // 범주 글줄의 아래끝을 쓰면 3D 가로 값축 숫자가 눈금 쪽으로 당겨진다.
                fy + fh + ROOM_TICK_DOWN + FRAME_PAD + TICK_GAP
                    + font * crate::legend_layout::TEXT_LINE_EM,
                font,
                hancom_default::TEXT_COLOR,
                xml_escape(&label(v))
            ));
        } else {
            let gy = fy + fh - fh * t;
            // 바닥 조그(앞 좌측 눈금 → 뒷벽) + 뒷벽 수평 격자 + 좌측 값 틱
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx,
                gy,
                fx + dxf,
                gy - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx + dxf,
                gy - dyf,
                fx + fw + dxf,
                gy - dyf
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx - ROOM_TICK_LEFT,
                gy,
                fx,
                gy
            ));
            svg.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" font-family=\"inherit\" font-size=\"{:.2}\" fill=\"{}\" text-anchor=\"end\">{}</text>\n",
                fx - FRAME_PAD - TICK_GAP,
                gy + font * 0.35,
                font,
                hancom_default::TEXT_COLOR,
                xml_escape(&label(v))
            ));
        }
    }
    // 카테고리 경계 틱 — 경계+양끝 = cat_count+1개 (한컴 실측: 세로형은 바닥
    // 앞모서리 아래로, 가로형은 축선 왼쪽으로; k=0 틱이 축선 연장 역할)
    for k in 0..=cat_count {
        if horizontal {
            let by = fy + fh * k as f64 / cat_count.max(1) as f64;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                fx - ROOM_TICK_LEFT,
                by,
                fx,
                by
            ));
        } else {
            let bx = fx + fw * k as f64 / cat_count.max(1) as f64;
            svg.push_str(&format!(
                "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {ROOM_LINE_STYLE}/>\n",
                bx,
                fy + fh,
                bx,
                fy + fh + ROOM_TICK_DOWN
            ));
        }
    }
    svg.push_str("</g>\n");
}

/// 3D 막대 렌더 — 시어 투영(rAngAx=1 관례) 기반 별도 경로. 축 범위
/// (vmin/vmax/vstep)는 호출측 render_bars가 플롯 rect 기준으로 확정(#1882
/// 앵커 무접촉), 여기서는 방·배치·압출만 담당. 배치는 fit 후 앞면 rect 기준 —
/// 2D 루프와 완전 분리(2D 출력 바이트 불변). (C2b #2278 v2)
#[allow(clippy::too_many_arguments)]
pub(super) fn render_bars_3d(
    svg: &mut String,
    chart: &OoxmlChart,
    px: f64,
    py: f64,
    pw: f64,
    ph: f64,
    horizontal: bool,
    stacked: bool,
    percent: bool,
    cat_count: usize,
    ser_count: usize,
    layout: &CartesianLayout,
) {
    let (vmin, vmax, vstep) = layout.scales.primary.tuple();
    let view = chart.view3d.clone().unwrap_or_default();
    // 두께 규칙 slot/(n_eff + gapWidth/100) — Excel gapWidth 의미(슬롯 내 여백을
    // 막대 폭 %로). 코퍼스 150: 누적 1/2.5=0.4, 묶은 3계열 3/4.5≈0.667 —
    // v1~v3 눈대중 상수의 유도 원형. 2D는 0.7 휴리스틱 동결.
    let gap_w = chart
        .bar_gap_width
        .unwrap_or(OOXML_DEFAULT_GAP_PERCENT)
        .max(0.0);
    let n_eff = if stacked { 1.0 } else { ser_count as f64 };

    // fit 전 좌표로 깊이 산출 (fit이 깊이에 의존 — 역방향 순환 없음).
    // depthPercent = 막대 깊이/폭 % — ECMA "차트 폭 대비"와 다른 의도적 편차
    // (mod.rs View3D 주석 참조).
    let slot0 = (if horizontal { ph } else { pw }) / cat_count as f64;
    let bar_w0 = slot0 / (n_eff + gap_w / 100.0);
    let depth_slots = 1.0
        + chart
            .gap_depth
            .unwrap_or(OOXML_DEFAULT_GAP_PERCENT)
            .max(0.0)
            / 100.0;
    let (b_depth, d_scene, proj) = if let Some(scene) = &layout.scene {
        // 자동 2.5D 장면은 범주 간격의 3/4 깊이를 사용한다. 원본·개수·크기 Print 대조로
        // 확인한 기본값이며 막대군의 폭·계열 수·z 여백으로 장면을 다시 늘리지 않는다.
        let depth = scene.depth(horizontal);
        (
            depth / depth_slots,
            depth,
            scene.project(&view, px, py, depth),
        )
    } else {
        let bar_depth = bar_w0 * view.depth_percent.max(0.0) / 100.0;
        let depth = bar_depth * depth_slots;
        (bar_depth, depth, shear_proj(&view, px, py, pw, ph, depth))
    };

    // 막대 깊이 센터링: z ∈ [z0, z0+b], z0 = (D−b)/2 — gapDepth 여백을 앞뒤로
    // 분할(Excel/한컴 관례, v2 설계 리뷰). d_scene ≤ ε 퇴화(depthPercent=0)는
    // 0/0 NaN 가드.
    let (bdx0, bdy0, bdx, bdy) = if d_scene > 1e-9 {
        let z0 = (d_scene - b_depth) / 2.0 / d_scene;
        let zb = b_depth / d_scene;
        (proj.dxf * z0, proj.dyf * z0, proj.dxf * zb, proj.dyf * zb)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    render_value_grid_3d(
        svg,
        label_font_px(chart),
        &proj,
        vmin,
        vmax,
        vstep,
        chart.series.first().and_then(|s| s.format_code.as_deref()),
        horizontal,
        percent,
        cat_count,
    );

    // 배치는 fit 후 앞면 rect 기준
    let (fx, fy, fw, fh) = (proj.fx, proj.fy, proj.fw, proj.fh);
    let cat_span = (if horizontal { fh } else { fw }) / cat_count as f64;
    let bar_w = cat_span / (n_eff + gap_w / 100.0);
    let bar_span_total = bar_w * n_eff;

    // 가로 막대는 카테고리를 아래→위로 배치 (2D와 동일 규칙)
    let cat_slot = |ci: usize| -> f64 {
        let idx = if horizontal { cat_count - 1 - ci } else { ci };
        cat_span * idx as f64
    };

    if stacked {
        // 페인트 순서(세로: 아래→위, 가로: 왼→오른쪽)가 은면 제거 담당 —
        // 시어 성분 ≥0 클램프(shear_proj)가 이 순서의 유효 전제. 순서 변경 금지.
        for ci in 0..cat_count {
            let denom = if percent {
                let s = category_positive_sum(chart, ci);
                if s > 0.0 {
                    s
                } else {
                    1.0
                }
            } else {
                (vmax - vmin).max(1e-9)
            };
            let mut acc = 0.0_f64;
            for (si, ser) in chart.series.iter().enumerate() {
                let v = ser.values.get(ci).copied().unwrap_or(0.0).max(0.0);
                let rgb = ser.color.unwrap_or_else(|| palette(si));
                let cell = (if horizontal { fy } else { fx })
                    + cat_slot(ci)
                    + (cat_span - bar_span_total) / 2.0;
                if horizontal {
                    let seg = fw * (v / denom);
                    push_bar_3d(
                        svg,
                        fx + acc + bdx0,
                        cell - bdy0,
                        seg.max(0.0),
                        bar_span_total,
                        bdx,
                        bdy,
                        rgb,
                    );
                    acc += seg;
                } else {
                    let seg = fh * (v / denom);
                    let by = fy + fh - acc - seg;
                    push_bar_3d(
                        svg,
                        cell + bdx0,
                        by - bdy0,
                        bar_span_total,
                        seg.max(0.0),
                        bdx,
                        bdy,
                        rgb,
                    );
                    acc += seg;
                }
            }
        }
    } else {
        for ci in 0..cat_count {
            for (si, ser) in chart.series.iter().enumerate() {
                let v = *ser.values.get(ci).unwrap_or(&0.0);
                let t = if vmax > vmin {
                    (v - vmin) / (vmax - vmin)
                } else {
                    0.0
                };
                let rgb = ser.color.unwrap_or_else(|| palette(si));
                if horizontal {
                    let cy = fy
                        + cat_slot(ci)
                        + (cat_span - bar_span_total) / 2.0
                        + bar_w * (ser_count - 1 - si) as f64;
                    let bw = fw * t;
                    // 3D는 gapWidth 규칙이 간격 담당 — 2D의 0.95 계수 미적용
                    push_bar_3d(svg, fx + bdx0, cy - bdy0, bw.max(0.0), bar_w, bdx, bdy, rgb);
                } else {
                    let cx =
                        fx + cat_slot(ci) + (cat_span - bar_span_total) / 2.0 + bar_w * si as f64;
                    let bh = fh * t;
                    let by = fy + fh - bh;
                    push_bar_3d(svg, cx + bdx0, by - bdy0, bar_w, bh.max(0.0), bdx, bdy, rgb);
                }
            }
        }
    }

    // 3D 라벨은 투영한 좌측 카테고리 틱 바깥에 놓는다.
    render_category_labels_at(
        svg,
        chart,
        fx,
        fy,
        fw,
        fh,
        cat_count,
        horizontal,
        FRAME_PAD + TICK_GAP,
        false,
        layout.category,
    );
}
