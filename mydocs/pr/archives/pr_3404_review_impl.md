# kevin9327 PR #3404–#3449 통합 검토·구현 기록

## 라우팅과 범위

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, visual_fixture_evidence,
  multi_pr_update_branch, rework_and_exceptions, review_only_fast_pass
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md,
  rework_and_exceptions.md, review_only_fast_pass.md
```

`upstream/devel`의 `732147a30cf122839afae59c99c91f7854e2f3f2`에서 사용자가 볼 수 있는
`review/kevin9327-20260726-v2`를 만들고, 검토 중 이동한 최신 devel
`7f8fcfef08610df7bf9f5cc2f4b32a9a711f5e2d`를 merge commit `3aa29c2d3`으로 동기화했다.
작업지시자가 지정한 `@kevin9327`의 open PR 가운데
#3399를 제외한 아래 10건을 원 source 순서대로 누적했다. #3452는 다른 contributor의 PR이므로
작업지시자 정정에 따라 이 통합 범위에 넣지 않았다. 원 PR branch는 수정하지 않았다.

PR 상태·head SHA·CI는 문서 작성 시점의 참고값이다. merge 전에는 통합 PR의 최신 head와 원 PR의
source head가 달라지지 않았는지 다시 확인한다.

| 원 PR | source head (작성 시점 참고값) | 통합 commit | 판정 |
| --- | --- | --- | --- |
| #3404 | `032e263f6c21` | `2883e3b77`, `ed2cda794`, `1accb0a06` | 문서 실측값 보정 후 수용 |
| #3408 | `db6ef6b0fa95` | `4cc1c80d1` | TSV·README 계약 보정 후 수용 |
| #3419 | `46ddd3adfdab` | `97981fea0` | conflict·줄끝 churn 제외 및 기능 보정 후 수용 |
| #3420 | `167c9dabece7` | `f8e0c37fd`, `79ea4e79e` | 중첩 갱신·회귀 범위 보정 후 수용 |
| #3426 | `803e1cea42c7` | `e91e8ad93`, `e8b1e672f`, `360e3a478` | 직접 회귀 보강 후 수용 |
| #3428 | `940b915435c8` | `3c9ae89b1` | 중첩 수식 검색·실제 변경 검증 보정 후 수용 |
| #3430 | `6c652bdde77d` | `e69a2d286`, `b7ae99580` | 총쪽수 상태·재귀 왕복 보정 후 수용 |
| #3447 | `f9f3073472f6` | `0db1f0ee4`, `8acec32ff`, `96eaa2288`, `5e75b5dcb` | 실제 완성본과 문서 계약 정합 후 수용 |
| #3448 | `e7179e214990` | `a6efc3cf0`, `6cc0c1e66` | CLI 오류·출력 계약 보정 후 수용 |
| #3449 | `8a52a36b6997` | `78f27671f`, `10bc93cfd` | #2439 무회귀 조건으로 범위 축소 후 수용 |

## 체리픽·conflict 처리

- contributor commit은 작성자와 원 SHA provenance를 보존해 최신 devel 위에 적용했다.
- #3419 source는 대형 Rust 파일의 CRLF/LF 변환과 최신 devel conflict가 섞여 있었다. 비기능 줄끝
  churn과 현재 devel의 WMF fallback을 덮는 부분은 제외하고, `EquationNode.script`와 생성·수집·회귀
  테스트의 의미 변경만 `97981fea0`에 보존했다.
- #3447의 마지막 contributor commit은 원 PR의 최종 문서 상태를 반영하기 위해 다른 source 적용 뒤
  누적됐지만, contributor 이력과 완성본 내용은 그대로 유지했다.
- 통합 중 원 PR에 직접 push, rebase, amend 또는 force-push하지 않았다.

## 메인터너 보정

통합 검토에서 발견한 차단 누락은 contributor commit과 분리한
`a1fe4ce760899f4ad0b12bc5fbddf808611e9dd5`로 보정했다.

- #3408: 271행 TSV를 실제 8열 계약과 LF로 정규화하고, 전체 페이지 title 추출을 포함한 2-pass
  절차와 집계값을 README에 맞췄다.
- #3419: Markdown 표 셀의 Equation script 수집, skip 없는 표준 CLI test binary 탐색, Native Skia
  생성자의 `script` 필드를 보강했다.
- #3420: 표 셀 안 중첩 머리말·꼬리말의 carry 상태를 갱신하고 선택과목 여섯 쪽을 모두 고정했다.
- #3426: HWPX chart의 `shapeComment`를 직접 검사하는 회귀를 추가했다.
- #3428: 표 셀·글상자 수식의 search/replace, 실제 mutation count와 dry-run parity를 검증했다.
- #3430: `TotalPage` counter를 현재 쪽번호와 분리하고 문서마다 reset되는 재귀 왕복 계약을 추가했다.
- #3447: 실제 `samples/복학원서.hwp`에 이름·서명·날짜까지 채우는 명령과 PUA-safe 결과를 문서화했다.
- #3448: 임의 문서 입력의 controlled error, `-o`/output/write 계약과 회귀 test를 보강했다.
- #3449: 최초 수정이 #2439의 일반 표 배치를 깨뜨려, 같은 anchor 앞에 자리차지 float가 있는 경우에만
  보정하도록 좁혔다. 최근접 셀 간격을 수치 assertion으로 고정했다.

#3404의 contributor 문서는 실제 명령 결과 `matchCount=2`와 다른 `123건·83%` 수치와 합성 증적을
포함했다. 별도 문서 보정 commit `7af17df3e8d8d43c38efbb918ca0891e046ff0f9`에서 README를 현재
release 실측값과 render-tree 확인 절차로 정정하고, 잘못된 수치가 박힌 PNG 두 장을 제거했다. 실제
`-p 3` 산출 페이지는 독립 review asset으로 보존했다. 제거한 원 이미지는 Git 이력에서 복구할 수 있다.

## 로컬 검증

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`과 검토 전용
`CARGO_TARGET_DIR=target/review-kevin9327-20260726-v2`를 사용했다. 공유 `target/debug`,
`target/release`, `target/release-test`, `target/wasm32-unknown-unknown`은 건드리지 않았다.

