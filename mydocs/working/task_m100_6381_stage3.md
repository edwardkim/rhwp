# Task M100 #6381 Stage 3 완료보고 — 최신 devel 전체 검증

- **이슈**: [#6381](https://github.com/edwardkim/rhwp/issues/6381)
- **최신 기준**: `upstream/devel@f5440811042f9c5ab7580d3a64204cf1d1e39dd8`
- **검증 기준 HEAD**: `143e3032d9c736caade605db3cbfc2cc2748ebb5`
- **상태**: 최신 기준 전체 로컬 검증 완료, remote push·Draft PR 생성 승인 완료

## 1. 최신 devel 반영

장기 게이트 직전 `upstream/devel`이 착수 기준 `2bcf9b261`에서 `97c4d7155`로 5개 commit 이동한 것을
확인했다. upstream 변경에는 picture edit module 보정이 포함됐지만 #6381이 수정한 caption validation
command·회귀 test·CLI 문서와 직접 경로 충돌은 없었다.

dry merge tree 생성을 먼저 확인한 뒤 Hyper-Waterfall 단계 commit을 보존하는 merge 방식으로 최신 기준을
반영했다.

- dry merge tree: `78b1467e3178ef0045accf03c6c1039ed3485a60`
- current-base merge: `49d0a61ea`
- 충돌: 없음
- merge 뒤 추가 제품 보정: 없음

## 2. focused 재검증

최신 기준에서 integration suite를 다시 준비하면서 #6381 test의 generated target 배정이
`regression_suite_004`에서 `regression_suite_002`로 이동했다. 이전 target을 지정한 첫 진단 실행은 대상
test가 없어 exit 4였으며 제품 test 실패가 아니다. 현재 manifest에서 배정된 target을 확인한 뒤 같은
세 시나리오를 다시 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --test regression_suite_002 \
  -E 'test(/issue_cli_test_caption_no_panic/)' --no-fail-fast
```

- nextest run: `b54f8ddf-6ab7-45ba-96ee-e3ac7b547fa8`
- 결과: 3/3 pass
- 계약: all-fail·partial-fail exit 1, all-pass exit 0

## 3. 장기 게이트

작업지시자가 Stage 2 focused 결과를 확인하고 장기 게이트 실행을 승인한 뒤 다음을 실행했다.

```bash
cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --no-fail-fast
```

| 게이트 | 결과 |
| --- | --- |
| clippy | `-D warnings` 통과, 1분 27초 |
| 전체 integration nextest | 8,636/8,636 pass, 43 skipped, 7 slow, 288.183초 |
| nextest run | `a70f4a26-f5f2-44ac-82e0-e8ffb5d7d84f` |

slow 7건은 통과한 test의 실행 시간 표기이며 실패가 아니다.

## 4. 최종 정적 게이트

```bash
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
git diff --check
```

- integration manifest: 1,032 sources / 4,533 static test attrs / 48/48 integration targets /
  nextest 최소 6,559 cases / weight 786,011..791,608
- unit tier: 4,221 tests / 299 modules / ready 0 / support 87 / white-box 4,130 /
  cfg support items 28
- format·diff check: 통과
- tracked working tree: clean

`tests/generated/`, `tests/suites/manifest.json`, `target/`, `output/`은 로컬 검증 산출물로 ignored 상태이며
stage하거나 제출 diff에 포함하지 않았다.

## 5. PR 게시 직전 재기준화

remote push·Draft PR 생성 승인 뒤 `upstream/devel`이 `97c4d7155`에서 `f54408110`으로 3개 commit 더
이동한 것을 확인했다. 추가 변경은 장기 baseline test 분할과 관련 문서였으며 #6381 source와 직접 경로
충돌은 없었다. dry merge tree `c9a4ab33a9a7ea8ca52f21fc284f590320c9ff66`을 확인하고 merge commit
`143e3032d`로 반영했다.

generated target이 `regression_suite_018`로 이동해 현재 manifest 배정으로 다시 검증했다.

| 게이트 | 최신 기준 결과 |
| --- | --- |
| focused nextest | 3/3 pass, run `9178a2dd-86d3-4842-a44b-cfe6e6132b96` |
| clippy | `-D warnings` 통과 |
| 전체 integration nextest | 8,660/8,660 pass, 43 skipped, 4 slow, 177.007초 |
| 전체 nextest run | `f5122360-2c28-47fa-a8a6-0824129d7d47` |
| manifest | 1,032 sources / 4,533 attrs / 48/48 targets / weight 786,188..791,783 |

## 6. Stage 3 판정

#6381의 fail-closed 구현은 최신 devel의 picture edit 변경과 함께 focused·clippy·전체 integration에서 모두
통과했다. renderer·layout·document model·Render Diff workflow 변경이 없어 별도 시각 검증 대상은 아니다.
로컬 구현·검증은 최신 PR 기준에서도 완료됐고, 작업지시자가 승인한 remote branch push와 Draft PR
생성 단계로 진행한다.
