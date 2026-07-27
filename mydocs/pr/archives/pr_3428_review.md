# PR #3428 검토 기록 — 수식 script 검색·grep·치환 계약

## 라우팅

```text
base route: collaborator_external_pr.md (작업지시자가 승인한 통합 PR 예외)
modifiers: intake_and_review.md, local_validation.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  pr_review/collaborator_external_pr.md,
                  pr_review/intake_and_review.md,
                  pr_review/local_validation.md,
                  pr_review/multi_pr_update_branch.md
current source head: 작성 시점 참고값 940b915435c8106a66ace73ae9463b0fd54d79f3
```

원 contributor branch에 직접 보정하지 않고, 작업지시자가 승인한 다수 PR 통합 branch에 원 commit을
저자 보존 적용한 뒤 source SHA→통합 SHA 매핑을 기록하고 메인터너 보정을 별도 commit으로 추가했다.

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| 원 PR | [#3428](https://github.com/edwardkim/rhwp/pull/3428) |
| 작성자 / base | `@kevin9327` / `devel` |
| source branch / head | `pr/task-cli-contract-bugfind` / `940b915435c8106a66ace73ae9463b0fd54d79f3` |
| 규모 | 2 files, +89 / -2, 1 commit |
| 원 PR 상태 | `MERGEABLE`, `BEHIND`, draft 아님; source head check 없음; 개별 메인터너 보류 comment/review 없음 |
| 관련 issue | [#3413](https://github.com/edwardkim/rhwp/issues/3413)의 일부 범위; 자동 close 대상 없음 |
| 통합 기준 | 최초 `upstream/devel` `732147a30cf122839afae59c99c91f7854e2f3f2`; 최신 동기화 `7f8fcfef08610df7bf9f5cc2f4b32a9a711f5e2d` |
| 통합 branch | `review/kevin9327-20260726-v2` |
| contributor 적용 | `940b915435c8` → `3c9ae89b1` |
| 메인터너 보정 | `a1fe4ce760899f4ad0b12bc5fbddf808611e9dd5` 중 #3428 관련 hunk |

source head, mergeable, CI는 문서 작성 시점 참고값이다. 최종 merge 조건은 최신 통합 PR head CI와
작업지시자 승인이다.

## 변경 범위와 코드 검토

### Contributor 원 변경

CLI `search`가 사용하는 `DocumentCore::grep`과 편집 검색의 `search_all`은 본문·표 셀·글상자를
검색하면서도 `Control::Equation.script`를 건너뛰었다. 그래서 실제 `lim` 수식이 있는
`samples/exam_math.hwp`에서 `matchCount: 0`을 반환했다.

원 변경은 최상위 Equation script를 두 검색 경로에 넣고, CLI JSON의 `equation.control`로 수식
control 좌표를 제공했다. 이는 #3419의 export-text/Markdown 렌더 트리 보존과 다른, 문서 IR 직접
검색 경로다.

### 메인터너 보정

원 변경만으로는 검색 계약과 편집 계약이 완결되지 않았다.

- 표 셀과 글상자 안에 중첩된 Equation도 검색하고, CLI grep 결과에 각각 `cell`/`textbox`와
  `equation` 좌표를 함께 제공한다.
- 편집 검색 결과는 `equationControl`을 별도 좌표로 제공한다. 일반 본문 커서 이동 필터가 수식
  매치를 본문 문자 오프셋으로 오인하지 않게 제외한다.
- `replace-all`은 검색 개수만 세지 않고 최상위·표 셀·글상자 Equation의 실제 `script`를 역순
  치환한다. 성공적으로 바꾼 건만 count하고 실제 변경 section만 raw stream 무효화·recompose한다.
- dry-run에 해당하는 grep 범위와 실제 replace-all 범위, 치환 전후 검색 결과와 count의 parity를
  회귀 테스트로 고정했다.

추가 JSON 좌표는 해당 매치에서만 나타나는 선택적 필드이므로 기존 본문 검색 결과 모양을 바꾸지 않는다.

## Renderer·fixture·baseline·시각 판정

- 변경은 텍스트 query/JSON/편집 계약이며 renderer, layout, pagination, paint 결과를 바꾸지 않는다.
  따라서 visual sweep은 생략했다.
- 기존 `samples/exam_math.hwp`만 회귀 입력으로 사용하고 새 HWP/HWPX fixture를 추가·교체·이동하지
  않는다. IR field sweep baseline 수동 등록 트리거가 없고 TSV 변경도 없다.
- 전수 `release-test --tests`에 포함된 `ir_field_sweep_baseline` 2/2가 통과했다.

## 실제 CLI·집중 검증

통합 후보의 검토 전용 release-test binary로 다음 사용자 경로를 실행했다.

```text
rhwp search samples/exam_math.hwp lim --json
```

결과는 `matchCount: 6`이며, 첫 매치는 `section: 0`, `paragraph: 18`, `page: 0`,
`equation.control: 1`로 식별됐다. 검색 결과가 단순히 존재하는 것뿐 아니라 수식 control 주소를
제공함을 확인했다.

집중 테스트는 다음 계약을 통과했다.

- `grep_finds_equation_script`: CLI grep 경로가 수식 script와 equation 좌표를 반환.
- `search_all_text_finds_equation_script`: 편집 검색 경로가 `equationControl`을 반환.
- `replace_all_updates_equation_scripts_and_reports_actual_count`: dry-run/실제 치환 범위, 실제 script
  mutation, count와 치환 전후 검색 결과가 일치.

## 공통 로컬 게이트

모든 Cargo 명령은 `CARGO_INCREMENTAL=0`, 검토 전용
`CARGO_TARGET_DIR=target/review-kevin9327-20260726-v2`로 순차 실행했다.

- `cargo build --release`: 통과.
- `cargo test --release --lib`: 2943 passed, 0 failed, 7 ignored.
- `cargo test --profile release-test --tests`: 모든 target exit 0; IR field sweep 2/2 포함.
- Native Skia 공식 3종: 57/0, 2/0, 4/0.
- `cargo fmt --all -- --check`, `git diff --check`,
  `cargo clippy --all-targets -- -D warnings`: 통과.
- `cargo test --doc`: 4 passed, 0 failed, 2 ignored.
- `wasm-pack build --target web`: 검토 전용
  `target/review-kevin9327-20260726-v2/wasm-pkg` 출력으로 통과.

## 리스크·issue·최종 권고

수식 script의 char offset은 본문 편집 커서와 다른 좌표계다. 이를 별도 `equationControl` 및
`equation` 주소로 분리하고 본문 커서 경로에서 제외했으며, 중첩 container 좌표와 함께 테스트했다.
검토 범위에서 추가 blocker는 찾지 못했다.

이 PR은 #3413의 수식 검색·치환 일부만 해결한다. export-text, 다른 구조화 출력, 남은 수식 텍스트
계약을 모두 해결한 것으로 보지 않으며 **#3413은 open 유지**한다.

**메인터너 보정 후 기술적으로 수용 가능**하다. #3445가 고정한 v0.8.2 핫픽스 기준선은
[릴리즈 완료](../../report/task_m100_3445_report.md)로 종료됐으므로 현재 `devel` merge 보류 사유가
아니다. 최신 통합 PR head CI와 mergeable 상태가 성공하면 merge한다.
