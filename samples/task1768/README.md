# Task #1768 재현 샘플

## distribution_doc.hwpx
- 출처: 서울 정보소통광장 결재문서(공개) 36381620 — **배포용(distribution) 플래그** 문서
  (`rhwp info` → 배포용: 예), 3쪽.
- 결함(수정 전): HWP5 직렬화 시 배포용 플래그는 복사되나 DISTRIBUTE_DOC_DATA 레코드를
  기록하지 않아, 산출물 재로드가 `InvalidFile("암호 오류: DISTRIBUTE_DOC_DATA 레코드
  없음")` 으로 실패.
- 기대(수정 후): 일반 문서로 강하(배포용 0x04·암호화 0x02 클리어) → 재로드 성공.
- 검증: `cargo test --test issue_1768_distribution_doc_save`
