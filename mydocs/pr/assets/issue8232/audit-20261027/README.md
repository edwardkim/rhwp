---
kind: investigation
status: active
canonical: mydocs/pr/assets/issue8232/audit-20261027/README.md
last_verified: 2026-10-27
---

# #8232 조사 결과 증적

이 디렉터리는 PR에 포함할 조사 결과의 정식 경로입니다. `output/issue8232-audit/`는 로그·임시 입력·실행 스크립트·기계 작업 원장 등 재실행용 자료 경로입니다.

- [현재 조사 보고서](<../../../../working/task_m100_8232_stage1.md>)
- [보정 대상 346개 함수](findings.md) · [전체 함수 상태](function-status.tsv)
- [현재 집계](scope-summary.json) · [구조화된 증적 인덱스](index.json)
- [판정 본문 조각](decisions/) · [기계 판독 보정 목록](findings.jsonl)
- [사용자 지시 제외 목록](user-excluded-functions.json)
- [누적 조사 기록](history/) · [이동·원문 hash 원장](report-relocations.json) · [당시 원문](history-original-texts.jsonl)
- [선정 증적의 출처·hash](supporting-evidence.json)

`decisions/*.jsonl`은 함수 한 개당 한 줄이며 현재 판정 본문을 유지합니다. `function-status.tsv`에는 아직 판정하지 않은 함수도 포함합니다. 과거 중복 원장·정규식 검색 메모는 임시 작업 자료로 남기며 현재 판정 대신 사용하지 않습니다.

선정 PNG는 실제 출력에 가시성·선 보존·변환을 달리 적용한 **진단 대조군**입니다. 독립 한컴 Print PDF 또는 정식 Visual Sweep 통과 그림이 아니며, 제품에 자연 발생한 결함으로 보고하지 않습니다. 이 자료를 실제 페이지의 90%·98% 자격으로 재사용하지 않습니다.
