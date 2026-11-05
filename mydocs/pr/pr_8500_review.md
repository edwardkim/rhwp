# PR #8500 검토 — fix(list): 글머리표·번호·개요 문단의 첫 줄에 한/글처럼 문단 머리 표시를 단다

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 head를 고정하고 접수·사전 조사를 완료했다. 이 기록은 기여 거절이나 수용 완료가 아니며, 누적 보정·필수 로컬/시각 검증과 최종 exact-head CI·원격 승인 조건을 완료한 뒤 판정한다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자·base | [#8500](https://github.com/edwardkim/rhwp/pull/8500) / semanticist21 / devel |
| 원 repository·branch | `semanticist21/rhwp` / `fix/paragraph-head-line-flag` |
| 원 head | `ddba007430877afdc2bc2c11d6fdcf6676c72801` |
| 고정한 최신 devel | `ed057276c1e0e07e743218d95641e015b46b6715` |
| reviewer·API 확인 | `edwardkim` / `2026-11-05T01:41:26.955346+00:00` UTC / `requested_reviewers.users` 확인·지정 전후 head 동일 |
| 규모·당시 mergeability | +214 / -17 / 10 files / `MERGEABLE`·`CLEAN` (조회 시점 참고값) |
| 검토 branch | `review/integration-20261105-semanticist21-next10` |

기본 경로는 maintainer 일반이며 접수·로컬 검증·다수 PR·시각 증적·후속 처리를 적용한다. 원 PR head·fork branch를 변경하지 않았다. [전체 범위·CI 근거·충돌 보정안](semanticist21_20261105_next10_review_plan.md)을 따른다.

## 원 기여 commit과 변경 범위

- `bd8c7677cd424d228094a0ea06912b0e5ffc7b80` — fix(list): 글머리표·번호·개요 문단의 첫 줄에 한/글처럼 문단 머리 표시를 단다
- `ddba007430877afdc2bc2c11d6fdcf6676c72801` — docs(pr): 문단 머리 표시의 한/글 화면을 남긴다

제품 변경 경로: `src/document_core/commands/footnote_ops.rs`, `src/document_core/commands/formatting.rs`, `src/renderer/composer/line_breaking.rs`.

기준 devel에서 기존 해결 여부는 코드 존재만으로 확정하지 않고 실제 원본·제품 API와 대조한다. 현재는 본문·실제 diff·독립 자료와 기존 원 해결 이력의 조사를 진행 중이다. 공통 조판 원칙과 실제 측정/배치/저장 소비 경로의 충족 여부는 미검증이며, 해당 검증을 완료하기 전에는 수용하지 않는다.

## CI와 병합 시뮬레이션

실패 check는 발견하지 않았다. 원 PR의 녹색 결과를 새 통합 후보의 통과로 대신하지 않는다.

- `git merge-tree --write-tree --name-only` exit: `0`.
- 계산 tree: `ffdea2be02c35babec28bd8774c4c56faf0972ba`.
- 충돌: 개별 merge-tree는 충돌 없이 종료했다. 누적 적용은 미실행이다.

CI 실패의 메인터너 처리 판단은 작업지시자의 위임 범위에서 수행했다. workflow 취소·재실행·정책 변경이나 source 원격 push는 수행하지 않았다. 두 충돌 PR의 보정은 계획서의 방침 승인 뒤 진행한다.

## 입력과 검증 상태

- `tests/fixtures/paragraph_head/hancom-heads.hwpx`

원 head의 실제 Git blob·SHA-256과 기준 devel의 동일 파일 여부를 대조했다. 입력/PDF를 실행한 뒤에는 실행 파일과 검토 commit의 동일성 및 Print 출처·쪽수를 다시 확인한다. 위 목록은 원 PR 변경 목록이며, 누락된 실제 재현 입력·독립 Print 또는 저장 왕복 자료의 충분성을 보증하지 않는다.

집중/전체 회귀·Native/fresh WASM·저장 재열기·전체 TSV·직접 확대 판독은 아직 완료하지 않았다. 기존 contributor 측 숫자·화면·SVG 동일성 주장을 이번 통합 후보의 시각 통과로 기록하지 않는다. 적용 범위와 실제 결과를 확보한 뒤 검증 절을 확정한다.

## Merge 후 contributor PR comment 계획

아직 확정하지 않았다. 실제 원 PR이 주장한 문서·페이지의 Native/fresh WASM review·overlay와 지표·남은 차이를 확인한 뒤 안정 경로와 SHA 고정 URL을 이 절에 기록한다. 이미지·해시·원본/쪽 대응을 게시 전에, API 본문·실제 화면을 게시 뒤 확인한다. 이 접수 기록을 완료 댓글로 게시하지 않는다.
