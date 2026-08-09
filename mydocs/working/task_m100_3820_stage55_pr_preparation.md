---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 55 — PR 후보 확정과 리베이스 후 게이트

## 목적

`task/3820-3821-fidelity`의 누적 렌더링 보정을 최신 `upstream/devel` 위의 단일 PR 후보로
확정한다. 이 단계는 로컬 커밋·검증·PR 본문 준비까지만 수행한다. 원격 push와 PR 생성은 별도
사용자 승인 뒤에 수행한다.

## 워크플로 라우팅

- 기본 절차: `mydocs/manual/pr_review/collaborator_self_merge.md`
- 보조 절차: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`
- 대형 변경 보조 절차: `rework_and_exceptions.md`의 1,000줄 초과 변경 규정
- PR 번호가 아직 없으므로 `pr_N_*` 검토 문서와 오늘할일 항목은 만들지 않는다.

## 리베이스·후보 상태

- 기준: `upstream/devel` `d9c530ee8ed4bd0830ff35bc47e552bb0f32274f`
- 후보: `86f8d57718258915b9b65f7ccddc1349784f78c8`
- 상태: ahead 69 / behind 0 / clean
- 리베이스: 69개 commit 모두 `git range-diff`에서 `=`로 보존, 충돌 없음
- 원격 head: 없음
- 열린 PR: 없음

변경 규모는 375 files, 190,612 insertions, 5,269 deletions이다. 구현·회귀·실문서 PDF
대조 증적이 누적된 대형 렌더링 PR이므로 merge 전에는 코드 회귀와 native·WASM 경로를 모두
확인한다.

## LFS 판정

- `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`: 19,609,731 bytes
- `pdf/exam_social-current-2020.pdf`: 3,756,350 bytes
- 저장소 LFS 규칙은 `pdf-large/**/*.pdf`에만 적용된다.
- 위 두 PDF는 `pdf/` 아래이고 모두 50 MB 미만이므로 일반 Git 대상이 맞다.
- 변경 파일 중 100 MB 초과 파일은 없다.

## 이슈 종료 판정

- `#3821`: p156 deferred Square 그림의 wrap anchor 전파 결함을 수정하고 focused 회귀와
  PDF 대조를 남겼으므로 `Closes #3821`로 연결한다.
- `#3820`: 2025 편람, 76076, #4090 전수 fidelity 등 후속 후보가 계속 남아 있으므로 닫지
  않고 `Refs #3820`으로 연결한다.
- `#4039`: issue2007 중첩 셀 pagination의 재현·보정·회귀 근거를 포함하지만 이슈가 이미
  닫혀 있으므로 `Refs #4039`로 연결한다.

## 리베이스 전 검증 근거

최신 리베이스가 브랜치의 69개 commit을 동일 patch로 보존하기 전, Stage 54에서 다음을
완료했다.

- `cargo test --profile release-test --tests`: 전체 integration 실패 0
- library: 3,315 passed / 10 ignored
- `overflow_cell_baseline`: 675 fixture, 비영점 17종, 총 691줄, 통과
- Native Skia 공식 3종: 58/58, 2/2, 4/4 통과
- `cargo clippy --all-targets -- -D warnings`: 통과
- `cargo fmt --all -- --check`, `git diff --check`: 통과

이 단계에서는 리베이스된 정확한 PR 후보에 대해 아래 게이트를 다시 실행하고 최종 결과를
기록한다.

## 리베이스 후 검증 계획

1. `cargo build --release`
2. `cargo test --release --lib`
3. `cargo test --profile release-test --tests`
4. Native Skia 공식 회귀 3종
5. `cargo fmt --all -- --check`, `git diff --check`
6. `cargo clippy --all-targets -- -D warnings`
7. `cargo test --doc`
8. `wasm-pack build --target web --out-dir pkg`

Cargo 검증은 다른 작업의 산출물과 섞이지 않도록 이 브랜치 전용 target을 사용하고,
장시간 실행되는 전체 integration은 최종 exit code와 summary까지 기다린다.

## PR readiness 시각 게이트 — 보류

PR 후보를 확정하기 전에 사용자가 `issue2007_nested_cell_pagination_42065.hwp` p11을 다시
확인했다. 리베이스된 정확한 HEAD를 release-test profile로 빌드한 뒤 한컴 PDF p11–p12와
144dpi로 다시 대조했다.

- 입력: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`
- 기준 PDF: `pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf`
- 현재 HEAD build: 성공
- 문서 쪽수: rhwp 17 / PDF 17
- p11 visual accuracy proxy: 6.63872%
- p12 visual accuracy proxy: 6.25419%

현재 rhwp p11은 기준 PDF에는 없는 `3 중앙선거관리위원회` 제목과 다음 표의 점선 상단선을
쪽 하단에 미리 그린다. 그 결과 p12는 기준 PDF의 첫 제목 없이 `공직선거법`부터 시작한다.
PDF 텍스트층도 p11이 `국세기본법`의 마지막 문장으로 끝나고 p12가
`중앙선거관리위원회`로 시작함을 확인했다.

`visual_sweep` 구조 heuristic은 두 페이지를 `flagged=0`으로 놓쳤지만,
`fidelity_compare --layout-ledger`는 p11의 `table_footer=1`과 p11→p12 동일 source 표
fragment를 기록했다. 기존 회귀는 p12의 `중앙선거관리위원회규칙` 문장도 제목 substring으로
세어 잘못 통과할 수 있으므로 exact heading owner assertion이 필요하다.

증적:

- [p11 review](../pr/assets/task_m100_3820_stage55_pr_readiness/review_p011_before.png)
- [p12 review](../pr/assets/task_m100_3820_stage55_pr_readiness/review_p012_before.png)
- [overlay metrics](../pr/assets/task_m100_3820_stage55_pr_readiness/overlay_metrics_before.json)
- [text ledger](../pr/assets/task_m100_3820_stage55_pr_readiness/text_report_before.tsv)
- [layout ledger](../pr/assets/task_m100_3820_stage55_pr_readiness/layout_candidates_before.tsv)

따라서 이 Stage의 결론은 **PR 준비 보류**다. 위 계획의 전체 회귀·Skia·Clippy·WASM은
p11→p12 source owner 결함을 다음 Stage에서 수정한 뒤 새 정확한 HEAD에서 실행한다.

## PR 초안

### 제목

```text
fix(renderer): 중첩 표 페이지네이션과 PDF fidelity를 보정한다
```

### 본문 골격

```markdown
## 변경 내용

- deferred Square 그림의 실제 wrap 대상 문단을 보존해 p156 본문 침범을 제거했습니다.
- HWP/HWPX 중첩 표의 RowBreak, continuation, source owner와 물리 clip 경계를 기준 PDF에
  맞췄습니다.
- issue2007, 59043, #3637, issue1891 실물 fixture의 페이지 소유·overflow 계약을 회귀로
  고정했습니다.
- visual sweep과 fidelity compare가 표 경계 손실, 본문 침범, page-owner drift 후보를 더
  정확히 찾도록 보강하고 페이지별 증적을 함께 남겼습니다.

## 검증

- 리베이스 후 전체 release/integration 회귀
- Native Skia 공식 게이트 3종
- fmt, diff-check, full Clippy, rustdoc
- WASM web build
- 한컴 기준 PDF와 페이지별 visual sweep 증적

Closes #3821
Refs #3820
Refs #4039
```

최종 PR 본문에는 이 단계의 실제 명령별 결과와 후보 SHA를 반영한다.
