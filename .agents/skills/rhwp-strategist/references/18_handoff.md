# 18 인계 문장

회신은 chief 와 같은 3부다. 기계 원본: `fixtures/handoff.json`.

## 1부 확인한 것

- 목표 문장 (objective)
- corpus 경로, documentCount, mappedCount, failed 파일 목록
- evidenceCount, searchFailures, truncatedSearches 건수
- noEvidenceQuestions
- validate verdict / violationCount
- SWS 도달 레벨 (있으면)
- scaffoldAdvertised

숫자를 꾸미지 않는다. mappedCount 4 / documentCount 5 를 "전부 읽음"으로
쓰지 않는다.

## 2부 산출물

- 통과한 spec.json (또는 deliverable.hwpx)
- evidence.json
- corpus_map.json
- 게이트 판정 JSON
- sws_audit.json (실행된 경우)

실패 납품이 있으면 2부는 "없음"이고 3부에 고칠 목록만 있다.

## 3부 다음

- 실패 문서: 암호 요청 / 손상 사본 재수집
- 0건 질문: 키워드 후보
- 절단: searchLimit 상향 또는 키워드 분할
- L3 이상을 원할 때: 반증 검색을 CLAIM 에 구조화 (별도 판단)
- FDE 로 넘길 증상, Chief 로 되돌릴 큐 항목

## 금지 인계

- "대략 다 읽었습니다" (전수성 모호)
- "시장은 긍정적입니다" (ST-FORECAST)
- 게이트 실패 spec 첨부
- gym 점수

다음: [19_failed_document_ledger.md](19_failed_document_ledger.md).
