---
kind: pr_review
status: local-validation-passed-intentional-red
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4315 검토 — #3211 저장 HWP LineSeg 대비 비캐시 재계산 정합성 회귀 테스트

## 결론

**로컬 전체 게이트 통과, 단 이 PR 이 추가한 4건은 의도적 red 다.** 한컴이 저장한
`PARA_LINE_SEG` 를 정답지로 삼아 `reflow_line_segs()` 재계산 결과를 직접 대조하는
회귀 테스트를 추가했다. 문서 31건·약 13,000 문단을 대조해 6개 임계 경계를 실측으로
분류했고, 그중 4개 코호트가 위반한다. 수정은 이 PR 범위 밖이므로 **Draft** 로 열었다.

`#3211` 은 실패 2건(`exam_3_09_2022`, `exam_3_09_2024_sep2020`)을 예상했지만, 렌더
결과가 아니라 LineSeg 필드를 직접 보면 발산은 그 두 문서에 국한되지 않고 거의 모든
실문서·모든 중첩 깊이에서 나타난다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md
devel base: 6be36976b (upstream/devel HEAD, rebase 대상)
validated code head: 909343cf5
```

시각·fixture 증적 보조 경로는 적용하지 않는다 — 렌더 출력·golden·fixture 를 만들거나
바꾸지 않는다. `src` 변경은 `reflow_line_segs` 의 가시성 확대 한 줄과 그 re-export 뿐이라
렌더 동작이 달라지지 않는다(그래도 4.3 표의 renderer 행 게이트는 전부 돌렸다).

별도 `review_impl` 문서는 만들지 않았다. 단일 목적(회귀 테스트 추가)이고 남은 결정
지점이 "언제 수정 PR 을 붙일 것인가" 하나뿐이다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4315](https://github.com/edwardkim/rhwp/pull/4315) |
| 관련 이슈 | [#3211](https://github.com/edwardkim/rhwp/issues/3211) (`Refs`, `closes` 아님 — 수정 미포함) |
| 작성자 | `humdrum00001010` (fork 기반) |
| 대상 / head | `devel` / `humdrum00001010:test/issue-3211-lineseg-uncached-compliance` |
| 상태 | **Draft**, mergeable(`MERGEABLE`), `mergeStateStatus: BLOCKED` — 작성 시점 참고값 |
| 규모 | +664 / -2, 1 commit, 3 files |

## 오래된 base 처리

시작 worktree 는 #3211 이 지정한 기준 커밋 `1d6d0073b`(2026-07-24)에 있었고,
`upstream/devel` 대비 **1798 커밋** 뒤처져 있었다. 그대로 PR 하면 검토가 불가능하고
측정값도 낡은 레이아웃 코드 기준이 된다.

현재 `upstream/devel`(`6be36976b`) 로 rebase 했고 충돌은 없었다. rebase 뒤 **재측정**해
결함이 그대로 재현되는 것을 확인했다 — `v0.7.19` → `v0.8.2` 에서 위반 문서 수와 축이
동일하고, 대조 줄 수만 한 코호트에서 7315 → 7317 로 미세하게 달라졌다.

## 비캐시 경로 계약 — 검토 중 정정한 사항

초안은 문단을 `clone()` 해서 `reflow_line_segs()` 에 넘겼는데, 이 함수는
`para.line_segs.first()` 를 `orig` 템플릿으로 읽어 `tag`·`vertical_pos` 를 물려받고
빈 문단 분기에서는 `line_spacing`·`segment_width` 까지 저장값에서 복사한다. 즉 "재계산"이
저장값에 기대고 있어 #3211 이 말하는 **비캐시** 경로가 아니었다.

넘기기 전에 `line_segs` 를 비우도록 고쳤다. 다만 **현재 비교하는 4축의 수치는 이 차단
전후가 동일**하다 — `make_line_seg()` 에서 `line_height` 는 폰트 크기,
`segment_width` 는 가용 폭, `text_start` 는 줄 나눔 결과에서 나오고 `orig` 는 `tag` 와
`vertical_pos` 에만 쓰이는데 둘 다 비교 대상이 아니기 때문이다. 그래도 계약을 명시적으로
고정해야 이후 텍스트 경로가 `orig` 에 기대게 바뀔 때 이 테스트가 조용히 캐시 의존으로
되돌아가지 않는다.

## 대조 범위와 결과

문서 31건 / 5개 코호트, 약 13,000 문단 · 18,500 줄.

| 코호트 | 문서 | 결과 |
| --- | --- | --- |
| LINESEG 축별 fixture (`lseg-01..06`) | 6 | **전건 통과** |
| 미주/수식 시험지 | 5 | 5건 위반 |
| 중첩 표 | 5 | 5건 위반 |
| 표 다수 장문 실문서 | 5 | 5건 위반 |
| 일반 실문서 | 10 | 8건 위반, 2건 스킵 |

`lseg-*` fixture 통과가 중요한 대조군이다 — 줄간격/들여쓰기/탭/혼합 크기를 하나씩 분리한
문서는 정합하므로, 계산식 자체가 깨진 것이 아니라 실문서 복합 조건에서만 발산한다.
빈 검사가 통과로 위장되지 않도록 전건 스킵이면 실패시키는 가드도 넣었고, fixture 코호트가
실제로 23문단 75줄을 대조하는 것을 확인했다.

## 실측 임계 경계

| 축 | 증거 | 성격 |
| --- | --- | --- |
| A. 인라인 개체 줄높이 오배치 | `3-09'23 p131` `L0 +3375 / L1 -3375` (부호 반전) | 미주 드리프트 직접 원인 |
| B. 인라인 개체 문단 줄 수 발산 | `3-09'22 p76` 저장 5줄 vs 재계산 3줄 | 구조 |
| C. 줄바꿈 위치 발산 | `stored_start=43 → 83` | 구조 |
| D. wrap zone 무시 | 저장 `sw=10718` vs 재계산 `26788` (+16070) | 개체 회피 폭 미반영 |
| E. 중첩 깊이 발산 | `issue1949` d1 `text_start` 36.8%, `issue2007` d3 `+1001` 계통 | 셀 폭 계약 |
| F. 좁은 셀 줄 수 붕괴 | 35~38px 셀 저장 2줄 vs 재계산 1줄 | 경계 |

