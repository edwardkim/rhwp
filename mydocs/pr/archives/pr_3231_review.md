---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3231 검토 기록 — 선택 삭제 undo 스냅샷 전환

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#3231](https://github.com/edwardkim/rhwp/pull/3231) |
| 작성자 | `lpaiu-cs` |
| 원 head | `f871be405a738e93142ef3d5a3059fc781dbae83` |
| base / 상태 | `devel` / `BEHIND` |
| 누적 검토 브랜치 | `review/lpaiu-cs-20260724` (`upstream/devel` `c8611dd84d002d2a776c040387bf21cf270f6448`) |
| 적용 순서 / 로컬 SHA | 3) `5f70c9ab4` → `cf00036ce` |
| 충돌 | 없음 |

## 변경과 판단

평문 `savedTexts`를 재삽입하던 선택 삭제 undo를 기존 스냅샷 기반 paste/replace 경로와 맞췄다. 따라서
글자 모양 run, 다문단 메타, 인라인 control을 별도 재구성하지 않고 before/after 문서 상태로 복원한다.
`DeleteSelectionCommand` 타입과 `kind:'command'`를 유지해 양식 모드 operation gate를 우회하지 않은 점도
확인했다.

선택 삭제 하나가 before/after snapshot 두 개를 쓰므로 history 깊이가 줄어드는 비용은 결함이 아니라
명시된 설계 선택이다. 저장소의 snapshot 예산 조정은 이미 이슈 [#3230](https://github.com/edwardkim/rhwp/issues/3230)으로
분리돼 있어 이번 변경에 추정 조정을 섞지 않았다.

## 독립 검증

| 검증 | 결과 |
| --- | --- |
| `rhwp-studio: npx tsc --noEmit` | PASS |
| `rhwp-studio: npm test` | 636 passed, 0 failed |
| `cargo test --profile release-test --tests` | PASS |
| `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --doc` | PASS (doc 4 passed, 2 ignored) |
| `wasm-pack build --target web --out-dir pkg` | PASS |

## 최종 권고

**통합 PR 수용 후보.** #3228 뒤, #3240 앞의 누적 체리픽으로 충돌 없이 적용했다. 원 PR의 `BEHIND` 상태는
직접 merge가 아닌 최신 `devel` 기반 통합 PR로 해소한다.
