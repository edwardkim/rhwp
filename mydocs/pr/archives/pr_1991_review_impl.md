# PR #1991 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1991
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1950
- 작성자: planet6897
- 원 head SHA: `01c74c4607f7a2089e3c0f99c57cf15b96684e89`
- 체리픽 commit: `b1cd472b8`

## Stage 1. 메타 확인

완료.

- reviewer `jangster77`를 assign했다.
- base `devel`, 문서 작성 시점 `MERGEABLE/CLEAN`을 확인했다.

## Stage 2. 체리픽

완료.

- #1990 뒤에 적용했다.
- 충돌은 없었다.

## Stage 3. 시각 검증

완료.

- 기준 PDF `pdf/issue1950_hwp3_tab_charoffset-2024.pdf`로 visual sweep을 수행했다.
- SVG/PDF 1/1쪽, `flagged=0/1`.
- 대표 asset은 `mydocs/pr/assets/pr_1991_issue1950_review_003.png`다.

## Stage 4. 로컬 검증

완료.

- targeted test, full integration test, clippy가 통과했다.

## Stage 5. 결론

통합 PR merge 후보. 원 PR에는 기준 PDF visual sweep 결과와 함께 통합 반영 사실을 남긴다.

