# PR #4096 검토 - 글자겹침 `charSz` 축소를 실제 테두리 경로로 한정

- PR: https://github.com/edwardkim/rhwp/pull/4096
- 작성자: `planet6897`
- 관련 이슈: [#4085](https://github.com/edwardkim/rhwp/issues/4085)
- 검토일: 2026-08-06
- contributor 원 head: `8128683632154b598aad1e44438fea53b15af6d7`
- 검토 경로: `review/planet6897-4096-20260806`

## 적용 절차

```text
base route: maintainer_general
modifiers: intake_and_review, local_validation, visual_fixture_evidence,
           rework_and_exceptions, collaborator_external_pr, review_only_fast_pass
loaded documents: AGENTS.md, pr_review_workflow.md, intake_and_review.md,
                  local_validation.md, visual_fixture_evidence.md,
                  collaborator_external_pr.md, review_only_fast_pass.md
current head: 8128683632154b598aad1e44438fea53b15af6d7
```

작성자는 외부 contributor이고 `maintainerCanModify=true`였다. reviewer `jangster77`를 local fetch 전에
assign했다. 현재 `upstream/devel` `8b85fd64f`는 PR head의 조상이 아니며 공통 조상은
`d76d4e98b`였다. contributor history를 rebase하거나 rewrite하지 않고, 최신 `devel`에서
`pr4096-merge-test` merge simulation을 수행했다. simulation은 conflict 없이 성립했다.

## 변경 검토

PR은 `char_overlap_size_ratio(effective_border, inner_char_size)`를 `composer.rs`에 두고 기존에
SVG, WebCanvas, Native Skia에 흩어져 있던 `charSz` 계산을 같은 함수로 교체했다.

- 실제 테두리가 없으면 양수·음수 `charSz` 모두 `1.0`을 사용한다.
- 일반 원/사각형과 반전 원/사각형은 기존 양수 percent 및 음수 10% step 축소를 유지한다.
- `border_type=0`이라도 PUA 다자리 숫자 결합 경로에서 원형으로 승격되면 `effective_border=1`을
  전달하므로 기존 축소 동작을 보존한다.
- SVG, WebCanvas, Native Skia 모두 helper를 사용해 renderer 간 음수 `charSz` 처리 불일치를 제거한다.

원본 기능 변경은 `src/renderer` 5개와 helper unit/SVG regression test이며, 계획·보고 문서 4개가
함께 추가됐다. 범위 밖 production 변경은 확인하지 못했다.

## 로컬 검증

모든 Cargo 실행은 `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=target/review-pr4096`로 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| 최신 `devel` merge simulation | conflict 없이 완료 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --profile release-test --lib char_overlap` | 10 passed, 0 failed |
| `cargo test --profile release-test --features native-skia skia --lib` | 58 passed, 0 failed |
| `cargo build --profile release-test --features native-skia` | 통과 |

`release-test --tests` 전체, Native Skia 추가 integration 2종, WASM package는 이번 검토에서 다시
실행하지 않았다. contributor code head의 CI `31082773449`와 CodeQL `31082773128`은 문서 작성 전
성공 상태였고, 이번 local 검증은 renderer 변경부와 Native Skia replay를 독립적으로 좁혀 확인했다.
문서-only commit push 뒤에는 같은 PR/source/candidate의 fast-pass aggregate와 최신 mergeability를
다시 확인해야 한다.

## 실제 fixture 확인

두 원본은 저장소의 추적 파일이며, merge simulation으로 만든 Native Skia CLI를 사용했다.

| 원본 | 페이지 | 확인 |
| --- | --- | --- |
| `samples/156636617_240617 2024년 5월 월간 수출입 현황(확정치).hwp` (`83fca015...f6845e`) | 1쪽 | SVG와 PNG를 생성했다. SVG에는 `font-size="22.67"`가 1건, `<ellipse>`가 0건으로 나와 테두리 없는 경로가 축소·원 테두리를 강제하지 않음을 확인했다. PNG에서 본문과 표·차트가 정상 렌더링됐다. |
| `samples/hwpx/k-water-rfp.hwpx` (`036ae73c...2fc14`) | 13쪽 | SVG와 PNG를 생성했다. SVG의 `font-size="18.13"`가 3건이고 반전 사각형 마커가 PNG에서 유지돼 `border_type=4`, `charSz=-2` 회귀 경로를 확인했다. |

임시 산출물은 `/tmp/rhwp-pr4096-review.V8uIeS/current/`에 있으며, 기준 PDF나 새 fixture를
변경에 추가하지 않았으므로 장기 asset으로 commit하지 않았다. PR 보고서의 한컴 COM PDF content-stream
수치는 contributor 증적으로만 취급했다. 이번 검토에서는 해당 두 원본을 rhwp SVG/PNG로 재현해
변경의 양쪽 분기를 확인했으며, 한컴 PDF를 새로 생성한 전수 visual sweep은 수행하지 않았다.

## 판정

**로컬 수용 권고.** 공통 helper로 renderer 세 경로를 정렬했고, 테두리 없음과 반전 사각형의 상반된
회귀 조건, Native Skia replay, 최신 `devel` merge simulation을 모두 통과했다. 발견한 merge blocker는 없다.

`mydocs/orders/20260806.md`가 현재 `devel`에서도 갱신돼 문서-only commit만으로는 merge conflict가
발생했다. 두 기록을 모두 보존하는 current-base merge `459b30ddd`를 만들었으나 자동 `merge-tree`는
해당 문서의 충돌 때문에 tree를 만들지 못했다. 따라서 review-only fast-pass 재사용 조건에는 해당하지
않고, 이 문서 commit을 push한 최신 head의 full CI가 필요하다.

원격 상태는 변동 가능하므로 merge 전에는 최신 head의 full CI, `mergeable=CLEAN`, 그리고 작업지시자
승인을 다시 확인한다.
