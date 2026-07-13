# Task M100 #2225 — 2·3단계 완료 보고: MissingPicture 컨텍스트 분기

- 이슈: #2225 / 브랜치: `local/task2225` / 작성일: 2026-07-12

## 구현 (6파일 +130줄)

1. **의미 노드 신설**: `PlaceholderKind { Ole, MissingPicture }` +
   `PlaceholderNode::missing_picture()` — layout(picture_footnote 2지점)이
   bin 참조 실패 + 외부 경로 없음인 그림을 Image 대신 이 노드로 방출
   (bbox 보존, 판정 근거는 스펙 필드만).
2. **편집 뷰(web_canvas, studio Screen)**: 한컴 편집기식 표시 — 개체 영역
   점선 테두리(#999) + 중앙 그림-없음 아이콘(실선 소박스 + 산/해 픽토그램 +
   붉은 사선). 내장 벡터, 외부 자산 없음.
3. **인쇄 등가(export)**: svg.rs·skia 렌더러에서 MissingPicture 무방출 —
   한컴 인쇄 동작 정합. json.rs 는 kind 문자열 통과(CanvasKit/diag 호환).
4. 기존 OLE placeholder 동작 불변 (kind 기본값 Ole).

## 검증

| 항목 | 결과 |
|------|------|
| export-svg | f0f0f0 placeholder **무방출** (수정 전 방출 — gov363svg 산출물로 실증) |
| export-png | 우상단 회색 픽셀 **0** |
| 정상 그림(로고 bin_id=1) | Image 노드·방출 유지 (과억제 없음) |
| 표적 테스트 신설 | `tests/issue_2225_missing_picture_placeholder.rs` — 트리/SVG/정상그림 3검증. 신설 API 사용으로 수정 전 소스에서는 컴파일 불가(구조 판별) |
| 픽스처 정합 | 데이터 없는 그림을 위치 프로브로 쓰던 단위 테스트 2건 술어 확장(Image|Placeholder — bbox 동일) |
| fmt/clippy/전수 | 통과/0/**3,052/0** / OVR 3샘플 0건 |

## 4단계 판정 자산

- export 억제: `output/poc/issue2225/{svg,png}` (회색 박스 소멸)
- 편집 뷰 표시: WASM 빌드 후 studio에서 "심볼" 필드 위치에 점선+아이콘 확인
  (dev 서버 재기동 필요)
