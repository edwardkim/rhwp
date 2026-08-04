---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: hancom_placeholder_print_suppress
description: "한컴 그림 placeholder 동작 — 편집기에서는 표시, 인쇄 시에만 그림 미지정 placeholder 미출력 (2026-07-12 작업지시자 확인)"
metadata: 
  node_type: memory
  type: feedback
---

**한컴의 그림 placeholder(그림 미지정) 컨텍스트 분기** (2026-07-12, 결재문서
직인/심볼 필드 건에서 작업지시자 확인):

- **편집기 화면**: placeholder 가 보이는 것이 정상.
- **인쇄(및 인쇄 등가 출력 — PDF 변환 포함)**: placeholder 에 그림이 지정되지
  않은 경우 **미출력** 처리.

**Why**: 결재문서류의 직인/심볼 필드는 bin_id 미지정(예: 36389312 "심볼" 필드
bin_id=0) 상태로 유통되는 경우가 많다. 한컴 2024 PDF 에는 안 보이는데 rhwp
export 에는 회색 박스가 보이는 차이를 **결함으로 오인하지 않아야** 하고,
역으로 rhwp 의 인쇄 등가 경로(export-svg/png/pdf)는 억제가 한컴 정합이다.

**How to apply**:
1. 시각 판정에서 한컴 PDF 대비 "rhwp 에만 회색 placeholder 박스" 차이는
   편집 뷰 기준으로는 정합 — 결함 목록에서 컨텍스트를 구분해 기재.
2. 정정 설계 시: studio 편집 뷰는 placeholder 유지, export 계열(인쇄 등가)만
   미지정 그림 억제 — 렌더 컨텍스트 분기.

관련: [[feedback_no_inference_authoritative_spec]] [[feedback_visual_judgment_authority]]

**보강 (2026-07-12, 편집기 표시 사양)**: 한컴 편집기의 그림 미지정 placeholder
표시 = **개체 영역 점선 테두리 + 중앙의 작은 그림-없음 아이콘**(사선 그어진
꽃 그림 픽토그램) — 밋밋한 회색 채움 박스가 아니라 편집자 정보 제공용 시각화.
rhwp 편집 뷰 정합 목표도 이 형태다 (#2225).
