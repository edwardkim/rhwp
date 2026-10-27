---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase162-gym-leaderboard-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase162-gym-leaderboard-contracts/goal-checkpoint.md` · 원문 SHA-256: `4f7cb17eff1c0cbc28015220810f1ae016f50a384e94f13ecc8ed3b8b5fc9c73`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 162차 기록

리더보드 검사 136개와 helper 41개를 직접 판정했다. 실제 Python 136 PASS/0 FAIL/0 skip이고 읽은 저장소 파일 21개가 변하지 않았다. 원본 점수판 27개 파일을 보존했다. 없는 임시 바이너리 호출의 OS 오류를 한 번 검증했으며 실제 Rust 실행·빌드는 없다.

머클 마지막 홀수 leaf를 버리는 작성 오류 함수에서 한 잎 검사는 PASS했지만 별도의 세 잎 검사가 실제 FAIL했다. 전체 caller를 확인하여 유효한 보완 검사로 유지하고 이름만으로 보정 대상으로 확대하지 않았다. baseline 실패와 mutant의 예상 실패를 구분했다. 추가 보정은 없고 총 199개 함수 항목이다. 전수 조사·최종 이슈 갱신·소스 보정·개명은 미완료다.
