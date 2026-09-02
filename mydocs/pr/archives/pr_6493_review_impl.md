---
kind: pr-review-implementation
status: approved
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6493
issue: 4969
---

# PR #6493 구현 검토 - #4969 W10 Q2-D5부터 Q4까지

## 제출 계보

1. Q2-D5에서 exact source 준비·resource transport·no-LineSeg atomic shaping을 단계별로 활성화했다.
2. Q3에서 variable instance를 dormant measurement에서 explicit request, WASM adapter, bounded composer,
   atomic portable publication 순으로 전진시켰다.
3. Q4에서 vertical oracle·intent·geometry를 고정한 뒤 exact source owner, table-cell layout, leaf
   publication, CanvasKit feature detection·pixel proof 순으로 활성화했다.
4. D5에서 correctness, runtime parity, 9-process 성능, clean-room causal WASM 크기를 별도 축으로
   측정하고 `qualified-bounded-subset`으로 판정했다.
5. 최신 `upstream/devel@3b301f725`를 포함한 head를 전체 로컬 게이트로 재검증한 뒤 원격
   `task_m100_4969_w10`과 PR #6493을 생성했다.
6. 첫 CI가 source-side test 증가 정책을 탐지해, 두 cache guard를 기존 #4969 integration source의
   공개 출력 계약으로 이동하고 correction head `7a67f2508`을 push했다.
7. correction head의 Full CI 30개가 모두 성공하고 4개가 정책상 skip됐으며, `MERGEABLE/CLEAN`과
   최신 base 포함·merge-tree 무충돌을 확인했다.

## 보호 불변식

- exact source와 지원 capability가 모두 확인되기 전에는 geometry나 glyph publication을 바꾸지 않는다.
- 한 run의 measurement, bbox, next-origin, sidecar는 함께 commit하거나 모두 fallback한다.
- unsupported·malformed·resource-missing·bounded-limit 결과는 기존 TextRun을 유지한다.
- backend는 게시된 glyph와 position을 replay하며 자체 reshape하지 않는다.
- cache와 입력 크기는 entry, bytes, glyphs, clusters의 독립 상한을 넘지 않는다.
- HWP/HWPX version 분기가 아니라 실제 vertical tuple과 LineSeg 유효성을 탐지한다.

## 검토 초점

- source registry와 page/layer cache의 pointer identity 및 resource multiplicity
- variable instance의 default/explicit/clear 가역성
- vertical source-node mapping, atomic leaf alternative, CanvasKit capability selector
- malformed vertical tuple rejection 및 기존 조판 fallback
- native/WASM/Studio 사이 schema·pixel parity
- 성능 개선과 WASM 크기 증가의 동일-tree causal attribution

## 후속 범위

- W10-Q5/Q6은 #4969에 남겨 별도 계획·승인·검증 단위로 진행한다.
- PR #6493 병합 뒤에는 merge commit ancestry, issue의 open 상태, 원격·로컬 branch 정리를
  `post_merge.md`에 따라 확인한다.
- 코드 후보 Full CI는 성공했다. trailing review-only CI와 최신 mergeability가 확인되기 전에는
  병합하지 않는다.