- focused 회귀: #3419 수식 평문·Markdown, #3420 소책자 6쪽, #3426 chart shapeComment,
  #3428 중첩 수식 search/replace, #3430 TotalPage, #3448 test-caption, #3449 float 간격,
  #2439 무회귀, Native Skia Equation 생성자, HWP→HWPX TotalPage — 모두 통과.
- `cargo build --release`: 통과.
- `cargo test --release --lib`: **2943 passed / 0 failed / 7 ignored**.
- `cargo test --profile release-test --tests`: 모든 test target exit 0, IR field sweep **2/2** 포함.
- Native Skia 공식 3종: **57/0**, **2/0**, **4/0**.
- `cargo fmt --all -- --check`, `git diff --check`,
  `cargo clippy --all-targets -- -D warnings`: 통과.
- doc test: **4 passed / 0 failed / 2 ignored**.
- `wasm-pack build --target web`: 검토 전용
  `target/review-kevin9327-20260726-v2/wasm-pkg` 출력으로 통과. 공유 `pkg`는 건드리지 않았다.
- frontend 변경이 없으므로 frontend gate는 로컬 생략했다.
- 새 HWP/HWPX fixture가 없어 IR field sweep baseline 신규 등록은 대상이 아니다. 기존 전체 sweep은
  위 release-test에서 통과했다.
- 최신 devel 동기화가 바꾼 Rust 범위는 `Cargo.toml`·`Cargo.lock`의 package version
  `0.8.1 → 0.8.2`뿐이고 `src/**`·`tests/**` 차이는 없다. 동기화 뒤 `cargo metadata --locked`와
  검토 전용 target의 `cargo check --profile release-test --locked --all-targets`를 다시 통과했다.
  이미 완료한 전수 test를 중복 실행하지 않고 최신 통합 PR full CI를 최종 게이트로 둔다.

## 기능·시각 검증

- #3404: 국립국어원 업무계획 검색 결과 2건이 모두 0-based page 3이고, `_004.svg`와 render-tree에
  인용 문구가 존재함을 확인했다.
- #3419: `exam_math.hwp` p13에서 `lim`·`sin`과 ①–⑤ 값이 추출됐고 수식 중첩 후보는 0건이었다.
  자동 sweep의 1/1 flag는 기준 PDF와 rhwp의 기존 scale·글꼴 차이로 재분류했다.
- #3420: `exam_math.hwp` p10·12·14·16·18·20 sweep은 **0/6 flagged**였고 p12·p20의 모서리
  쪽번호 `4`를 독립 asset에서 확인했다.
- #3430: `exam_eng.hwp` p1–8 sweep은 **0/8 flagged**, p3 꼬리말 `3 / 8`을 확인했다.
- #3449: `synam-001.hwp` p30의 host text와 표 첫 셀 간격은 16.52px로 분리됐다. p31 flag는
  기준·후보 양쪽의 기존 footer/frame bleed로 이 변경의 blocker가 아니다.
- #3426 실제 `export-hwpx --verify --verify-pages`는 1쪽·IR diff 0으로 통과했고, #3447 실제
  `set-cell`/`replace-text`, #3448 `test-caption -o` 산출도 CLI에서 재현했다.

개별 수용 근거와 인라인 증적은 `pr_3404_review.md`부터 각 원 PR 번호의 review 문서에 보존한다.

## 최종 권고와 후속 단계

10건 모두 **메인터너 보정 포함 기술적 수용 가능**하다. #3413은 #3419와 #3428만으로 전체 요구를
완결하지 않으므로 통합 PR 본문에서 close하지 않고 open 유지한다.

owner가 [#3445](https://github.com/edwardkim/rhwp/issues/3445#issuecomment-5083833363)에서 당시 열린
PR을 v0.8.2 hotfix 기준선에서 제외했지만, 이후
[v0.8.2 릴리즈가 완료](../../report/task_m100_3445_report.md)됐다. #3445는 운영 기록상 open을
유지하는 이슈이며, 최초 범위 지시를 현재 `devel` merge 보류로 확장하지 않는다. 다음 순서로 처리한다.

1. review·asset·오늘할일 commit을 통합 branch에 추가하고 `devel` 대상 통합 PR을 만든다.
2. 최신 통합 head의 GitHub Actions와 mergeable 상태를 확인한다.
3. 최신 통합 head의 GitHub Actions와 mergeable 상태가 성공하면 작업지시자의 자동승인 범위로 merge한다.
4. 통합 PR이 실제 merge된 뒤에만 원 PR 10건에 통합 commit·검증 증적·감사 comment를 남기고
   close/merge 상태 및 관련 issue 상태를 확인한다. 원 PR 후속 GitHub 변경은 통합 PR 생성 단계와
   구분하며, 작업지시자의 자동승인 범위로 수행한다.
5. 후속 처리 뒤 `upstream/devel`을 동기화하고 review branch·worktree·검토 전용 target을 정확히 정리한다.

보정 rollback이 필요하면 contributor commit은 유지한 채 `7af17df3e`와 `a1fe4ce76`을 역순으로 분리
검토한다. 통합 전체를 폐기할 때도 원 contributor branch에는 영향이 없다.
