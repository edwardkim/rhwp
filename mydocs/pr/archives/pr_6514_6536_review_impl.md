# planet6897 #6514/#6536 체리픽 통합 검토 기록

> **후속 판정으로 대체됨:** 이 문서는 #6541 통합 초기에 내린 중간 판정을 보존한 기록이다.
> 아래 `0ff2e25b6`의 “보정 완료/수용 가능” 판정은 후속 bbox 검토에서 표와 종결문 겹침이
> 확인되어 철회됐고, 최종 보정 `7cf17c1ce` 및 현재 판정은
> [`pr_6536_review.md`](pr_6536_review.md)를 정본으로 삼는다.

- 검토일: 2026-09-01
- 통합 브랜치: `review/planet6897-6514-6536-20260831`
- base: `upstream/devel@891e395bb`
- 최종 통합: PR #6541, merge commit `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`

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
- [p1 review 패널](../assets/pr_6536_issue6535_p1_review.png)은 flagged `0`이지만, Hancom PDF의 표 앞 `2.` 문단이 rhwp에서 표 뒤로 이동한 것을 보인다.
- 따라서 #6536은 P1 변경 요청이며, 이 통합 브랜치로 remote PR을 만들거나 병합하지 않는다.
- #6514는 독립 변경으로 승인 가능하지만, 현재 요청의 통합 범위에서는 #6536 수정본과 함께 재검토한다.


## #6536 메인터너 중간 보정 (2026-09-01, 후속 판정으로 대체됨)

#6536의 원 contributor head `8e4269db82cae5a45115f332c2fb80a467a45f32`는 빈 host paragraph 뒤 양수 offset 표가 다음 생성 본문을 앞질러 배치하는 P1 시각 오류를 포함했다. 검토 브랜치의 `0ff2e25b6`에서 본문 anchor 복원과 표 anchor 보존을 함께 적용하고, `2.` 본문 -> `연번` 표 -> `끝.` 순서를 회귀 테스트로 고정했다.

- 당시 결론: **메인터너 보정 후 수용 가능**. 후속 bbox 검토에서 철회됐으며 현재 판정은
  [`pr_6536_review.md`](pr_6536_review.md)를 따른다.
- 로컬 검증: lint 묶음, Native Skia, locked WASM package, focused #6535, 전체 nextest `8,912 passed, 46 skipped` 통과.
- 시각 증적: `mydocs/pr/assets/pr_6536_issue6535_p1_2020.pdf` (SHA-256 `d5a4a5f8702937d835aba7111c1c72dbbdfed6297c6d1ae3eff23ae656e8c66b`) 및 `mydocs/pr/assets/pr_6536_issue6535_p1_maintainer_review.png`을 함께 보관했다. physical page 1, visual sweep flagged 0, 사람 검토 순서 일치.
- 당시 원격 조건은 보정 commit을 포함한 integration head의 최신 CI와 mergeability 확인이었다. 이후
  최종 후보 `88d0924e550041746464627bc7bc32b1a2511177`이 승인·검증됐고 정상 merge commit
  `e9d2f8b258b8310fd10d465b486b9ab4d85e771e`로 병합됐다. 원 contributor fork head는 변경하지 않았다.
