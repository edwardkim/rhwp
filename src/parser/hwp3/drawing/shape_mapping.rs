//! HWP3 그리기 기본 속성과 세부 정보를 공통 도형 모델에 보존한다.
use super::*;

/// 신뢰할 수 없는 파일에서 읽은 HWP3 raw margin(u32)을 `* HWP3_UNIT_SCALE` 스케일 후
/// `i16` 필드(`TextBox::margin_*`)에 담는다. 곱셈이 `i32`/`i16` 범위를 넘으면 그대로
/// 캐스팅하는 대신 클램프해 오버플로 panic(malformed/fuzzed 파일에서의 DoS)을 막는다.
fn hwp3_margin_to_i16(raw_margin: u32) -> i16 {
    let scaled = raw_margin as i64 * HWP3_UNIT_SCALE as i64;
    scaled.clamp(i16::MIN as i64, i16::MAX as i64) as i16
}

/// HWP3 사각형의 무색·무늬 없음 sentinel을 채우기 없음으로 판정한다.
///
/// `fill_color`의 high byte는 표 69의 RGB 영역이 아니다. 실제 HWP3 원본에서
/// 사각형의 `0x10000000`은 테두리만 남기는 no-fill 표식으로 사용된다. 반면 같은
/// 표식이 글상자 기본 스타일에도 나타나므로 개체 종류와 무늬/그라데이션 부재까지
/// 함께 확인한다.
pub(super) fn hwp3_rectangle_uses_no_fill_marker(header: &Hwp3DrawingObjectCommonHeader) -> bool {
    header.object_type == 2
        && header.basic_attr.fill_color == 0x1000_0000
        && header.basic_attr.pattern_type == 0
        && header.gradient_attr.is_none()
}

/// HWP3 선 색상의 무색 sentinel을 테두리 없음으로 판정한다.
///
/// `0x10000000`은 검정 RGB 값이 아니다. HWP3 원본의 글자처럼 취급된 사각형과
/// 여백 구분선에서 이 값은 "선 없음"을 뜻한다. 기존의 `line_width > 0` 보정이
/// 이 sentinel도 검정 실선으로 승격해 한컴/HWP5 변환본에는 없는 테두리를 그렸다.
/// 실제 검정은 `0x00000000`이므로 구분할 수 있다.
pub(super) fn hwp3_uses_no_line_marker(header: &Hwp3DrawingObjectCommonHeader) -> bool {
    header.basic_attr.line_color == 0x1000_0000
}

/// [다섯 번째 계약] HWP3 다각형 점 배열 → PolygonShape (HWP3 단위 ×HWP3_UNIT_SCALE).
fn polygon_shape_from_points(points: &[[i32; 2]]) -> PolygonShape {
    PolygonShape {
        points: points
            .iter()
            .map(|p| crate::model::Point {
                x: p[0].saturating_mul(HWP3_UNIT_SCALE),
                y: p[1].saturating_mul(HWP3_UNIT_SCALE),
            })
            .collect(),
        ..Default::default()
    }
}

/// [열세 번째 계약] HWP3 곡선 점 배열 → CurveShape (HWP3 단위 ×HWP3_UNIT_SCALE).
/// 종전에는 점을 버려 점 0개 곡선을 저장했고, 한글 2022 는 빈 곡선을 만나면
/// **크래시**(RPC 붕괴)한다(빈 다각형=거부와 달리 곡선은 크래시 — 다섯 번째
/// 계약의 곡선 판). segment_types 는 점 수-1(모두 곡선 세그먼트 1).
fn curve_shape_from_points(points: &[[i32; 2]]) -> CurveShape {
    let pts: Vec<crate::model::Point> = points
        .iter()
        .map(|p| crate::model::Point {
            x: p[0].saturating_mul(HWP3_UNIT_SCALE),
            y: p[1].saturating_mul(HWP3_UNIT_SCALE),
        })
        .collect();
    let segs = pts.len().saturating_sub(1);
    CurveShape {
        segment_types: vec![1u8; segs],
        points: pts,
        ..Default::default()
    }
}

