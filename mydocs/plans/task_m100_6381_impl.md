# 구현계획 — Task M100 #6381 `test-caption` false-pass 제거

- **상위 수행계획**: [task_m100_6381.md](task_m100_6381.md)
- **이슈**: [#6381](https://github.com/edwardkim/rhwp/issues/6381)
- **작성일**: 2026-08-29 KST
- **작업 브랜치**: `task_m100_6381-test-caption-false-pass`
- **착수 기준**: `upstream/devel@2bcf9b261c3b761d114bc2b3a35ed85ccd1e461e`
- **현재 기준**: `upstream/devel@f5440811042f9c5ab7580d3a64204cf1d1e39dd8`
- **구현 상태**: fail-closed 구현과 최신 devel 기준 전체 로컬 검증 완료

## 1. 구현 불변식

- 고정 대상 `(para, control)` 네 개와 적용 순서를 유지한다.
- Bottom/Top/Left/Right, Top/Top/Center/Center, width 8504, spacing 850을 유지한다.
- 모든 mutation과 verification이 성공해야만 렌더 단계로 이동한다.
- 실패 진단은 stderr, 성공 진행 정보와 `완료`는 stdout을 사용한다.
- 실패 시 출력 폴더를 만들거나 SVG를 쓰지 않는다.
- usage 오류는 exit 2, 파일·파싱·검증·렌더·저장 오류는 exit 1을 유지한다.

## 2. 파일별 변경안

| 파일 | 변경 | 고정할 계약 |
| --- | --- | --- |
| [`src/cli/commands/caption_validation.rs`](../../src/cli/commands/caption_validation.rs) | mutation·verification 실패 집계, 정확한 caption 비교, fail-closed 반환 | 성공 대상·속성·SVG 이름 |
| [`tests/issue_cli_test_caption_no_panic.rs`](../../tests/issue_cli_test_caption_no_panic.rs) | all-fail·partial-fail·all-pass subprocess 회귀 | exit/stdout/stderr/산출물 |
| [`mydocs/manual/cli_commands.md`](../manual/cli_commands.md) | `test-caption` 성공·검증 실패 의미 명시 | 공통 exit code 표 |

## 3. 테스트 fixture 전략

회귀 테스트는 저장소 binary를 subprocess로 실행하므로 실제 parse→mutation→render 경계를 지난다.

1. **all-fail**: 기존 임의 실문서를 사용한다. 고정 대상이 없으며 exit 1이어야 한다.
2. **partial-fail**: `HwpDocument::create_empty()`에 문단 둘을 만들고 일부 대상 위치에만 그림을 삽입해
   HWP로 export한다. 앞 대상 mutation은 성공하지만 나머지는 실패해야 한다.
3. **all-pass**: 같은 방식으로 두 문단의 네 대상 위치 모두 그림이 되도록 만든 뒤 export한다. 명령이
   exit 0을 반환하고 SVG를 하나 이상 남겨야 한다.

fixture는 `assets/logo/logo-16.png`와 공개 native 편집 API를 사용하고 테스트 종료 때 제거한다. 별도 binary
fixture를 repository에 추가하지 않는다.

## 4. 구현 순서

1. integration test helper로 임시 경로·합성 HWP·command 실행·정리를 구성한다.
2. 기존 no-panic test를 all-fail 계약으로 바꾸고 partial/all-pass 테스트를 추가한다.
3. focused test가 현행 구현에서 false-pass를 재현하는지 확인한다.
4. mutation 오류를 stderr에 남기고 실패 flag를 설정한다.
5. 네 대상 각각의 존재, picture 종류, caption 존재와 정확한 속성을 검증한다.
6. 실패 flag가 있으면 출력 폴더 생성 전 exit 1을 반환한다.
7. page가 없거나 SVG 렌더·저장이 실패한 경우도 exit 1을 반환한다.
8. focused test와 문서 계약을 맞춘다.

## 5. 예상 진단 계약

| 상황 | exit | stderr | stdout·산출물 |
| --- | ---: | --- | --- |
| 입력 파일/파싱 실패 | 1 | 오류 원인 | `완료` 없음, SVG 없음 |
| 대상 좌표 없음 | 1 | 대상 좌표와 범위 | `완료` 없음, SVG 없음 |
| 대상이 Picture가 아님 | 1 | 대상 종류 오류 | `완료` 없음, SVG 없음 |
| 캡션 속성 불일치 | 1 | 기대값과 실제값 | `완료` 없음, SVG 없음 |
| 네 대상 모두 성공 | 0 | 비어 있음 | 진행 정보, `완료`, SVG 1개 이상 |

## 6. 검증 명령

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --test issue_cli_test_caption_no_panic --no-fail-fast
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast
python3 scripts/check_markdown_links.py
git diff --check
```

integration suite 준비가 필요한 review worktree 검증에서는 먼저
`node scripts/rust-test-suite-manifest.mjs --prepare`를 실행한다. 생성된 suite와 manifest는 stage하지 않는다.

## 7. 커밋 경계

1. 계획: 수행·구현 계획과 기준선
2. 구현: CLI fail-closed 동작과 세 회귀 테스트
3. 문서·검증: CLI 문서, 단계 보고와 최종 검증 결과

각 단계는 exact path만 stage해 local commit으로 고정한다. push·PR은 별도 승인 게이트다.

## 8. 구현 결과

- 계획 commit: `c0bc5c486`
- fail-closed 구현과 세 CLI 회귀 test: `21c3bd43c`
- 내부 CLI 계약 문서: `025ce6806`
- 기존 제품에서 all-fail·partial-fail이 exit 0으로 재현됐고, 구현 뒤 all-fail·partial-fail·all-pass 세
  시나리오가 모두 통과했다.
- mutation 오류를 stderr에 누적하고 mutation 성공 대상만 정확한 캡션 속성으로 verification한다.
- 실패가 하나라도 있으면 출력 폴더 생성과 page render 전에 exit 1로 종료한다.
- renderer·layout·model·Render Diff workflow는 변경하지 않았다.
- 최신 `upstream/devel@97c4d7155`를 merge commit `49d0a61ea`로 반영했으며 직접 충돌이나 추가 제품
  보정은 없었다.
- generated suite 재준비 뒤 focused 3/3, clippy `-D warnings`, 전체 integration 8,636/8,636이
  통과했다. 상세 실행 근거는 [Stage 3 보고](../working/task_m100_6381_stage3.md)에 기록했다.
- source-side test 정책, format, manifest, Markdown link와 diff 검사를 모두 통과했고 generated
  suite·manifest와 로컬 출력물은 제출 diff에 포함하지 않았다.
- PR 게시 직전 이동한 `upstream/devel@f54408110`도 merge commit `143e3032d`로 반영했다. 새 generated
  배정 `regression_suite_018`에서 focused 3/3, clippy와 전체 integration 8,660/8,660을 재통과했다.
- 작업지시자 승인 뒤 code candidate `988b9c85f`를 push하고 Draft PR #6391을 생성했다. 번호 기반
  self-review는 [PR #6391 검토 문서](../pr/archives/pr_6391_review.md)로 trailing한다.
