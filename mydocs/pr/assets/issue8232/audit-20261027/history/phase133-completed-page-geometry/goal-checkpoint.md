---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase133-completed-page-geometry/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase133-completed-page-geometry/goal-checkpoint.md` · 원문 SHA-256: `68568c82508b5570994f3c2846675102508c67b57990302e625effe8ba933e50`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 133차 기록

완료 페이지 기하의 증적 대기 9개를 판정해 561→552개로 줄였다. 직접 분석 함수 11,939개·Rust 검사 7,107개/873파일·보정 필요 확정 124개는 중복 합산하지 않았다. 미점검 9,661개가 남아 전수 조사는 미완료다.

issue_7902_completed_page_geometry 전체 455줄, 검사 14개·helper 16개를 확인했다. Native 기존 검사 14 PASS/0 FAIL/0 ignore, 현재 Native/fresh WASM 각 15문서의 선택 17쪽 최저 99.14586%/99.14623%다. 독립 Print 기준선 17개·이미지 bbox 10개·실제 벡터 그룹 7개, 원 통합 commit과 동일한 입력/PDF 30개 바이트를 확인했다. ±2px와 내용·소유·포함·순서 관계를 함께 보호하는 정상 검사는 유지한다.

Square와 overfill의 논리 줄은 PDF에 실제 선행 공백이 있다. 첫 가시 글자의 원점으로 대체해 잘못된 검사로 판정하지 않았다. PDFium glyph와 별도 PDF 텍스트 연산자를 대조해 독립 기대값이 맞음을 확인했다. 둥근 예시 Rectangle은 축 정렬 선 성분만이 아니라 단일 닫힌 PDF path로 경계를 검증했다. 관측기를 정정했으며 기대값·Rust 검사·캡처는 재실행하거나 바꾸지 않았다.

8개 접촉판의 68개 2배 ROI와 대표 정본 review/overlay를 직접 읽었다. 실제 raw 점수와 남은 획·AA 차이를 구분한다. 현재 선택 쪽 확인을 원래 19문서 835쪽 전체의 새 수용으로 확대하지 않는다. Native/WASM PNG는 같은 바이트가 아니므로 한쪽 이미지를 양쪽 판독으로 대신하지 않았다.

현재 package·바이너리·source 증명을 재사용해 추가 빌드 0, 제품/test source 수정 0, 이름 변경 0이다. 사용자 Studio JS는 보존했다. 최종 이슈 갱신과 필요한 보정·issue_8232 명명·호출 갱신 및 재검증은 전수 조사 후 계속한다.