HWPUNIT ±1 반올림 잡음은 `Budget::dim_rounding_tolerance_hwp` 로 흡수해 결함으로 세지
않는다(`1300 vs 1299` 류가 광범위하게 존재).

## 로컬 검증 (code head `909343cf5`, rebase 후)

`local_validation.md` 4.3 의 renderer 행을 따랐다.

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --profile release-test --tests --no-fail-fast` | **5487 pass / 4 fail** — 실패 4건은 전부 이 PR 의 의도적 red. 부수 회귀 없음 |
| 인접 핀 `issue_1082_endnote_multicolumn_drift` | **5/5 PASS** (무회귀) |
| Native Skia 3종 | PASS (`skia --lib` 56 / `issue_2225` 2 / `render_p37` 4) |
| `wasm-pack build --target web` | PASS |
| `git diff --check` | PASS |

rebase 전 baseline(`1d6d0073b`)에서도 같은 게이트를 돌려 3881 pass / 4 fail 이었고,
실패 4건의 정체는 동일했다.

## 발견한 문제와 처리

- (정정) 초안의 캐시 미차단 — 위 §비캐시 경로 계약에서 수정.
- (설계) `reflow_line_segs` 를 `pub(crate)` → `pub` 으로 넓혔다. `tests/` 통합 테스트에서
  부르기 위함이며, 비교 상대인 `lineseg_compare` 가 이미 `pub mod` 인 것과 짝을 맞춘다.
  동작 변화는 없다.
- (범위 밖) 수정은 포함하지 않는다. #3211 완료 조건의 "수정 후 통과" 는 후속 PR 몫이다.
- (관찰) `#3211` 이 예상한 실패 2건보다 발산 범위가 훨씬 넓다. 이슈 본문의 "실패 2건"
  프레이밍은 렌더 초과 px 기준이며, LineSeg 직접 대조에서는 거의 모든 실문서가 위반한다.

## GitHub Actions 와 남은 게이트

- **이 PR 은 red 로 유지한다** — CI 의 test job 은 실패한다. 의도된 상태이며 Draft 인 이유다.
- merge 조건이 아니라 **후속 수정 PR 의 착수 기준**으로 쓴다. 수정이 붙으면 이 4건이
  green 이 되어 #3211 의 red→green 요구가 닫힌다.
- draft·mergeable·head SHA·CI 상태는 작성 시점 참고값이며 후속 판단 전에 다시 확인한다.

## 최종 권고

**merge 하지 않는다.** Draft 로 두고 수정 PR 의 근거·게이트로 사용한다. 수정 PR 이
이 테스트를 green 으로 만들 때 두 PR 을 함께 판단한다. 지금 필요한 결정은 수정 착수
여부와 담당자 지정이다.
