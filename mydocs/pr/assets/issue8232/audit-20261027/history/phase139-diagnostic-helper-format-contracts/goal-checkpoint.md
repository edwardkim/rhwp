---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase139-diagnostic-helper-format-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase139-diagnostic-helper-format-contracts/goal-checkpoint.md` · 원문 SHA-256: `4baff3f64972545d4f2def2918c290de6024562d21bc17859c07da900ca0d3b0`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 139차 기록

이전 단계에서 AST 문맥만 기록했던 helper 32개를 직접 판정했다. 공식 HWP5 개정 1.3 PDF의 해당 표와 현재 정오표, 독립 CFB/Deflate/record 판독 11개 원본을 대조했다. 추가 빌드나 Native 재실행은 없다.

5개 helper에 보정이 필요하다. TABLE 간격 2바이트를 4바이트로 읽어 실제 0HU가 33,423,360이 되고, 행별 값 시작 오프셋도 2바이트 어긋난다. LineSeg 높이를 가로 위치로 읽고, 문단 Instance ID의 상위 16비트를 버리며, 표 확장 제어 payload를 본문 문자처럼 표시한다. 또 실제 assertion용 tag 개수 helper가 parser 오류를 정상 0개로 바꾼다. 제품 renderer 결함 수와 구분하며 이 진단값을 회귀 기대값으로 재사용하지 않는다.

원본 byte/제어 태그/논리 소유/hex 표시는 px 핀과 구분했다. helper 본문 AST와 출력문을 제외한 핵심 계산·분기를 확인했으며 모든 진단 decoder의 전체 형식 적합성을 주장하지 않는다. 실제 보정과 issue_8232 새 명명은 전수 조사 뒤에 진행한다.

누적 직접 점검 12,246개 함수·Rust 검사 7,327개, 보정 필요 159개 함수 항목이다. 미점검 9,441개·증적 대기 573개·추가 함수 판정 문맥 163개가 남아 전수 조사는 미완료다. 사용자 Studio JS·원본/PDF·공유 target을 보존했고 제품/test source·지침·원격 이슈는 변경하지 않았다.
