---
name: HWPX→HWP 단순 어댑터의 한계
description: HWPX 출처 IR 을 HWP 직렬화기에 그대로 넣을 수 없음. 어댑터 (필드 보강) 로 부족한 의미적 영역이 광범위
type: project
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
HWPX 파서가 채운 IR ↔ HWP 직렬화기 가정 사이에 단순 어댑터로 메울 수 없는 의미적 영역이 광범위.

**Why**: #178 두 번째 시도 (2026-04-19) 결론. 어댑터로 SectionDef 컨트롤 삽입 + table.raw_ctrl_data 합성 + cell.list_attr bit 16 까지 했고 rhwp 자기 호환은 100% 회복했으나 한컴은 거부. 미해결 영역: SectionDef.flags/column_spacing/raw_ctrl_extra 결손, 첫 문단 char_count/text 동기화 (cell_split_save_corruption.md 와 동일 패턴), DocInfo (FONT/CHAR_SHAPE/PARA_SHAPE/BORDER_FILL) 한컴 기대 형식, FileHeader version/properties.

**How to apply**: HWPX→HWP 변환은 단순 어댑터가 아니라 "HWP 출처 IR 과 동등한 IR 을 만드는 본격 변환기" 가 필요. 다음 작업 (별도 이슈) 의 정체성은 "HWPX→HWP 완전 변환기 (한컴 정상 파일 패턴 모방)" — hwp2hwpx 라이브러리 (`/home/edward/vsworks/hwp2hwpx`, hwplib/hwpxlib 저자, Apache 2.0) 가 권위 있는 매핑 명세 자료. 단순 어댑터 접근으로 다시 시도하지 말 것.

본 시도 자산 (보존됨): `src/document_core/converters/` (어댑터 + 진단 + CommonObjAttr 작성기), `tests/hwpx_to_hwp_adapter.rs` (25개), `examples/hwpx_hwp_ir_diff.rs`. typeset.rs:1582 버그픽스도 보존.

상세: `mydocs/troubleshootings/task178_second_attempt_hancom_rejection.md`, `mydocs/report/task_m100_178_report.md`
