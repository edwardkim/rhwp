---
kind: pr_review
status: maintainer-review
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4314 검토 - kevin9327 보안·바인딩·MCP 개선 20건 통합

## 결론

**수용 권고.** 최신 `upstream/devel` `f62f7503f` 위에 `kevin9327`의 열린 `devel` 대상
PR 20건을 번호순으로 누적 적용했다. 충돌은 #4307의 Python test와 roadmap 문서에서만 발생했고,
선행·후행 변경을 함께 보존한 뒤 roadmap 생성기로 정합을 확인했다.

표 편집 `u16` overflow 사전 검사(#4282), MCP 통계 키 상한(#4302), Python 공개 예외 호환성과
문서(#4304), Node 세션 timeout 뒤 실행 경계 종료(#4308), OWPML source 근거 경로(#4293)는
contributor의 의도를 유지하는 메인터너 보정으로 별도 커밋에 고정했다. 자세한 적용 경계와
보정 근거는 [누적 구현·검토 계획](pr_4282_review_impl.md)에 남겼다.

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4314](https://github.com/edwardkim/rhwp/pull/4314) |
| 대상 / head | `devel` / `review/kevin9327-20260809` |
| code candidate | `9a836c8e2e95c5e950520c9996c53411be03cb57` |
| base | `upstream/devel` `f62f7503f` |
| 작성자 | `jangster77` |
| 통합 원 PR | #4282, #4283, #4284, #4286, #4289, #4293-#4302, #4304-#4308 |
| 초기 Full CI | 2026-08-09 완료, 40개 check 성공 또는 의도적 skip, 실패 없음 |
| 초기 merge 상태 | `MERGEABLE`, `CLEAN` |

GitHub는 PR 작성자에게 reviewer request를 허용하지 않아 `jangster77` 지정 요청은 HTTP 422로
거부됐다. 따라서 병합 전 self-review는 `APPROVE`가 아닌 `COMMENT`로 남기며, 이는 플랫폼 제한을
우회하려는 것이 아니라 검토 책임과 기록을 분리하는 절차다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| Rust 전체 nextest | 5,499 passed, 35 skipped, 450.799초 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| Python `pytest` / mypy / ruff | 251 passed, 43 skipped / 통과 / 통과 |
| Node test / typecheck / build | 427 passed / 통과 / 통과 |
| 문서 metadata·link·roadmap 검사와 `git diff --check` | 통과 |

Rust 전체 검증은 고정 review target에서 다음으로 수행했다.

```bash
CARGO_INCREMENTAL=0 cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
```

## GitHub Actions

code candidate에서 CI preflight, CodeQL preflight, Render Diff preflight, Python·Node binding
matrix, Lint, Frontend package gates, Canvas visual diff, Native Skia, test archive, slow shard,
regular shard 1-3 및 `Build & Test` aggregate가 성공했다. WASM Build와 Frontend unit gates의
skip은 preflight가 변경 범위에 따라 의도적으로 결정한 상태다. 세부 실행은
[CI workflow](https://github.com/edwardkim/rhwp/actions/runs/31298931988)와
[CodeQL workflow](https://github.com/edwardkim/rhwp/actions/runs/31298932067)에서 확인했다.

이 문서와 오늘할일은 code candidate 뒤에 추가하는 review-only trailing commit이다. 최신 head는
`review_only_fast_pass.md` A 경로에 따라 candidate의 녹색 결과를 재사용해야 하며, preflight와
`Build & Test` aggregate 성공을 다시 확인한 뒤 병합한다.

## 병합·후속 처리

최신 문서 head의 fast-pass와 mergeability를 재확인한 뒤 self-review `COMMENT`를 남기고 병합한다.
병합 후에는 통합된 원 PR 20건에 #4314 반영 사실과 필요한 메인터너 보정 이유를 알린 뒤 종료한다.
정확히 해결된 보안 이슈 #4264, #4280, #4281, #4285, #4288도 통합 PR 반영 사실을 남기고 종료한다.
