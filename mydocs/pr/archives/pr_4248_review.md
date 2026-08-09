---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4248 검토 - 셀 캐럿 rect fast path

## 대상과 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4248](https://github.com/edwardkim/rhwp/pull/4248) / @humdrum00001010 |
| contributor 원 head | `a35d980d0a9c8da3dc45aa56b8471c4c319bee11` |
| base / 규모 | `devel`, 24개 파일, +2,505/-188 |
| 관련 이슈 | [#4167](https://github.com/edwardkim/rhwp/issues/4167) |
| 작성 시점 원격 상태 | `mergeable=CONFLICTING`, `mergeStateStatus=DIRTY`; #4249와 같은 stack이므로 통합 PR에서 해소한다. |

전체 페이지 트리 대신 production partial-table compose를 대상 셀에만 적용해 셀 캐럿 rect를 계산한다.
불확실 형상은 기존 전체 경로로 fallback해 정합을 우선한다.

## 메인터너 보정

통합 검증에서 KTX fixture의 뒤 문단 중첩 표 border clip이 바깥 셀 `cellBounds`를 넓히는 형상을
fast path가 생략했다. `da2d9ae75`는 해당 셀에 뒤 문단의 직접 표 control이 있으면 종료 문단만 셀 끝으로
확장한다. 기존 분할 셀 `window_paras`는 유지하므로 거대 셀을 전량 compose하지 않는다.

## 검증과 판단

- 원 head의 `Build & Test`가 통과했다.
- KTX parity는 `3/371`, giant-cell warm latency는 fast `6.185ms`, legacy `92.136ms/call`로 통과했다.
- 전체 fast-path parity 3건과 `cargo test --profile release-test --tests`가 통과했다.

**메인터너 보정 포함 통합 수용 권고.** 원 PR의 현재 단독 충돌은 #4249가 후속 stack이기 때문이며,
#4265에서는 두 변경과 보정을 함께 적용한다.
