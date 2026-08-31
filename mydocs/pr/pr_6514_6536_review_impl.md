# planet6897 #6514/#6536 체리픽 통합 검토 기록

- 검토일: 2026-09-01
- 통합 브랜치: `review/planet6897-6514-6536-20260831`
- base: `upstream/devel@891e395bb`

## 적용 계보

| PR | 원 head | 통합 commit | 결과 |
| --- | --- | --- | --- |
| #6514 | `b643b3822edccaa234133fc4cf2701910b090b8f` | `c8708e2d8` | 승인 |
| #6536 | `8e4269db82cae5a45115f332c2fb80a467a45f32` | `b8041f23c` | 변경 요청 |

## 통합 검증

- 원 PR head 기준 CI: #6514 성공 28건, #6536 성공 27건, 실패·대기 0건.
- `node scripts/rust-test-suite-manifest.mjs --prepare/check`: 48/48 targets, 최소 6559 cases.
- Rust format, native/WASM/workspace/all-target Clippy, workspace build 통과.
- focused `issue_5678_fit_test_letter_spacing_trim`, `issue_6535_page_anchored_block_keeps_page` 종료 코드 `0`.
- full `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` 종료 코드 `0`.

## 시각 검증과 결론

- #6536 fixture를 Hancom 2020 direct-dll-host로 PDF 변환해 physical p1을 비교했다. PDF SHA-256은 `d5a4a5f8702937d835aba7111c1c72dbbdfed6297c6d1ae3eff23ae656e8c66b`이다.
- [p1 review 패널](assets/pr_6536_issue6535_p1_review.png)은 flagged `0`이지만, Hancom PDF의 표 앞 `2.` 문단이 rhwp에서 표 뒤로 이동한 것을 보인다.
- 따라서 #6536은 P1 변경 요청이며, 이 통합 브랜치로 remote PR을 만들거나 병합하지 않는다.
- #6514는 독립 변경으로 승인 가능하지만, 현재 요청의 통합 범위에서는 #6536 수정본과 함께 재검토한다.

