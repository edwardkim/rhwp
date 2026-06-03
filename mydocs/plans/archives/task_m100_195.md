# Task #195: 차트/OLE 개체 렌더링 지원 — 수행계획서

> 마일스톤: 미지정

## 목표

HWP 파일 내 차트 및 OLE 개체를 파싱·모델링·렌더링하여, 현재 "빈 사각형"으로 대체되는 문제를 해결한다.

## 배경

### 재현 파일
- `/home/planet/iop/1.hwp` — 테이블 셀 내부에 차트 2개 포함
- `rhwp dump` 결과: `ctrl[0] 도형: tac=false, wrap=Square` 2건 (셀[42], 셀[62])
- `rhwp export-svg` 결과: 차트 위치에 빈 사각형 출력

### 현재 구현 상태 (미구현 지점)

1. **태그 상수만 정의, 파싱 없음** (`src/parser/tags.rs`)
   - `HWPTAG_SHAPE_COMPONENT_OLE` = HWPTAG_BEGIN + 68
   - `HWPTAG_CHART_DATA` = HWPTAG_BEGIN + 79

2. **shape_tag_id 미지 분기에서 RectangleShape로 대체** (`src/parser/control/shape.rs:236-242`)
   ```rust
   _ => {
       // 알 수 없는 도형 → 사각형으로 대체
       let mut rect = RectangleShape::default();
       ...
   }
   ```

3. **모델 enum에 Chart/Ole variant 없음** (`src/model/shape.rs:250`)
   - 현재: Line / Rectangle / Ellipse / Arc / Polygon / Curve / Group / Picture

4. **CHART_DATA 태그는 어디서도 참조되지 않음** (grep 기준 tags.rs 정의 외 0건)

## 범위

### 포함 (1차 범위 — 단계 1~5)
- **차트(Chart)**: SHAPE_COMPONENT + HWPTAG_CHART_DATA 구조 파싱 + placeholder 렌더
- **OLE 개체(Ole)**: SHAPE_COMPONENT_OLE 구조 파싱 + placeholder 렌더

### 포함 (2차 범위 — 단계 6~8, 스코프 확장)
- **BinData 스트림 해제**: HWP zlib raw deflate 압축 해제 + DocInfo BinDataItem 플래그 활용
- **중첩 OLE/CFB 파싱**: 내부 `\x02OlePres000`, `OOXMLChartContents`, `Contents` 스트림 접근
- **실제 차트 렌더링**:
  - **우선 경로**: OOXMLChartContents(XML) 파싱 → 네이티브 SVG 차트 (barChart/lineChart/pieChart)
  - **폴백 경로**: OOXMLChartContents 부재 시 EMF 프리뷰 바이트를 외부 파일로 추출 + `<image>` 참조

### 포함 (3차 범위 — 단계 9~14, 스코프 재확장)
- **EMF → 네이티브 SVG 벡터 변환기**: `\x02OlePres000`에서 추출된 EMF 바이트를 네이티브 SVG로 변환하여, OOXMLChartContents가 없는 OLE 객체(워드/엑셀 임베딩, Visio, 수식 등)도 placeholder 대신 실제 내용으로 렌더
- 1차 EMF 범위: GDI 기본 레코드(선/사각형/타원/패스/텍스트/비트맵), DC 스택, 객체 핸들
- `src/emf/` 독립 모듈로 신설, WMF 모듈과 분리

### 제외
- 3D 차트, 복합 차트, 보조축, 추세선
- OOXML 차트 애니메이션/세밀 스타일
- EMF+ (GDI+ 확장 레코드), 그라데이션/알파블렌드/리전 클리핑 고급 레코드 — 후속 이슈
- ICM 색상 관리, 디바이스 의존 비트맵 회전·왜곡 정밀 재현
- 차트/EMF 편집(읽기 전용)

## 영향 범위

