# PR #6858 접수·self-review 기록

- PR: [#6858](https://github.com/edwardkim/rhwp/pull/6858).
- Issue: [#6852](https://github.com/edwardkim/rhwp/issues/6852).
- 작성일: 2026-09-08. **최신 제출 HEAD CI 성공 확인·self-review 승인. 리뷰 기록 push·병합은 미실행.**

## 라우팅과 접수

- base route: `collaborator_self_merge.md`의 본인 PR 절차. 작성자 `edwardkim`이며 외부 contributor PR이 아니다.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서,
  `codex/docs_and_git_workflow.md`, `github_operations.md` O1 metadata 절차.
- 본인 PR이므로 reviewer를 별도 지정하지 않는다. GitHub approve를 수행한 것이 아니다.

| 항목 | 접수 시점 참고값 |
| --- | --- |
| 작성자·assignee | `edwardkim` |
| base / head branch | `devel` / `task_m100_6852_baseline` |
| 최초 제출 HEAD | `aa3cc0bfd58a7ebd9387c5008e6c2d0d60175d76` |
| 전체 로컬 검증 후보 | `393401e5a` (그 뒤 최초 제출까지 문서만 변경) |
| 최신 base | `7138fe784`, fetch·merge-tree clean |
| 규모 | 최초 제출 9 files, +755/-23. 이 접수 기록 추가 전 수치 |
| 상태 | 접수 당시 Open, MERGEABLE / BLOCKED. 아래 self-review 시점에는 MERGEABLE / CLEAN |
| milestone | `v1.0.0` |
| labels | `bug`, `rust`, `rendering`, `layout`, `hwp5`, `hwpx` |

head SHA·CI·merge 상태는 변할 수 있으며 최종 판단 시 다시 조회한다. 등록 전 같은 head branch의
기존 PR이 없음을 확인했다. #6852 본문·제목·라벨은 승인된 실제 수용 범위로 정정하고 게시 후 재조회했다.

## 범위와 검증 증거

일반 흰 사각형을 마스크로 추정하던 B 선 억제를 제거한다. 원본 선 없음·그림자·좌표를 보존한다.
제품 변경은 `shape_layout.rs`, 새 회귀는 `tests/cases/issue_6852_group_rectangle_stroke.rs`다.
새 HWPX와 해당 문단 ID 정규화 원장 네 행을 포함한다. generated suite·manifest·Cargo 파생물은 없다.
그룹 API·Studio UI 확장은 제외하며, IR 구조 보존·A 재검토는 #6856에서 처리한다.

완료한 명령과 상세 결과는 [Stage 3](../../working/task_m100_6852_stage3.md),
수용 범위는 [결과보고서](../../report/task_m100_6852_report.md)에 있다.

- fmt, native/WASM32/workspace all-target Clippy, workspace build, manifest를 통과했다.
- 전체 nextest 9,224 PASS / 0 FAIL / 46 skipped. 신규 HWPX를 명시한 보안 검사도 통과했다.
- Native Skia lib 4,112 PASS / 13 ignored, 그림 2건·직접 PDF 4건을 통과했다.
- Docker 최적화 WASM을 빌드하고 실제 Vite HTTP 제공 파일의 hash 일치를 확인했다.
- 실제 HWP/HWPX 두 원본의 1·5·7쪽 WASM SVG는 기존 검증 CLI와 모두 바이트 동일하다.
- 신규 sample IR 원장 최초 실패는 수정 전후 모두 같은 626개 문단 ID 바이트 재부여로 확인했다.
  두 번째 왕복의 차이 0을 확인한 뒤 네 행만 등록하고 전체 회귀를 다시 통과했다.
- clipping controlset 외부 원본 92개가 이 환경에 없어 해당 gate 통과는 주장하지 않는다.
  기존 선 보정 A의 정당성, 전체 문서의 한컴 완전 일치도 주장하지 않는다.

## 최신 HEAD 확인과 self-review

검토한 원격 HEAD는 `7887e5c6683e26a469f4bb114c4e8559411b5949`다.
전체 로컬 검증 후보 `393401e5a` 이후에는 mydocs 문서만 변경되었다.
최신 `upstream/devel`은 `7138fe7848a0cda157bce33f73c9d9d7e977209a`이며
fetch 후 merge-tree clean, GitHub에서도 MERGEABLE / CLEAN을 확인했다.

- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/34173615768): 성공.
  Lint, Native Skia, frontend package, build/test archive와 Build & Test를 확인했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34173615743): JS·Python·Rust 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34173615671),
  [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34173615778),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34173615760): 성공.
