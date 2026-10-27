---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase111-flow-and-overlap-relations/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase111-flow-and-overlap-relations/goal-checkpoint.md` · 원문 SHA-256: `4d8d9f3d688153641a8603f316e4d58f12f3259a718f2c95d3fdee367626a19c`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 111차 기록

직접 검토 11,119개 함수, Rust 테스트 6,757개/851개 파일이다. 미검토 10,254개, 증거 대기 567개, 보정 확정 103개 함수 항목으로 전수 조사는 미완료다.

두 모듈 782줄·44개 함수를 읽고 신규 37개와 기존 보류 2개를 판정했다. 기존 Rust는 25 PASS / 0 FAIL / 0 IGNORE다. 같은 HEAD의 기존 suite010/014와 frozen Native를 SHA로 검증해 사용했으며 새 빌드는 없다.

76076 선행 para357 TextRun을 각각 14개 제거한 두 형식의 제어 응답을 원래 기존 Rust root에 주입했는데 추가 1 PASS했다. 이는 검사 검출 공백의 실제 음성 반례이며 제품에서 실제 본문이 누락됐다는 뜻은 아니다. table y518.67의 양 backend 완료 페이지 자격은 별도 미검증으로 남긴다.

HWP CFB/record를 rhwp parser 없이 읽어 p141/142/143의 char shape6=1000HU, 문단 간격120%, 앞뒤 여백0, 저장 LS0을 확인했다. 1000/75×1.2=16px는 source 줄 전진 관계여서 이 보류 1건을 해소했다. 겹침 검사의 합성 bbox 숫자와 실제 페이지 golden도 구분했다.

관측기의 get_charbox 값을 CSS 원점으로 오명명한 필드가 있어 원본 JSON을 보존하고 PDF point bbox로만 정정했다. 수치·입력·기존 제품 검사 결과는 바꾸지 않았고 Rust 재실행 없이 해당 관측 기록만 고쳤다.

소스/테스트 변경·이름 변경은 0이며 기존 JS 변경은 보존했다. 전수 조사와 보류 증거 정리가 끝난 뒤 최종 이슈 갱신·issue_8232 명명의 필요한 보정·실제 재검증을 진행한다.
