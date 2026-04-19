# HWP OLE 개체(SHAPE_COMPONENT_OLE) 바이너리 스펙 정리

> Task #195 단계 1 산출물. 구현 전 스펙 조사 및 IR 설계 문서.

## 1. OLE 컨트롤의 전체 구조

HWP 5.0의 OLE 개체는 **GSO 컨트롤의 한 형태**로 저장된다.
```
CTRL_HEADER (ctrl_id=b"gso ")
├── SHAPE_COMPONENT (공통 도형 속성)
├── HWPTAG_SHAPE_COMPONENT_OLE (= HWPTAG_BEGIN + 68)
│   └── OLE 개체 속성 (크기, 드로잉 속성, BinData 참조 ID 등)
└── (캡션 등)
```

OLE 개체의 실제 바이너리 내용은 `BinData/BIN000N.OLE` 스트림에 별도로 저장되며, SHAPE_COMPONENT_OLE 레코드는 해당 스트림을 가리키는 **참조 ID**와 렌더링 속성만 담는다.

## 2. HWPTAG_SHAPE_COMPONENT_OLE 레코드 필드

HWP 5.0 스펙 및 pyhwp 구현 기준 필드 구성 (바이트 오프셋):

| 오프셋 | 길이 | 필드 | 설명 |
|---|---|---|---|
| 0 | 4 | extent_x | 개체 가로(EMU 또는 HWPUNIT, 스펙 확인 필요) |
| 4 | 4 | extent_y | 개체 세로 |
| 8 | 1 | flags | OLE 속성 플래그 |
| 9 | 2 | drawing_aspect | 표시 방식(Icon/Content) |
| 11 | 4 | bin_data_id | `BinData/BIN000N.OLE` 참조 번호 |
| 15 | ... | border + reserved | 테두리 정보 |

> 정확한 오프셋은 구현 시(단계 3) pyhwp/hwplib 소스와 교차 확인한다. 현재 문서는 설계 지침 수준.

## 3. BinData 스트림 구조

HWP 파일은 Compound File Binary(CFB)이고, 내부에 `BinData/BIN000N.{OLE,bmp,jpg,png,...}` 형태의 스트림이 저장된다.

- 확장자가 `.OLE`인 스트림 = **내부가 또 다른 CFB (nested OLE)**
- DocInfo의 `BinDataItem` 레코드에 압축/암호화 속성이 정의됨
  - `compressed: bool`
  - `encrypted: bool`

### 1.hwp 실제 조사 결과
```
BinData/BIN0001.OLE  size=30110  magic=ec975b4c...
BinData/BIN0002.OLE  size=22204  magic=ec965b6c...
```
CFB 매직(`d0cf11e0`) 아님 → **압축된 스트림**으로 추정 (deflate + BinDataItem 압축 플래그).
단계 3에서 DocInfo의 BinDataItem 속성 확인 후 압축 해제 로직 추가.

## 4. 중첩 CFB 내부의 프리뷰 이미지

압축 해제 후 얻은 CFB의 표준 구조 (MS Graph / Excel OLE):
```
BIN0001.OLE (CFB 압축 해제 후)
├── \001CompObj        — OLE 타입 정보 (ClassID)
├── \005SummaryInformation
├── \005DocumentSummaryInformation
├── CONTENTS            — 실제 OLE 데이터
├── \001Ole             — OLE 스트림
└── Workbook / Graph    — MS Graph인 경우
```

프리뷰 이미지는 보통 **없거나** `\001Ole` 스트림 내부에 WMF/EMF 형태로 포함된다. 표준 OLE 포맷(Package Stream 등)의 `PresentationData` 영역도 후보.

## 5. 프리뷰 추출 전략 (단계 4 렌더링 범위)

우선순위:
1. **Graph 차트**인 경우 → CHART_DATA를 통해 차트 렌더 (OLE 자체는 placeholder)
2. **이미지 리소스**인 경우 (DocInfo의 BinDataItem에 이미지 확장자가 있으면) → 직접 이미지 로드
3. **일반 OLE**(워드/엑셀/파워포인트 등) → placeholder 사각형 + 앱 아이콘 텍스트

1차 구현은 **(3) placeholder만** 목표. WMF/EMF 추출은 기존 `src/wmf/` 파서와 연계하여 2차 작업.

## 6. rhwp IR 설계 — `OleShape`

```rust
pub struct OleShape {
    pub common: CommonObjAttr,
    pub drawing: DrawingObject,
    pub extent: (i32, i32),               // 크기(HWPUNIT)
    pub flags: u8,
    pub drawing_aspect: u16,
    pub bin_data_id: u32,                 // BinData 참조
    pub preview: Option<OlePreview>,      // 추출된 프리뷰 (1차: None)
    pub raw_tag_data: Vec<u8>,            // 라운드트립용
    pub caption: Option<Caption>,
}

pub struct OlePreview {
    pub format: OlePreviewFormat,         // Wmf/Emf/Png/Bmp
    pub bytes: Vec<u8>,
}

pub enum OlePreviewFormat { Wmf, Emf, Png, Bmp }
```

## 7. 파싱 플로우

```
parse_gso_control
  └── shape_tag_id == HWPTAG_SHAPE_COMPONENT_OLE
       └── parse_ole_shape(common, drawing, shape_tag_data)
            ├── OleShape 속성 파싱
            ├── bin_data_id 기록
            └── preview = None (1차)
```

프리뷰는 **렌더 시점**에 DocInfo.BinDataItem과 BinData 스트림을 조회하여 로드 (zero-copy 어려우면 렌더러가 캐시).

## 8. 렌더링 방침 (단계 4)

- 1차: 회색 사각형(`#E0E0E0`) + 중앙에 "OLE 개체" 텍스트 + 크기는 common.width/height
- 2차(별도 이슈): 실제 프리뷰 이미지 추출

## 9. 검증 체크리스트

- [ ] SHAPE_COMPONENT_OLE가 분기 타는지 (기존 "Rectangle 폴백" 제거 확인)
- [ ] bin_data_id 파싱 정확성 — DocInfo의 BinDataItem 번호와 일치
- [ ] raw_tag_data 보존으로 라운드트립
- [ ] 1.hwp 차트 OLE 2건이 빈 사각형 아닌 "OLE 개체" placeholder로 렌더 (단, 차트로 먼저 분기되면 이 케이스 해당 없음)

## 10. 참고 자료

- HWP 5.0 파일 포맷 스펙
- pyhwp (`pyhwp/hwp5/binmodel.py`의 `ShapeOLE`)
- hwplib (`com.hancom.objects.OleObject`)
- MS-CFB, MS-OLEDS 공식 문서
- rhwp 기존 WMF 파서: `src/wmf/` (OLE 내부 WMF 디코드 시 재사용)

## 11. 미해결 이슈

- BinData 스트림 압축(`ec 97 ...` 매직)이 zlib raw deflate인지 HWP 자체 포맷인지 단계 3에서 확정
- 중첩 CFB 파싱에 `cfb` crate(이미 Cargo.toml에 존재) 재사용 가능
- WMF/EMF 구분 및 SVG 변환은 별도 이슈 분리 가능성
