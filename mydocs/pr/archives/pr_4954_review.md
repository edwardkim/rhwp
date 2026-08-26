---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4954 검토 - loadsave 실패 키 재측정

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4954](https://github.com/edwardkim/rhwp/pull/4954) |
| 작성자 / source | @planet6897 / `fix/4899-oracle-retry` |
| 원 source head | `600571bc330b1a3072852e9f744c8dd469a174b3` |
| 기준 devel | `418e5b191d23cf0618ce99f0cfec332c19ac1bc2` |
| 통합 branch / local 적용 | `review/non-draft-20260816` / `c89e77e89` |
| 작성 시점 원 PR 상태 | `OPEN` / `MERGEABLE` / `CLEAN`; merge 전 재확인 필요 |

한글 COM 환경의 일시 불안정으로 생긴 실패 행을 깨끗한 worker에서 한 번 재측정하고, 성공한 행만 BOM 없는
UTF-8로 교체하며 원래 실패와 재측정 결과를 `retried.tsv`로 남긴다.

## 검증과 판단

Windows PowerShell에서 실제 바이트 기준 구문 파싱, 실패 키 선별, BOM 없는 UTF-8 재기록 helper를 임시 TSV로
검증해 통과했다. 최신 `upstream/devel` merge-tree도 충돌·whitespace 오류 없이 깨끗했다. HKCU COM 설정 변경과
`Hwp.exe` 강제 종료를 수반하는 한글 COM 전수 실행은 검증 전용 범위를 벗어나 실행하지 않았다. 이는 미실행
범위이지 성공으로 기록하지 않는다. **통합 수용 권고.**
