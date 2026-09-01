---
doc_kind: pr_review
title: "PR #6536 메인테이너 리뷰"
status: archived
issue: 6535
pr: 6536
reviewed_at: 2026-09-01
---

# PR #6536 메인테이너 리뷰

> **후속 판정으로 대체됨:** 이 문서는 원 PR head와 첫 integration 보정의 P1 차단 판정을
> 보존한 기록이다. bbox 하단 기준 보정과 최종 시각 판정은 PR #6541의 `7cf17c1ce`에서
> 완료됐고, 현재 판정과 merge 계보는 [`pr_6536_review.md`](pr_6536_review.md)를 정본으로 삼는다.

## 1. 최종 판정

- 판정: **머지 보류**
- 원 PR head: `8e4269db82cae5a45115f332c2fb80a467a45f32`
- 검토 base: `upstream/devel@336c4526e9cc5047d6dd9906ebc8d0d5ee6f2188`
- 원 PR은 빈 페이지를 제거해 1페이지를 만들지만, 본문과 표의 문서 순서를 뒤집는 P1 시각 회귀를 포함한다.
- 통합 PR #6541의 메인테이너 보정 `0ff2e25b6`도 종결문 `끝.`을 표 내부 세로 범위에 남겨 유효한 보정 head가 아니다.
- 원격 조치: 이 기록은 GitHub review/comment, close, push 또는 merge를 수행하지 않는다.

## 2. 라우팅과 metadata

- base route: `maintainer_general.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `multi_pr_update_branch.md`
- 작성자: `planet6897` — 기존 외부 contributor
- base/head branch: `devel` ← `fix/6535-topbottom-table-zero-charge`
- 규모: 1 commit, 3 files, `+82/-0`
- 변경: `src/renderer/typeset.rs`, 신규 `tests/cases` 회귀 테스트, 신규 HWPX fixture
- 문서 작성 시점 참고값: open, non-draft, MERGEABLE/CLEAN, exact-head CI green
- reviewer: `edwardkim` 지정 완료
- related issue: #6535 open

## 3. 발견 사항

### P1 — 원 PR은 페이지 수를 맞추지만 본문과 표의 읽기 순서를 뒤집음

원 PR의 `block_anchor_vpos_is_absolute` 예외는 쪽-앵커 블록의 저장 `vpos`로 본문 흐름을
동기화하지 않아 fixture를 2페이지에서 1페이지로 줄인다. 그러나 직접 시각 비교하면 Hancom 2020
PDF에서 표 위에 있는 `2.` 본문이 rhwp에서는 표 아래로 이동한다.

신규 테스트는 `page_count == 1`과 page 1의 표 개수만 확인한다. 최신 `devel` merge tree에서 해당
focused test는 통과했으나 이 순서 오류를 검출하지 못했다. 따라서 녹색 CI와 쪽수 기준선은
사용자-visible 의미 순서를 보장하는 증거가 아니다.

### P1 — #6541의 기존 메인테이너 보정도 표-종결문 겹침을 놓침

통합 PR #6541은 원 patch와 stable patch-id가 같은 `b8041f23c` 뒤에 `0ff2e25b6`을 추가해
`2.` 본문을 표 위로 되돌린다. 보정 테스트는 다음 조건만 확인한다.

```rust
body_y < table_y && table_y < ending_y
```

이 조건은 `table_y`가 표의 위쪽 좌표라는 점을 놓친다. 보정 head의 실제 render tree는 다음과 같다.

| 개체 | y | h | bottom |
|---|---:|---:|---:|
| `연번` 표 | 456.2 | 178.4 | 634.6 |
| `끝.` text run | 600.1 | 17.3 | 617.4 |

즉 `table_y < ending_y`는 참이지만 `끝.` 전체가 표의 세로 범위 안에 있다. 직접 연
`pr_6536_issue6535_p1_maintainer_review.png`에서도 rhwp의 `끝.`이 표 첫 셀 내부에 보이고,
Hancom PDF에서는 표 아래에 있다. #6541의 CI와 focused test가 모두 녹색이어도 보정은 완료되지 않았다.

해제 조건은 최소한 `table_bottom <= ending_y`를 검증하고, 실제 배치에서도 표와 종결문이 겹치지 않도록
flow 소비를 보정한 새 integration head와 Hancom 2020 page 1 직접 시각 판정이다.

### P2 — 부분 해결 PR이 #6535 전체를 자동 close하도록 작성됨

PR 본문 첫 줄은 `Fixes #6535 (부분 — 7건 중 2건)`이다. GitHub의 `Fixes #6535`는 merge 시
이슈 전체를 자동 close하지만, PR 설명 자체가 7건 중 5건은 다른 원인으로 남았다고 명시한다.

