---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase226-adapter-probe-dependencies/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase226-adapter-probe-dependencies/goal-checkpoint.md` · 원문 SHA-256: `48e438a8642446e7238b6acef20649f453199527e9639f3914fa6090d404c136`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 226차 기록

HWPX→HWP 생성형 검사 열 개를 격리 cwd에서 빌드 없이 실행했다. 임시 positive 자료가 없으면10 PASS이며,8개는 생성 변형/재로드/쪽수 단언을 전혀 하지 않고2개는 HWP 생성·9쪽 확인 뒤 핵심 payload 비교를 생략했다. 읽힌 파일이 손상되면10개 모두 파싱 FAIL했다. 이10 FAIL은 의도한 오류 대조군이며 제품 회귀나 예상 밖 실패가 아니다.

원 계획은 한컴 읽기 오류를 생성 변형으로 분리하고 TABLE row_sizes를 행별 셀 수와 대조하는 것이다. 그 형식 검증 목적은 유지해야 한다. 이전 보류 설명의 expected_hwp_page_count helper 연결은 실제 source와 달라 정정했다. 이 열 root는 임시 output의 positive와 직접9쪽 단언을 사용한다. 정상 positive의 전체 graft/시각 자격을 새로 완료한 것으로 세지 않는다.

증거 보류440→430개, 보정·정리 함수 지점309→319개다. 같은 파일의 Stage31/52/54 세 root는 보류를 유지한다. 직접 검토15,750개는 중복 증가시키지 않는다. 새 빌드·캡처·소스 수정·개명·외부 게시 없이 전체 감사는 진행 중이다.
