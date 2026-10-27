---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase171-oracle-probe-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase171-oracle-probe-contracts/goal-checkpoint.md` · 원문 SHA-256: `e80b91eee88ece7b338b419d45cbd4d7e0b30ed89c96ad298ee0fed3a63b73e9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 171차 기록

Oracle probe 검사 117개와 helper 10개를 직접 판정했다. 기존 Python 117 PASS/0 FAIL/0 skip이며 검토한 Python CLI만 4회 실행했다. 실제 읽은 source 7개가 변하지 않았고 rhwp 바이너리·외부망·빌드·시각 캡처는 추가하지 않았다.

올바른 helper 단언과 실제 소비 경로를 구분했다. bool과 숫자는 정규화에서 구분되지만 두 번/n번/묶음 소비는 Python 등호로 같은 것으로 통과시켰다. 현재 기준 조립기는 mixed input/sub를 모두 치환하지만 probe는 input을 남기고 ok를 보고했다. 현재 실제 코드에 대한 독립 소비 검사 2개가 실제로 FAIL했고, 기존 117 PASS와 별도로 기록했다. 올바른 기존 단언은 유지하고 이 두 소비 검사를 보완해야 한다. 보정 필요 누계는 204개다.

NaN 정규화·0byte 파일 존재·자기점검의 의도된 음성 결과를 실제 오류로 오인하지 않았다. 전수 조사는 미완료이며 166차 시험지 전쪽 수집은 별도로 실행 중이다. 최종 수집 결과는 현재 원장에 병합한다. 제품/테스트 수정·개명·외부 게시는 없다.
