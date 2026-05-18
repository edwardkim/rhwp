//! EMF RecordType 카탈로그 (MS-EMF 2.1.1 기준, rhwp 1차 범위 발췌).
//!
//! 값 출처: [MS-EMF] Record Types. 전체 200+ 중 rhwp 1차 구현에 필요한 항목만.
//! 구현 단계는 단계 10/11/12/13 태그로 구분한다.

/// EMF 레코드 타입. `u32` 리터럴과 호환되도록 `#[repr(u32)]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RecordType {
    // 제어 (단계 10)
    EmrHeader = 0x00000001,
    EmrEof = 0x0000000E,

    // 드로잉 (단계 12)
    EmrPolyBezier = 0x00000002,
    EmrPolygon = 0x00000003,
    EmrPolyline = 0x00000004,
    EmrPolyBezierTo = 0x00000005,
    EmrPolyLineTo = 0x00000006,
    EmrPolyPolyline = 0x00000007,
    EmrPolyPolygon = 0x00000008,
    EmrMoveToEx = 0x0000001B,
    EmrSetPixelV = 0x0000000F,
    EmrLineTo = 0x00000036,
    EmrArcTo = 0x00000037,
    EmrPolyDraw = 0x00000038,
    EmrEllipse = 0x0000002A,
    EmrRectangle = 0x0000002B,
    EmrRoundRect = 0x0000002C,
    EmrArc = 0x0000002D,
    EmrChord = 0x0000002E,
    EmrPie = 0x0000002F,
    EmrPolyline16 = 0x00000056,
    EmrPolyBezier16 = 0x00000055,
    EmrPolygon16 = 0x00000057,
    EmrPolyPolyline16 = 0x00000058,
    EmrPolyPolygon16 = 0x00000059,
    EmrPolyBezierTo16 = 0x00000060,
    EmrPolyLineTo16 = 0x00000061,

    // 패스 (단계 12)
    EmrBeginPath = 0x0000003B,
    EmrEndPath = 0x0000003C,
    EmrCloseFigure = 0x0000003D,
    EmrFillPath = 0x0000003E,
    EmrStrokeAndFillPath = 0x0000003F,
    EmrStrokePath = 0x00000040,
    EmrAbortPath = 0x00000044,

    // 객체 (단계 11)
    EmrCreatePen = 0x00000026,
    EmrCreateBrushIndirect = 0x00000027,
    EmrDeleteObject = 0x00000028,
    EmrSelectObject = 0x00000025,
    EmrExtCreateFontIndirectW = 0x00000052,
    EmrExtCreatePen = 0x0000005F,

    // 상태 (단계 11)
    EmrSaveDC = 0x00000021,
    EmrRestoreDC = 0x00000022,
    EmrSetWorldTransform = 0x00000023,
    EmrModifyWorldTransform = 0x00000024,
    EmrSetMapMode = 0x00000011,
    EmrSetBkMode = 0x00000012,
    EmrSetPolyFillMode = 0x00000013,
    EmrSetROP2 = 0x00000014,
    EmrSetStretchBltMode = 0x00000015,
    EmrSetTextAlign = 0x00000016,
    EmrSetTextColor = 0x00000018,
    EmrSetBkColor = 0x00000019,
    EmrSetWindowExtEx = 0x00000009,
    EmrSetWindowOrgEx = 0x0000000A,
    EmrSetViewportExtEx = 0x0000000B,
    EmrSetViewportOrgEx = 0x0000000C,
    EmrSetBrushOrgEx = 0x0000000D,
    EmrScaleViewportExtEx = 0x0000001F,
    EmrScaleWindowExtEx = 0x00000020,

    // 텍스트 (단계 13)
    EmrExtTextOutA = 0x00000053,
    EmrExtTextOutW = 0x00000054,

    // 비트맵 (단계 13)
    EmrBitBlt = 0x0000004C,
    EmrStretchBlt = 0x0000004D,
    EmrMaskBlt = 0x0000004E,
    EmrStretchDIBits = 0x00000051,
    EmrSetDIBitsToDevice = 0x00000050,

    // 주석 (EMF+ 컨테이너 식별용, 단계 10 이후 감지)
    EmrComment = 0x00000046,
}

impl RecordType {
    /// u32 값으로부터 RecordType으로 변환(매핑되지 않으면 None).
    #[must_use]
    pub fn from_u32(v: u32) -> Option<Self> {
        // 주의: enum 값이 많아 기계적 match는 생략하고 후속 단계에서 필요 항목만 분기.
        // 단계 10은 Header/Eof만 식별하면 충분하다.
        match v {
            0x00000001 => Some(Self::EmrHeader),
            0x0000000E => Some(Self::EmrEof),
            _ => None,
        }
    }
}