직접 병합 후보라면 `Refs #6535`로 바꾸거나, 해결된 2건과 남은 원인군을 별도 issue로 분리한 뒤 closure
관계를 명시해야 한다. 현 상태에서 #6535는 close하면 안 된다.

## 4. 계보와 시각 증적

- 원 fixture: `samples/issue6535/36404612_page_anchored_footer_block.hwpx`
  - SHA-256: `c187c26644dcdece4b784751250fbcaaa274b34e7662de73505fe0bab6bc013e`
  - `rhwp info --json`: format `hwpx`, last saved with `hancom-office-2020` `11.0.0.8227`
- Hancom 2020 기준 PDF: #6541
  `mydocs/pr/assets/pr_6536_issue6535_p1_2020.pdf`
  - SHA-256: `d5a4a5f8702937d835aba7111c1c72dbbdfed6297c6d1ae3eff23ae656e8c66b`
- 원 PR 비교 패널: #6541
  `mydocs/pr/assets/pr_6536_issue6535_p1_review.png`
  - SHA-256: `2b8b4528df3b056aea0f8c3682971e4111d8b93faaa3d0304df280ce963de34c`
- 기존 보정 패널: #6541
  `mydocs/pr/assets/pr_6536_issue6535_p1_maintainer_review.png`
  - SHA-256: `62494c68d0f88e7acb65416b7102155565f96786e3c7b502db6b830b3c3ca604`
- 원 commit `8e4269db8`과 #6541 통합 commit `b8041f23c`의 stable patch-id:
  `fcbe5e98a3550db3d8d379e67ac7c2359c2cbebd`

자동 지표의 flagged `0`은 사람 판정을 대체하지 못했다. 원 패널과 보정 패널 모두 자동 후보는 없지만,
사람이 읽는 문서 순서와 개체 겹침에는 명백한 차이가 있다.

## 5. 검증 결과와 생략 범위

| 검증 | 결과 |
|---|---|
| 최신 `upstream/devel` 비커밋 merge simulation | PASS, 충돌 없음 |
| `git diff --cached --check` | PASS |
| integration manifest prepare/check | PASS, 1098 sources / 48 targets |
| 원 PR focused `issue_6535_page_anchored_block_stays_on_its_page` | PASS, 1/1 |
| 원 PR exact-head GitHub CI | green |
| #6541 exact-head GitHub CI | green |
| 원 PR 패널 직접 이미지 판정 | FAIL, `2.`가 표 뒤 |
| #6541 보정 패널 직접 이미지 판정 | FAIL, `끝.`이 표 세로 범위와 겹침 |
| #6541 보정 render-tree bbox | FAIL, `600.1 < table bottom 634.6` |

원 code head와 #6541 integration head의 광범위 Rust CI가 이미 녹색이고 사용자-visible P1 blocker를
직접 재현했으므로, 이 리뷰에서는 전체 nextest·Native Skia·WASM을 중복 실행하지 않았다. 수정된 새 code
head가 나오면 renderer 검증 묶음과 직접 시각 검증을 다시 수행해야 한다.

## 6. 보류 해제 조건

1. `2.` 본문 → `연번` 표 → `끝.`의 순서뿐 아니라 비겹침을 bbox 하단 기준으로 고정한다.
2. 실제 typeset/placement flow를 보정해 `끝.`의 위쪽이 표 하단 이상이 되도록 한다.
3. 새 integration head에서 focused test, 필수 Rust lint, 전체 renderer 검증 및 locked WASM을 통과한다.
4. Hancom 2020 page 1과 다시 비교해 세 개체의 순서와 비겹침을 사람이 확인한다.
5. `Fixes #6535`를 미해결 범위와 일치하도록 정정하고 #6535를 open으로 유지한다.
6. #6514 blocker도 별도로 해소한 뒤에만 두 변경을 포함한 #6541을 병합 후보로 재검토한다.
