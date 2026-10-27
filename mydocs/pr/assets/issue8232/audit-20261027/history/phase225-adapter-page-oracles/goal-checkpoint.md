---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase225-adapter-page-oracles/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase225-adapter-page-oracles/goal-checkpoint.md` · 원문 SHA-256: `f4f55749fa18729f9dacb971f60675b15df2aa56657164600809df4cda580790`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 225차 기록

원본 입력·기준 파일은 보존하고 output 안의 세 격리 cwd에서 기존 Native 검사를 실행했다. 정상 참고 파일8 PASS, 참고 HWP가 없는 경우6 PASS, 손상된 경우6 PASS로20회 모두 PASS다. 이는 원 Rust 검사가 참고 파일 실패를 현재 쪽수 fallback으로 숨기는 실제 관측이며 잘못된 출력 배치를 주입해 통과시킨 재현은 아니다.

현재 엔진 참고값·fallback·대응 내용 보존 보완은 helper3개와 root6개에 연결했다. 한컴이 저장한 HWP 입력과 현재 rhwp가 계산한 쪽수는 독립 Print 기대값이 아니다. 자기 재로드 일관성 및 h-01의 원본 빈 줄/외부 여백/양수 offset 관계는 유지해야 한다. 진단 root 하나는 실제 빈 raw_ctrl_data26개의 저장 전 형식 차이이므로 페이지 결함 유지로 보지 않고 보류를 해소했다.

기존 PDF6개의 쪽수9와 metadata만 확인했으며 현재 전체 시각 통과를 주장하지 않는다. Stage31~54의 생성 변형 보류13개는 그대로다. 전체 보류450→440개, 보정·정리 함수 지점300→309개다. 직접 검토15,750개는 중복 증가시키지 않는다. 새 빌드·캡처·소스 수정·개명·외부 게시 없이 전체 감사는 미완료다.
