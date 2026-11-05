# PR #8484 검토 — fix(table): 계산식 LOG·LN·DEGTORAD·RADTODEG·ROUND를 한/글 뜻에 맞춘다

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 head를 고정하고 접수·사전 조사를 완료했다. 이 기록은 기여 거절이나 수용 완료가 아니며, 누적 보정·필수 로컬/시각 검증과 최종 exact-head CI·원격 승인 조건을 완료한 뒤 판정한다.

## 접수 정보

| 항목 | 확인값 |
| --- | --- |
| 원 PR·작성자·base | [#8484](https://github.com/edwardkim/rhwp/pull/8484) / semanticist21 / devel |
| 원 repository·branch | `semanticist21/rhwp` / `fix/table-formula-hancom-functions` |
| 원 head | `eff6d08cfc2bf781a934f358250d676a4fc775cd` |
| 고정한 최신 devel | `ed057276c1e0e07e743218d95641e015b46b6715` |
| reviewer·API 확인 | `edwardkim` / `2026-11-05T01:41:09.772554+00:00` UTC / `requested_reviewers.users` 확인·지정 전후 head 동일 |
| 규모·당시 mergeability | +125 / -5 / 4 files / `MERGEABLE`·`BLOCKED` (조회 시점 참고값) |
| 검토 branch | `review/integration-20261105-semanticist21-next10` |

기본 경로는 maintainer 일반이며 접수·로컬 검증·다수 PR·시각 증적·후속 처리를 적용한다. 원 PR head·fork branch를 변경하지 않았다. [전체 범위·CI 근거·충돌 보정안](semanticist21_20261105_next10_review_plan.md)을 따른다.

## 원 기여 commit과 변경 범위

- `19745abf26050a5390c2b8e91627adcad24e8cad` — fix(table): 계산식 LOG·LN·DEGTORAD·RADTODEG·ROUND를 한/글 뜻에 맞춘다
- `eff6d08cfc2bf781a934f358250d676a4fc775cd` — test(table): 계산식 함수 값을 한/글에서 계산한 인쇄 PDF를 남긴다

제품 변경 경로: `rhwp-studio/src/ui/formula-dialog.ts`, `src/document_core/table_calc/evaluator.rs`.

기준 devel에서 기존 해결 여부는 코드 존재만으로 확정하지 않고 실제 원본·제품 API와 대조한다. 현재는 본문·실제 diff·독립 자료와 기존 원 해결 이력의 조사를 진행 중이다. 공통 조판 원칙과 실제 측정/배치/저장 소비 경로의 충족 여부는 미검증이며, 해당 검증을 완료하기 전에는 수용하지 않는다.

## CI와 병합 시뮬레이션

Archive C의 실패는 `issue_6612_sample14_wmf_pictures_reach_pdf`의 기대 21·실제 101이었다. 계산식 함수 실패가 아니다. #6612 → #6618 → `7946a457d9` → #8187의 `8c97771760`과 독립 글꼴 대조 근거를 확인했으며 해당 보정이 현재 base에 포함되어 있다. 현재 통합 후보에서의 실행은 아직 완료하지 않았다.

- `git merge-tree --write-tree --name-only` exit: `0`.
- 계산 tree: `ce515c10473afcb0627819ca6638f57a8b606333`.
- 충돌: 개별 merge-tree는 충돌 없이 종료했다. 누적 적용은 미실행이다.

CI 실패의 메인터너 처리 판단은 작업지시자의 위임 범위에서 수행했다. workflow 취소·재실행·정책 변경이나 source 원격 push는 수행하지 않았다. 두 충돌 PR의 보정은 계획서의 방침 승인 뒤 진행한다.

## 입력과 검증 상태

- `pdf/table_formula_hancom_functions/functions-hancom-mac-12.30.pdf`

원 head의 실제 Git blob·SHA-256과 기준 devel의 동일 파일 여부를 대조했다. 입력/PDF를 실행한 뒤에는 실행 파일과 검토 commit의 동일성 및 Print 출처·쪽수를 다시 확인한다. 위 목록은 원 PR 변경 목록이며, 누락된 실제 재현 입력·독립 Print 또는 저장 왕복 자료의 충분성을 보증하지 않는다.

집중/전체 회귀·Native/fresh WASM·저장 재열기·전체 TSV·직접 확대 판독은 아직 완료하지 않았다. 기존 contributor 측 숫자·화면·SVG 동일성 주장을 이번 통합 후보의 시각 통과로 기록하지 않는다. 적용 범위와 실제 결과를 확보한 뒤 검증 절을 확정한다.

## Merge 후 contributor PR comment 계획

아직 확정하지 않았다. 실제 원 PR이 주장한 문서·페이지의 Native/fresh WASM review·overlay와 지표·남은 차이를 확인한 뒤 안정 경로와 SHA 고정 URL을 이 절에 기록한다. 이미지·해시·원본/쪽 대응을 게시 전에, API 본문·실제 화면을 게시 뒤 확인한다. 이 접수 기록을 완료 댓글로 게시하지 않는다.
