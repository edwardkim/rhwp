---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase235-gym-object-layout-serialization-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase235-gym-object-layout-serialization-contracts/goal-checkpoint.md` · 원문 SHA-256: `95e712a6e03e8442b84322506b02c572e6129475d86deebf71767bbeb7b4a5a9`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 235차 기록

개체·조판·코퍼스·직렬화 pack의 원 정적 검사 53개는 모두 PASS다. helper 10개도 실제 소비됐다. 과제/기준풀이 153쌍과 정적 파일 읽기 322개를 대조했다. 기본 검사에는 문서 CLI 실행이 없었고, 후속 probe에만 기존 Native CLI로 두 추출물 생성·두 info 조회를 실행했다. 빌드는 하지 않았다.

CD06/CD10의 재조판 쪽수 핀을 허용하는 예외 1곳이 값·출처를 검사하지 않았다. 원 Python 메서드에 CD10=999를 주입하면 PASS하고, 비예외 LR01=999는 예상대로 FAIL한다. 현재 추출 결과는 17쪽 입력에서 문단 2개를 남긴 CD06이 1쪽, 문단 1개를 남긴 CD10이 2쪽으로 원 핀과 일치한다. 제품이 999쪽을 만든 회귀라는 뜻은 아니다.

원 추출기는 요청 쪽에 걸친 문단 전체를 남겨 재조판하므로 결과 쪽수를 보장하지 않는다. 과거 CD10 기대값을 1→2로 바꾼 기록과 단일 쪽 계약이라는 문서를 연결했다. 이 허용 예외와 연결된 과제 쪽수 핀은 실제 문단 보존·재조판 관계를 보호하도록 보정할 대상이다. 독립 PDF·98% 자격이나 현재 2쪽의 오답을 주장하지 않는다.

field 값·종료 코드·형식·요청 인덱스 및 A=A 변위 0 관계는 유지한다. 병합 스트림 진단으로 초기 JSON 해석이 중단됐을 때 이미 완료된 두 추출을 다시 만들지 않고 나머지 info만 이어서 실행했다.

직접 검토는 15,953→16,016함수, 미검토는 6,400→6,347개다. 보정 지점은 345→346개이며 증거 보류는 396개다. 소스 수정·개명·외부 게시 0이고 전체 감사는 진행 중이다.
