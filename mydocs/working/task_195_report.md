# Task #195 최종 보고서 — 차트/OLE 개체 렌더링 지원

> Issue: [#195](https://github.com/edwardkim/rhwp/issues/195)
> 브랜치: `local/task195` (from `local/devel`)
> 마일스톤: 미지정
> 기간: 2026-04-19

## 배경

- 재현 파일 `1.hwp` (외부, 저작권 이슈로 samples/ 포함 금지)
- 증상: 테이블 셀 안 차트 2개가 export-svg 시 완전 빈 사각형으로 출력
- 원인: rhwp의 GSO 파서가 `HWPTAG_SHAPE_COMPONENT_OLE`, `HWPTAG_CHART_DATA`를 미처리 (상수만 정의)

## 범위

- 포함:
  - `ShapeObject::Chart`, `ShapeObject::Ole` variant 추가
  - `HWPTAG_SHAPE_COMPONENT_OLE` 레코드 필드 파싱 (`parse_ole_shape`)
  - `HWPTAG_CHART_DATA` 감지 및 raw 보존(라운드트립)
  - placeholder SVG 렌더 (회색/파란 박스 + 점선)
- 제외(분리된 후속 이슈):
  - OLE 프리뷰 이미지 실제 추출
  - CHART_DATA 하위 태그(80~95) 구조화 파싱 및 차트 시리즈 렌더

## 단계별 진행

| 단계 | 커밋 | 내용 |
|------|------|------|
| 1 | `8c660f0` | 스펙 조사 + IR 설계 (문서만) |
| 2 | `80558bb` | Model 확장 (`ChartShape`/`OleShape`), 8개 매치 사이트 확장 |
| 3 | `2aa5f2f` | Parser (`parse_ole_shape`, 차트 우선 분류, 단위 테스트 3건) |
| 4 | `081df07` | Renderer (placeholder SVG 색상/테두리) |
| 5 | (본 커밋) | 검증 + 최종 보고서 + 오늘할일 갱신 |

## 주요 기술 결정

1. **차트/OLE 재분류 우선순위**: CHART_DATA 존재 시 Chart 우선, 없으면 OLE 태그로 분류
2. **라운드트립 전략**: IR 필드가 비어 있어도 `raw_chart_data` / `raw_tag_data`로 원본 바이트 보존 → 읽기·저장 시 손실 없음
3. **placeholder 스타일**: 기존 fill/stroke가 있으면 유지, 없을 때만 회색/파란 배경 오버라이드 → 사용자 의도 보존
4. **1.hwp 조사 결과**: 네이티브 HWP CHART_DATA가 아니라 MS Graph OLE 임베드 → 실무에서 OLE 경로가 더 빈번. OLE 렌더가 실질적 수혜

## 검증

- 단위 테스트: 878 passed (신규 3건 포함), 회귀 0
- tests/: 13 passed
- 1.hwp 수동 검증: 2개 OLE 차트 placeholder 정상 렌더 (`/tmp/task195_out/1_004.svg`에 `fill="#f0f0f0"` + `stroke-dasharray` 확인)
- 회귀: samples/draw-group.hwp, aift.hwp, biz_plan.hwp export-svg 크래시 없음

## 변경 파일 목록

```
src/model/shape.rs                        (+137)  enum + 신규 struct
src/renderer/layout/shape_layout.rs       ( +53)  Chart/Ole 렌더 arm
src/serializer/control.rs                 ( +68)  라운드트립 arm 3곳
src/document_core/commands/object_ops.rs  ( +10)  shape_attr 매치 확장 3곳
src/main.rs                               ( +11)  dump 출력
src/parser/control/shape.rs               ( +99)  parse_ole_shape + 분기 + 테스트
src/parser/control/tests.rs               (  +4)  테스트 헬퍼
mydocs/plans/task_195.md                  (신규)  수행계획서
mydocs/plans/task_195_impl.md             (신규)  구현계획서
mydocs/tech/hwp_chart_spec.md             (신규)  차트 스펙
mydocs/tech/hwp_ole_spec.md               (신규)  OLE 스펙
mydocs/working/task_195_stage1~5.md       (신규)  단계별 보고서
mydocs/working/task_195_report.md         (본 문서)
mydocs/orders/20260419.md                 (갱신)  오늘할일
```

## 분리 제안 후속 이슈

1. OLE 프리뷰 이미지 추출 (BinData 압축 해제 + CFB 파싱 + WMF/EMF→SVG)
2. CHART_DATA 하위 태그 파싱 + 차트 시리즈 실제 렌더
3. 자체 제작 `samples/chart-basic.hwp` 추가 (한컴오피스 authoring 환경 필요)

## 승인 후 프로세스

- `local/task195` → `local/devel` merge (`--no-ff`)
- `local/devel` → `origin/devel` push
- GitHub Issue #195 close