pub(super) fn map_to_shape_object(
    raw: Hwp3DrawingObject,
    doc_char_shapes: &mut Vec<crate::model::style::CharShape>,
    doc_para_shapes: &mut Vec<crate::model::style::ParaShape>,
    doc_border_fills: &mut Vec<crate::model::style::BorderFill>,
    doc_tab_defs: &mut Vec<crate::model::style::TabDef>,
    pic_name_to_id: &mut HashMap<String, u16>,
) -> Result<(ShapeObject, u16), Hwp3Error> {
    let mut parsed_paragraphs = Vec::new();

    let (header, shape) = match raw {
        Hwp3DrawingObject::Container(hdr) => (hdr, ShapeObject::Group(GroupShape::default())),
        Hwp3DrawingObject::Line(hdr, details) => {
            // 공식 HWP3 표 68/74: 선의 개체 크기와 축별 반전으로 끝점을 복원한다.
            // 선 두께를 위한 차지 영역(bounds)을 끝점으로 대신하지 않는다.
            let scale = |v: u32| {
                (i64::from(v) * i64::from(HWP3_UNIT_SCALE)).min(i64::from(i32::MAX)) as i32
            };
            let width = scale(hdr.object_size[0]);
            let height = scale(hdr.object_size[1]);
            let flip_x = details.shape_info & 1 != 0;
            let flip_y = details.shape_info & 2 != 0;
            let line = LineShape {
                start: crate::model::Point {
                    x: if flip_x { width } else { 0 },
                    y: if flip_y { height } else { 0 },
                },
                end: crate::model::Point {
                    x: if flip_x { 0 } else { width },
                    y: if flip_y { 0 } else { height },
                },
                ..Default::default()
            };
            (hdr, ShapeObject::Line(line))
        }
        Hwp3DrawingObject::Rectangle(hdr) => {
            (hdr, ShapeObject::Rectangle(RectangleShape::default()))
        }
        Hwp3DrawingObject::Ellipse(hdr) => (hdr, ShapeObject::Ellipse(EllipseShape::default())),
        Hwp3DrawingObject::Arc(hdr, _details) => (hdr, ShapeObject::Arc(ArcShape::default())),
        Hwp3DrawingObject::Polygon(hdr, details) => {
            // [다섯 번째 계약] 꼭짓점을 IR 에 싣는다 — 종전 default() 는 점 0개
            // SC_POLYGON(8B)을 저장했고, 한글 2022 는 빈 다각형이 든 문서를
            // 통째로 거부했다(hwp3-sample11 문단 이등분 COM 실측 — p1809 Polygon).
            (
                hdr,
                ShapeObject::Polygon(polygon_shape_from_points(&details.points)),
            )
        }
        Hwp3DrawingObject::TextBox(hdr, details) => {
            if details.info2_len > 0 {
                let mut text_cursor = std::io::Cursor::new(details.paragraph_list_data.as_slice());
                let paras = crate::parser::hwp3::parse_paragraph_list(
                    &mut text_cursor,
                    doc_char_shapes,
                    doc_para_shapes,
                    doc_border_fills,
                    doc_tab_defs,
                    pic_name_to_id,
                    0,            // body_left_hu: 드로잉 내부 텍스트, wrap zone 불필요
                    i32::MAX / 2, // column_width_hu
                    0,            // body_height_hu: 도형 내부 텍스트는 본문 페이지 분할 제외
                    false,        // 복호화 원본의 본문 Square-wrap 계약은 적용하지 않음
                )?;
                parsed_paragraphs = paras;
            }
            (hdr, ShapeObject::Rectangle(RectangleShape::default()))
        }
        Hwp3DrawingObject::Curve(hdr, details) => (
            hdr,
            ShapeObject::Curve(curve_shape_from_points(&details.points)),
        ),
        Hwp3DrawingObject::ModifiedEllipse(hdr, _details) => {
            (hdr, ShapeObject::Ellipse(EllipseShape::default()))
        }
        Hwp3DrawingObject::ModifiedArc(hdr) => (hdr, ShapeObject::Arc(ArcShape::default())),
        Hwp3DrawingObject::ExtendedCurve(hdr, _details) => {
            (hdr, ShapeObject::Curve(CurveShape::default()))
        }
        Hwp3DrawingObject::ClosedPolygon(hdr, details) => {
            // 닫힌 다각형(ExtendedPolygon)도 같은 계약 — 점 좌표는 동일 레이아웃.
            (
                hdr,
                ShapeObject::Polygon(polygon_shape_from_points(&details.points)),
            )
        }
        Hwp3DrawingObject::Unknown(hdr, _data) => (hdr, ShapeObject::Group(GroupShape::default())),
    };

    let connection_info = header.connection_info;
    let mut final_shape = shape;

    // [#5558] 공통 헤더 뒤 글상자 정보로 실려 온 내부 문단 리스트(사각형 등
    // 비-글상자 개체의 라벨 텍스트). 글상자(type 6)와 같은 계약으로 파싱해
    // 아래 text_box 조립에 태운다. 손상된 리스트는 텍스트만 포기하고 도형은
    // 유지한다(종전 동작과 동일).
    if parsed_paragraphs.is_empty() {
        if let Some(data) = header.textbox_paragraph_list.as_deref() {
            let mut text_cursor = std::io::Cursor::new(data);
            match crate::parser::hwp3::parse_paragraph_list(
                &mut text_cursor,
                doc_char_shapes,
                doc_para_shapes,
                doc_border_fills,
                doc_tab_defs,
                pic_name_to_id,
                0,            // body_left_hu: 드로잉 내부 텍스트, wrap zone 불필요
                i32::MAX / 2, // column_width_hu
                0,            // body_height_hu: 도형 내부 텍스트는 본문 페이지 분할 제외
                false,        // 복호화 원본의 본문 Square-wrap 계약은 적용하지 않음
            ) {
                Ok(paras) => parsed_paragraphs = paras,
                Err(error @ Hwp3Error::NestingLimitExceeded { .. }) => return Err(error),
                Err(_) => {} // Preserve recovery for ordinary damaged textbox data.
            }
        }
    }

    let common = CommonObjAttr {
        width: header.object_size[0].saturating_mul(HWP3_UNIT_SCALE as u32),
        height: header.object_size[1].saturating_mul(HWP3_UNIT_SCALE as u32),
        ..Default::default()
    };

    let mut rotation_angle = 0i16;
    if let Some(ref rot) = header.rotation_attr {
        let x0 = rot.parallelogram[0] as f64;
        let y0 = rot.parallelogram[1] as f64;
        let x1 = rot.parallelogram[2] as f64;
        let y1 = rot.parallelogram[3] as f64;

        let dx = x1 - x0;
        let dy = y1 - y0;
        if dx != 0.0 || dy != 0.0 {
            let mut angle = dy.atan2(dx) * 180.0 / std::f64::consts::PI;
            if angle < 0.0 {
                angle += 360.0;
            }
            rotation_angle = angle.round() as i16;
        }
    }

    let mut shape_attr = ShapeComponentAttr {
        offset_x: (header.relative_pos[0] as i64 * HWP3_UNIT_SCALE as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        offset_y: (header.relative_pos[1] as i64 * HWP3_UNIT_SCALE as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        original_width: header.object_size[0].saturating_mul(HWP3_UNIT_SCALE as u32),
        original_height: header.object_size[1].saturating_mul(HWP3_UNIT_SCALE as u32),
        current_width: header.object_size[0].saturating_mul(HWP3_UNIT_SCALE as u32),
        current_height: header.object_size[1].saturating_mul(HWP3_UNIT_SCALE as u32),
        rotation_angle,
        ..Default::default()
    };

    // V3 표68의 absolute_pos는 그림 틀 기준 위치다. 표70의 세 점은
    // 개체 로컬 평행사변형을 저장한다. 각도 하나로 줄이면 전단·뒤집기와
    // 시작 꼭짓점을 잃으므로 공통 ShapeComponentAttr의 전체 affine으로 옮긴다.
    // 컨테이너는 구조만 소유하고, 자식의 absolute_pos가 이미 모든 부모 이동을 포함한다.
    if !matches!(&final_shape, ShapeObject::Group(_)) {
        shape_attr.render_tx = f64::from(header.absolute_pos[0]) * f64::from(HWP3_UNIT_SCALE);
        shape_attr.render_ty = f64::from(header.absolute_pos[1]) * f64::from(HWP3_UNIT_SCALE);
        if let Some(rot) = &header.rotation_attr {
            let width = f64::from(header.object_size[0]);
            let height = f64::from(header.object_size[1]);
            if width > 0.0 && height > 0.0 {
                let p = rot.parallelogram.map(f64::from);
                shape_attr.render_sx = (p[2] - p[0]) / width;
                shape_attr.render_c = (p[3] - p[1]) / width;
                shape_attr.render_b = (p[4] - p[2]) / height;
                shape_attr.render_sy = (p[5] - p[3]) / height;
                shape_attr.render_tx += p[0] * f64::from(HWP3_UNIT_SCALE);
                shape_attr.render_ty += p[1] * f64::from(HWP3_UNIT_SCALE);
            }
        }
    }

    let arrow_attr = ((header.basic_attr.arrow_start & 0x3F) << 10)
        | ((header.basic_attr.arrow_end & 0x3F) << 16)
        | (u32::from(header.basic_attr.arrow_start != 0) << 30)
        | (u32::from(header.basic_attr.arrow_end != 0) << 31);
    let border_line = ShapeBorderLine {
        color: header.basic_attr.line_color,
        width: (header.basic_attr.line_width as i64 * HWP3_UNIT_SCALE as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        // [Task #877 Stage 3] HWP3 drawing line_style = 0 (= "선 종류 없음") 인데
        // line_width > 0 인 경우 → 실제 한컴 viewer 는 실선으로 표시. (sample16 RFP
        // 박스 외곽선 회귀: raw line_style=0, line_width=84, line_color=0 검정)
        // 렌더러 [renderer/layout/utils.rs:163] 의 `attr & 0x3F == 0` 시 외곽선 미표시
        // 규칙에 맞추기 위해 bit 0..5 = 1 (Solid LineType) 보강.
        //
        // 독립 한컴 Print와 같은 원본의 저장본에서 HWP3 스타일 2는
        // HWP5 Dot(3)이다. 다른 선 종류까지 실선으로 지우지 않는다.
        attr: if hwp3_uses_no_line_marker(&header) {
            // line_width가 남아 있어도 HWP3 무색 sentinel은 선을 그리지 않는다.
            0
        } else {
            let raw_attr = header.basic_attr.line_style as u32;
            let line_type = raw_attr & 0x3F;
            if line_type == 0 && header.basic_attr.line_width > 0 {
                raw_attr | 0x01
            } else if line_type == 2 {
                (raw_attr & !0x3F) | 0x03
            } else {
                raw_attr
            }
        },
        outline_style: 0,
    };
    // 기본 속성 표 69의 시작/끝 화살표를 보존한다. 무색 선은 그대로 비가시다.
    let border_line = ShapeBorderLine {
        attr: border_line.attr | arrow_attr,
        ..border_line
    };

    let fill = if hwp3_rectangle_uses_no_fill_marker(&header) {
        // HWP3 사각형의 `0x10000000`은 RGB가 아니라 "채우기 없음" sentinel이다.
        // 이 값을 흰색 단색 채움으로 대체하면, 아이콘 위에 놓인 테두리 사각형이
        // 아이콘을 가려 버린다. 한컴 HWP5 변환본도 같은 사각형을 Fill=None으로
        // 기록한다. 글상자(type 6)는 같은 high byte를 서로 다른 기본 스타일에
        // 사용하므로, 사각형·무늬 없음·비그라데이션 조합으로 한정한다.
        Fill::default()
    } else {
        let raw_fc = header.basic_attr.fill_color;
        let fill_flag = (raw_fc >> 24) & 0xFF;
        let fill_rgb = raw_fc & 0x00FFFFFF;
        // HWP3 글상자 등은 high-byte marker와 RGB=0을 기본 흰색 면으로 사용한다.
        // 사각형의 no-fill marker만 위 분기에서 분리하고, 기존 가시성 보정은 유지한다.
        let effective_rgb = if fill_flag != 0 && fill_rgb == 0 {
            0x00FFFFFF
        } else {
            fill_rgb
        };

        // [Task #1008 격차 A] HWP3 gradient_attr 이 파싱된 경우 IR Fill.gradient 에 매핑.
        // HWP3 raw stream 의 Hwp3DrawingObjectGradientAttr (drawing.rs:149~170) 은 이미
        // basic_attr.has_gradient() 시 파싱되어 header.gradient_attr 에 보존되지만, 종전
        // 코드는 fill_type 을 항상 Solid 로 하드코딩하여 데이터가 무시되었음. HWP5 의
        // doc_info.rs:404 매핑과 동일 contract 로 IR 주입 (step→blur, 2-stop colors,
        // positions=vec![] → renderer 가 균등 분포).
        let (fill_type, gradient) = if let Some(g) = header.gradient_attr.as_ref() {
            let grad = crate::model::style::GradientFill {
                gradient_type: g.kind as i16,
                angle: g.angle as i16,
                center_x: g.center_x as i16,
                center_y: g.center_y as i16,
                blur: g.step as i16,
                // HWP3에는 추가 전이 중심 필드가 없다. 균등한 색상 띠의
                // 중점을 보존한다. 명시적인 0%로 넣으면 공통 띠 확장에서
                // 앞쪽 절반이 원점에 접혀 원래 중심색까지 옅어진다.
                step_center: 50,
                colors: vec![g.start_color, g.end_color],
                positions: vec![],
            };
            (crate::model::style::FillType::Gradient, Some(grad))
        } else {
            (crate::model::style::FillType::Solid, None)
        };
        Fill {
            fill_type,
            solid: Some(crate::model::style::SolidFill {
                background_color: effective_rgb,
                pattern_color: header.basic_attr.pattern_color,
                pattern_type: header.basic_attr.pattern_type as i32,
            }),
            gradient,
            image: None,
            // [Task #877 Stage 4] 한컴 호환 alpha convention: 0=불투명, 255=완전 투명.
            // (renderer/layout/utils.rs:199 의 opacity 식: opacity = 1 - alpha/255)
            // 기존 alpha=255 → opacity=0 → SVG <rect opacity="0.000"> 완전 투명 회귀.
            // HWP3 raw 에는 alpha 정보 없음, 한컴 viewer 의 default = 불투명 = alpha 0.
            alpha: 0,
        }
    };

    // 글상자는 **실제 문단이 있을 때만** 합성한다. HWP3 옵션 비트 19("글상자 있음")가
    // 켜져 있어도 파서가 문단을 복원하지 못한 개체(예: 회전 타원 type 8 — 텍스트가
    // info2 밖에 저장돼 미복원)에 빈 글상자(nPara=0 LIST_HEADER)를 방출하면, 한글 2022
    // 는 LIST_HEADER 뒤 문단이 없을 때 다음 레코드(SHAPE_COMPONENT 등)를 문단으로
    // 오독해 **문서 개방을 거부**한다(크롤 빈티지 "검인" 도장 회전 타원 COM 이등분 실측:
    // 빈 글상자 LIST 제거 → 개방). 빈 글상자는 보이는 내용이 없어 프레임 생략에도
    // 시각 손실이 없다.
    let text_box = if !parsed_paragraphs.is_empty() {
        Some(TextBox {
            margin_left: hwp3_margin_to_i16(header.basic_attr.textbox_margin[0]),
            margin_top: hwp3_margin_to_i16(header.basic_attr.textbox_margin[1]),
            margin_right: hwp3_margin_to_i16(header.basic_attr.textbox_margin[0]),
            margin_bottom: hwp3_margin_to_i16(header.basic_attr.textbox_margin[1]),
            paragraphs: parsed_paragraphs,
            ..Default::default()
        })
    } else {
        None
    };

    let drawing_attr = DrawingObjAttr {
        shape_attr,
        border_line,
        fill,
        text_box,
        ..Default::default()
    };

    match final_shape {
        ShapeObject::Line(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Rectangle(ref mut s) => {
            // HWP3 사각형과 글상자는 공통 헤더의 크기로 윤곽을 정의한다.
            // 이 암묵 기하를 IR에 남겨야 HWPX의 명시적인 네 꼭짓점도
            // 같은 면을 그린다. default의 네 원점은 퇴화 사각형이다.
            let width = common.width.min(i32::MAX as u32) as i32;
            let height = common.height.min(i32::MAX as u32) as i32;
            s.x_coords = [0, width, width, 0];
            s.y_coords = [0, 0, height, height];
            s.common = common;
            s.drawing = drawing_attr;
            // HWP3 기본 속성의 모서리 옵션을 공통 사각형 계약에 보존한다.
            // 둥근 모서리의 정상 저장본은 10%, 반원 모서리는 50%다.
            s.round_rate = if header.basic_attr.options & (1 << 2) != 0 {
                50
            } else if header.basic_attr.options & 1 != 0 {
                10
            } else {
                0
            };
        }
        ShapeObject::Ellipse(ref mut s) => {
            // 보통 타원의 중심과 두 축도 같은 로컬 개체 상자에서 정한다.
            // 변형 타원의 추가 정보는 이 보통 타원 계약으로 대체하지 않는다.
            if header.object_type == 3 {
                let width = common.width.min(i32::MAX as u32) as i32;
                let height = common.height.min(i32::MAX as u32) as i32;
                s.center = crate::model::Point {
                    x: width / 2,
                    y: height / 2,
                };
                s.axis1 = crate::model::Point { x: width / 2, y: 0 };
                s.axis2 = crate::model::Point {
                    x: width,
                    y: height / 2,
                };
            }
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Arc(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Polygon(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Curve(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Group(ref mut s) => {
            s.common = common;
            s.shape_attr = drawing_attr.shape_attr;
        }
        _ => {}
    }

    Ok((final_shape, connection_info))
}
