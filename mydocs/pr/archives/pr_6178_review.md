---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6178 review - #6132 stored vpos overflow page break

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6178
- 작성자: `planet6897`
- 원 PR head: `8bfd9404fa12d3e435ffe7551eec11f1aeaf0a39`
- 통합 검토 브랜치: `review/open-prs-6178-6198-20260827`
- 기준: `upstream/devel@529ab90c25d5`
- 적용 커밋: `7dce818a1`, `8bfd9404f`
- 적용 문서: `maintainer_general`, `intake_and_review`, `local_validation`,
  `multi_pr_update_branch`, `visual_fixture_evidence`

## 검토 판단

**수용 가능**. 첫 체리픽 뒤 원 PR에 추가된 `8bfd9404f`를 다시 확인해 통합 브랜치에
`d5d43caa9`로 반영했다. 추가 커밋은 저장 `vpos`가 쪽 본문을 넘는 신호를 표 문단, 근소 초과,
잔여 영역 조건으로 다시 좁혀 #6132 대상 문단만 다음 쪽으로 보내도록 보정한다.

원 PR 코멘트에 이전 판정 신호가 넓다는 우려가 남아 있었지만, 최신 head는 non-draft/CLEAN이고
원 PR CI도 실패 없이 완료됐다. 통합 브랜치의 전체 회귀에서도 기존 장시간 코퍼스 케이스를 포함해
전건 통과했다. 추가 차단 결함은 발견하지 못했다.

## 증적과 검증

- `rhwp info --json samples/issue6132/156482639_startup_ir_contest.hwp`:
  `format=hwp5`, `version=5.0.5.0`, `lastSavedWith.version=9.6.1.6189`, `pageCount=10`
- focused: `node scripts/run-rust-test.mjs issue_6132_stored_vpos_overflow_page_break -- --cargo-profile release-test --target-dir target/pr-review`
  - `1 passed`, `126 skipped`
- 전체: `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  - `8438 passed`, `43 skipped`, `10 slow`, `924.930s`
- 시각 증적 직접 확인:
  - `mydocs/report/stored-vpos-overflow-6132/after_p7.png`
  - `mydocs/report/stored-vpos-overflow-6132/oracle_p7.png`
  - `mydocs/report/stored-vpos-overflow-6132/after_p8.png`
  - `mydocs/report/stored-vpos-overflow-6132/oracle_p8.png`
- 증적 SHA: `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198_visual_evidence_sha256.tsv`

## 후속

통합 PR 생성 전 최신 `upstream/devel` 기준으로 충돌 여부를 다시 확인한다. 병합 후에는 #6132와
원 PR #6178에 `8bfd9404f` 추가 반영, 전체 회귀, 대표 시각 증적을 근거로 통합 완료 코멘트를 남긴다.