- CI Impact Policy도 성공이며 의도된 skipped job 외 실패·대기 check가 없었다.
- 소스 검토: 일반 흰 사각형의 B 억제만 제거했고, 기존 A는 글상자가 있는 경로에 남는다.
  선 없음의 실선 강제, 기하·그림자 변경 또는 API/UI 확장은 없다.
- 회귀 검토: 실제 HWP/HWPX와 선 없음·비채움·빈/공백 글상자·기존 A를 검사한다.
  SVG clipPath 사각형은 대상에서 제외한다. IR 원장은 수정 전후 동일한 네 경로만 등록했다.
- 검증 후보의 기존 review worktree와 target를 재사용하여 집중 회귀를 재실행했다.
  `node scripts/run-rust-test.mjs issue_6852_group_rectangle_stroke -- --cargo-profile release-test
  --target-dir /home/edward/mygithub/rhwp-6812-review-target --no-fail-fast` 결과는
  **6 PASS / 0 FAIL**, 필터 제외 180건이다.
  로컬 로그: `output/6852/stage3/self-review-focused.log`.
  제품 코드 동일성과 최신 Full CI 성공을 근거로 전체 회귀는 다시 반복하지 않았다.
- 새 integration 원본만 제출하며 generated suite·manifest·Cargo 파생물과 private corpus는 없다.

## 시각 증거와 후속 기록 계획

renderer와 실제 HWPX fixture를 변경하므로 시각 검증 대상이다. 메인테이너는 원본 5쪽 앞쪽
사각형 실선 복원을 SVG로 직접 수용했다. 새 WASM의 해당 SVG도 같은 hash
`81312df7ceace805dd48154b79d55791f4c506f026d3858befbc7db5ed875e9c`다.
별도 픽셀 sweep 점수를 새로 측정하지 않았으므로 pixel_match·visual_accuracy_proxy_percent는 미측정이다.
표준 비교·게시 정본은 [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)다.

### Merge 후 contributor PR comment 계획

본인 PR이므로 contributor 감사 대신 실제 수정·판정·잔여 범위를 기록한다. 게시 자체는 별도 승인 대상이다.
메인테이너 시각 수용 범위는 원본 5쪽이며, 일반 흰 사각형 stroke 외 다른 SVG 불변 및 11쪽 유지가 근거다.
해당 SVG를 `rsvg-convert --zoom 2 --background-color white`로 래스터화하여
대표 PNG를 [pr_6858_page5_after.png](../assets/pr_6858_page5_after.png)에 보존했다.
직접 열어 소제목 앞 흰 사각형의 위·왼쪽 선과 뒤쪽 그림자를 식별할 수 있음을 확인했다.
PNG는 1588×2246이며 SHA-256은
`7c5a18a392821e639efd46fecbf2213529eb96d6bd4481c5bf6c915ef34fafe0`이다.
이는 수용한 SVG의 보존용 이미지이지 독립적인 한컴 비교나 새 픽셀 계측이 아니다.
원격 게시 시에는 merge 후 `<merge-commit-sha>` 고정 raw URL을 사용한다.
비교 방법 정본 링크와 픽셀 점수 미측정·전체 fidelity 보증 아님을 함께 명시한다.

## 최종 판정

**Self-review 승인.** 수용한 최소 수정 범위에서 병합을 막을 새 결함을 발견하지 않았다.
이 판정은 #6856의 IR 구조 보존이나 기존 A 조건을 승인하는 것이 아니다.
리뷰 문서·대표 PNG는 로컬 커밋으로 남기며 아직 원격에 반영하지 않았다.
남은 절차는 승인된 기록 push → 새 원격 HEAD checks 확인 → 별도 승인된 병합이다.
승인 없이 merge·issue close·브랜치 삭제는 하지 않는다. 일반 merge commit 방식을 유지한다.
