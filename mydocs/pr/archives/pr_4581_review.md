---
kind: report
status: active
last_verified: 2026-08-12
---

# PR #4581 검토 — cold-cache release test archive 완주 정책

## 라우팅

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md,
  rework_and_exceptions.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/maintainer_general.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md, pr_review/rework_and_exceptions.md,
  pr_review/post_merge.md
current head: 7585b2e85f018a902db79e917916133583fbe82c (접수 시점 참고)
```

## Metadata

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR | [#4581](https://github.com/edwardkim/rhwp/pull/4581) |
| 작성자 | `postmelee` |
| reviewer | `edwardkim` |
| base / head | `devel` / `issue-4029-cold-release-ci` |
| 관련 이슈 | `Closes #4029` |
| 규모 | 7개 파일, +657/-34, 4 commits |
| 상태 | Open, non-draft, `MERGEABLE/CLEAN` |

PR head는 최신 `upstream/devel@298c2c1b2`보다 32 commits 뒤에 있다. GitHub의 `CLEAN`은 텍스트
충돌이 없다는 참고값으로만 사용하고, 최신 devel merge tree를 별도로 검증했다.

## 변경 범위

이 PR은 cache miss 상태의 `main`·tag release-grade CI가 `release` profile 컴파일을 30분 안에
끝내지 못해 반복 취소되는 문제를 다음 정책으로 닫는다.

1. PR·`devel`·일반 수동 실행은 `release-test`와 archive builder 30분 상한을 사용한다.
2. `main`·`v*` tag·`release_grade=true` 수동 실행은 `release`와 archive builder 60분 상한을 사용한다.
3. 알 수 없는 event/ref와 잘못된 수동 입력은 `release/60`으로 fail-closed 한다.
4. 세 archive builder와 Native Skia가 preflight의 같은 test profile을 사용한다.
5. reusable archive workflow는 `release-test:30`과 `release:60` 조합만 허용한다.
6. event, ref, profile, timeout, cache exact hit, cache 저장 자격을 Job Summary에 남긴다.

실제 release binary와 WASM artifact의 `--release` 명령, archive worker topology, required check
`Build & Test`, cache 저장 범위는 바꾸지 않는다. renderer, layout, WASM API와 rhwp-studio 출력은
변경하지 않으므로 시각·fixture 증적 경로는 적용하지 않는다.

## Review finding

blocking finding은 없다.

- profile과 timeout을 preflight에서 한 번 계산해 세 builder 사이의 정책 드리프트를 막는다.
- reusable workflow가 허용 조합을 build 전에 다시 검사해 caller 오배선을 조기에 실패시킨다.
- timeout 상향은 release-grade archive builder에만 적용되고 PR·devel 경로 비용은 유지된다.
- Native Skia는 같은 test profile을 소비하지만 기존 job timeout은 늘리지 않는다.
- `cache_exact_hit=false`를 부분 restore와 완전 miss로 구분하지 않는다는 안전 경계가 문서에 명시됐다.
- 60분에도 cold release가 완주하지 못하면 추가 상향 대신 target scope 또는 LTO 구조를 재검토하도록
  롤백·재설계 경계가 고정됐다.

## 검증

- PR head `7585b2e85`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/31505640147):
  archive builder 3개, worker 4개, Native Skia, Lint, 최종 `Build & Test` 성공
- 같은 head의 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31505639760):
  JavaScript/TypeScript·Python·Rust 분석 성공
- 최신 `upstream/devel@298c2c1b2`와의 merge simulation: 충돌 없음,
  merge tree `a93df49d5d84c7e6a7e34e416af3a895b0bb389a`
- merge tree `git diff --check`: 통과
- `python3 -m unittest scripts/tests/test_nextest_archive_workflow.py`: 9/9 통과
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow*.py'`: 101/101 통과

로컬 WSL2에는 `actionlint` 실행 파일이 없어 해당 명령은 실행하지 못했다. 같은 PR head의 GitHub Lint와
workflow 계약 테스트 성공으로 보완한다. Rust 제품 코드나 Cargo profile 정의는 바뀌지 않아 로컬
release-test 전체 회귀는 적용하지 않았다.

## 최종 권고

최신 devel merge tree에서 충돌과 workflow 계약 회귀가 없고, 이 세션에서 세 release-grade archive
builder가 다시 30분 상한에 걸린 현상을 수정 정책이 직접 다룬다. blocking finding 없이 merge를
권고한다.

작업지시자의 명시적 merge 승인과 merge 직전 최신 head·required check·mergeability 재확인을 조건으로
`devel`에 admin merge한다. merge 후 devel CI가 성공하면 최신 devel을 main에 직접 반영하고, 새
60분 상한을 사용하는 main release-grade CI의 세 builder·네 worker·Native Skia·최종 aggregate 성공을
확인한 뒤에만 `v0.8.3` 태그와 Release 단계로 진행한다.

## Merge 결과

- 2026-08-12 작업지시자가 엄격한 CI 경로를 승인했다.
- `edwardkim` 승인 리뷰 뒤 PR을 `devel`에 admin merge했다.
- merge commit: `96e52a01be63a7f7b75441a01fdd4281d3a99ade`
- devel [CI](https://github.com/edwardkim/rhwp/actions/runs/31526760445)와
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31526760022)이 성공했다.
- devel CI에서 archive builder 3개, worker 4개, Native Skia, 최종 `Build & Test`가 모두 성공했다.
- 다음 게이트는 이 운영 기록의 review-only fast-pass와 최신 devel의 main 반영, main release-grade
  CI 성공이다.
