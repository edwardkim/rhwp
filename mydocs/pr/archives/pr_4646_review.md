---
kind: pr-review
status: local-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4646 검토 - 썸네일 압축 해제 출력 상한

## 판정

로컬 수용. HWP/HWPX preview를 소비하는 public 경계가 10 MiB 상한을 선택하고, CFB·ZIP·stream
도우미는 호출자가 전달한 상한만 기계적으로 집행한다. 초과 preview는 문서 열기를 실패시키지 않고
선택적 썸네일만 생략한다.

## 검토 기준

- 원격 head: `e83353608c4514e5c3aa041c2f5b59aa28e0d382`
- 로컬 누적 검토 브랜치: `review/humdrum00001010-20260812`
- 적용 순서: #4645 다음에 #4646의 9개 commit을 적용했다.

## 확인

- `node --test rhwp-shared/sw/thumbnail-decompression.test.js`: 6 passed.
- Chrome·Firefox·Safari의 thumbnail consumer가 같은 공용 bounded stream helper를 import하는 것을 확인했다.
- 통합 Studio test 870 passed, production build 통과.
- 통합 전체 Rust 회귀: 5,906 passed, 37 skipped.

## 범위

본문 parsing, renderer, layout은 변경하지 않는다. ZIP central-directory 선언 크기, 실제 compressed span,
실제 decompression 결과가 모두 caller 상한과 일치해야 preview를 반환하는 fail-closed 계약이다.