| 계층 | 파일 | 변경 내용 |
|------|------|----------|
| Parser | `src/parser/control/shape.rs` | CHART_DATA / OLE 분기 추가 |
| Parser | `src/parser/control/shape_chart.rs` (신규) | 차트 파서 |
| Parser | `src/parser/control/shape_ole.rs` (신규) | OLE 파서 |
| Model | `src/model/shape.rs` | ShapeObject::Chart, ShapeObject::Ole variant 추가 |
| Model | `src/model/shape/chart.rs` (신규) | ChartShape 구조체 |
| Model | `src/model/shape/ole.rs` (신규) | OleShape 구조체 |
| Renderer | `src/renderer/layout/shape_layout.rs` | Chart/Ole 레이아웃 분기 |
| Renderer | `src/renderer/svg.rs` (또는 신규 `svg_chart.rs`) | 차트 SVG 출력 |
| Serializer | `src/serializer/control.rs` | 라운드트립 직렬화 |
| CLI | `src/main.rs` | `dump`에 차트/OLE 정보 출력 |

## 검증 전략

- **1차 검증**: 로컬 1.hwp(저장소 외부, 저작권 이슈로 samples/ 포함 금지) export-svg → 차트 위치에 막대/선 그래프 출력
- **2차 검증**: 저작권 문제 없는 차트 샘플 자체 제작(한컴 오피스로 간단 차트 작성) → samples/chart-*.hwp로 커밋
- **3차 검증**: `ir-diff`로 차트 포함 HWPX/HWP 비교 (가능 시)
- **회귀 테스트**: 기존 samples/ 전체 export-svg 재실행, 사각형 대체 로직 의존 테스트 확인
- **단위 테스트**: shape_chart.rs / shape_ole.rs 각각 고정 바이트 테스트 (바이트 픽스처는 저작권 문제 없는 자체 제작 파일에서 추출)

## 구현 단계 (초안)

| 단계 | 내용 | 상태 |
|------|------|------|
| 1 | CHART_DATA 레코드 스펙 조사 + IR 설계 (코드 변경 없음, 문서만) | ✅ 완료 |
| 2 | Model 계층: ChartShape / OleShape 구조체 + ShapeObject enum 확장 | ✅ 완료 |
| 3 | Parser 계층: shape_tag_id 분기 추가, CHART_DATA/OLE 감지 | ✅ 완료 |
| 4 | Renderer 계층: placeholder SVG + 라벨 | ✅ 완료 |
| 5 | 기존 samples 회귀 + 단계1~4 최종 보고서 | ✅ 완료 |
| 6 | BinData 해제 인프라 (zlib raw deflate + DocInfo BinDataItem 플래그 + API) | ✅ 완료 |
| 7 | 내부 CFB 파싱 (OlePres000 / OOXMLChartContents / Contents 추출) | ✅ 완료 |
| 8 | OOXML 차트 네이티브 SVG 렌더 + EMF 폴백 | ✅ 완료 |
| **9** | **EMF 스펙 조사 + IR 설계** (문서만) | 신규 |
| **10** | **EMF 모듈 골격 + 헤더 파서** | 신규 |
| **11** | **객체/상태 레코드 파서** (펜/브러시/폰트, DC 스택) | 신규 |
| **12** | **드로잉 레코드 + SVG 컨버터 1차** | 신규 |
| **13** | **텍스트·비트맵 레코드** | 신규 |
| **14** | **shape_layout 통합 + 회귀 + 최종 보고서** | 신규 |

상세 구현 단계는 **구현계획서(`task_195_impl.md`)**에서 확정한다.

## 리스크

- CHART_DATA 바이너리 스펙이 HWP 공식 문서에 부분적으로만 기술됨 → hwplib/pyhwp 구현 참조 필요
- OLE 프리뷰 이미지 포맷 다양성 (WMF/EMF/PNG/BMP) — WMF/EMF는 rhwp에 기존 WMF 파서 존재 확인 필요
- 차트 종류가 많아 1차 범위를 막대/선/파이로 제한

## 승인 요청 항목

1. 범위(차트 + OLE 프리뷰)로 진행 가능한지
2. ~~마일스톤~~ → **미지정**으로 확정
3. ~~1.hwp samples/ 포함~~ → **저작권 이슈로 제외 확정**. samples/에는 자체 제작한 차트 HWP만 추가
4. 승인 시 구현계획서 작성 단계로 진행
