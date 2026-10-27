---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase105-exam-source-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase105-exam-source-contracts/goal-checkpoint.md` · 원문 SHA-256: `570f5d042d385d5cd1ef2ac79b53d97c5eac05015d9fa45a045bfc5019a18833`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 105차 기록

직접 검토 10,782개 함수, Rust 테스트 6,674개/845개 파일이다. 미검토 그룹 10,509개, 증거 대기 565개, 보정 필요 확정 항목 92개로 전수 조사는 미완료다.

시험지 검사 전체 62개 함수 중 신규 61개와 기존 check 1개를 검토했다. 현재 Rust 52PASS/0FAIL/0IGNORE이며 추가 빌드는 없다. 52쌍의 원본·독립 Print PDF SHA와 전쪽 수, 보존 Native/WASM TSV의 원본 hash·선택 쪽 raw를 대조했다. 50개 guard는 과거 양쪽98 이상인 구조 경계 시작점 ±2CSS px였고, 두 eq-002는 관계 검사만 한다. 기존 check의 “구조4변 전체”라는 설명을 실제 x/y 시작점 두 값으로 바로잡았다.

현재 HEAD의 raw98 유지·새 확대 판독은 미검증으로 남겨 과거 수치를 현재 통과로 재사용하지 않는다. Rust PASS도 넘침 진단 부재나 최종 glyph/paint 수용을 뜻하지 않는다. 독립 PDF 텍스트/path 차이 표본은 JSON에 그대로 기록했다.

제품/테스트 수정·이름 변경은 0이며 기존 JS 변경을 보존했다. 미검토·보류 항목 조사 완료 후 최종 이슈 갱신과 필요한 보정·issue_8232 이름 변경을 수행한다.
