---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6347
issue: 6337
author: kevin9327
---

# PR #6347 review - oracle pair index helper

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6347
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `774df6aeb77679ba5ca6a44a16c39259c2feeba1`
- Review branch: `review/kevin9327-nondocs-20260829`
- Cherry-pick result: `3073a9260`

## 검토 판단

**수용 권고.** 작은 수동 목록 대신 실제 tree에서 sample/PDF 짝을 산출해 저장소 PDF oracle을
쉽게 사용할 수 있게 한다. #6338과 향후 visual/page-count 조사에 직접적으로 필요하다.

## 증적과 검증

- `python3 -m py_compile tools/fidelity_compare/oracle_pair_index.py` passed.
- `--list` found 566 paired documents, with 96 directory-narrowed matches.
- Representative `--args` output matched the expected same-directory PDF paths for `sungeo` and the two `KTX` variants.
- Current sample paths do not contain a double quote character, so the emitted shell argument examples are valid for the current corpus.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 566개 짝 산출과 같은 디렉터리 충돌 예제 검증을 남긴다.
