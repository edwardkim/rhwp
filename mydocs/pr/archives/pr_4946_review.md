---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4946 검토 - HWP5 글꼴 기본 이름 실측표

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4946](https://github.com/edwardkim/rhwp/pull/4946) |
| 작성자 / source | @planet6897 / `fix/4898-font-default-name` |
| 원 source head | `62fe2dc775f1b2aaa2d6bb6ac16121246af51c2b` |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| 가시성 검토 branch | `review/planet6897-20260816-r6` |
| local 적용 commit | `328e6e886aeb97d3d61164113a427d9c325bd9ac` |
| 원 PR 상태 참고값 | `MERGEABLE` / `CLEAN` |

HWPX 출처 글꼴은 HWP5 `FACE_NAME`의 영문 기본 이름이 비어 있을 수 있다. serializer는 명시된
`default_name`을 우선하고, 없는 경우에만 실측표의 이름을 채우며 미확인 이름을 추정하지 않는다.

## 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 글꼴 단위 | `cargo test --profile release-test --target-dir target/pr-review --lib issue4898_face_name_fills_measured_default_font_name -- --nocapture` | 1 passed |
| 실제 HWPX→HWP5 | `target/pr-review/release-test/rhwp convert samples/hwpx_sample2.hwpx /tmp/rhwp-pr4946-hwpx_sample2.hwp --verify --json` | HWP5 300,032 bytes, `identical:true`, `diffCount:0` |
| 누적 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,514 passed, 38 skipped, 7 slow, 378.554초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD` | 통과 |
| 원 source CI | 최신 head의 Build & Test와 필수 분석 job | 성공; CodeQL aggregate는 `NEUTRAL` |

실제 변환 과정에서 알려진 HWP5 Hyperlink 확장 파라미터 손실 경고(#4396)가 stderr에 있었지만,
이 PR의 글꼴 기본 이름 경계와 무관하며 `--verify` IR 판정은 성공했다.

## 판단

명시값 우선·미확인값 미추정 원칙과 실제 HWPX→HWP5 자기검증이 함께 확인됐다. 현재 후보에서 글꼴 표가
새로운 페이지 수 충실도 해결책이라고 과장하지 않으며, 별도 fidelity 문제는 분리된 상태다.
**통합 수용 권고.**
