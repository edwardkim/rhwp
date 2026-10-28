---
kind: pr-review-implementation
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# Open PR CI-green 통합 검토 구현 기록 - 2026-08-26

## 기준과 포함 범위

- 통합 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@a9a590963c0a`
- #6142는 이미 `2026-08-26T09:02:49Z`에 merge commit `1011a89475c9`로 병합되어 기준선에 포함됐다.
- 포함 PR:
  - #6088, #6089, #6091, #6092, #6093, #6094, #6096, #6097, #6098, #6100, #6103, #6105,
    #6113, #6115, #6119, #6120, #6131, #6136, #6137, #6144
- 제외 PR:
  - draft: #5953, #6059
  - CI 미통과 또는 진행 중: #6073, #6148
  - 최신 `upstream/devel` 기준선에 이미 포함: #6116
  - 이미 #6142로 병합되어 `upstream/devel`에 있는 패치: #6075, #6077, #6079, #6080, #6084
  - review 보류: #6083. 기존 `pr_6083_review.md`와 PR comment에서 편람 61쪽 새 visual regression을
    기록했으므로 통합하지 않았다.

## 메인터너 보정

- #6093은 #6142의 측정 기반 「」 폭 판정과 충돌했다. 최신 기준의 #6142 구현을 유지하고 #6093의
  fixture·테스트·시각 증적만 보존했다.
- #6097 계열에서 #6025/#6035 조합의 CENTER 셀 source-frame 판정 차이를 `cd4866211`로 보정했다.
- `rhwp-studio/e2e/MANIFEST.md`에는 tracked e2e 파일 4건
  (`issue-6099-probe.mjs`, `loading-busy-cursor.test.mjs`, `status-page-number.test.mjs`,
  `toolbox-visibility.test.mjs`)을 추가했다. `issue-6099-probe.mjs`는 진단 파일명 규칙을 맞추기 위해
  `legacy-name` note를 명시했다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| Rust suite manifest prepare/check + unit tiers | 통과, 960 sources / 4,357 static test attrs |
| `python3 scripts/tests/test_cancel_stale_pr_runs_workflow.py -v` | 통과, 4 tests OK |
| focused renderer nextest 13개 suite | 통과, 1,691 pass / 6 skip |
| 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast` | 통과, 8,399 pass / 43 skip |
| Studio unit focused set | 통과, 100 tests pass |
| `npm run build` in `rhwp-studio` | 통과 |
| Studio e2e manifest + 3개 headless E2E | 통과 |
| `scripts/wasm-pack-locked.sh --target web --out-dir pkg` | 통과, 8m 57s |
| native-Skia `cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib` | 통과, 3,948 pass / 13 ignored + crate unit 정상 |
| native-Skia `issue_2225_missing_picture_placeholder` | 통과, 2 pass |
| native-Skia `render_p37_direct_pdf_export` | 통과, 4 pass |

Docker compose WASM 경로는 현재 `ubuntu-ted`에 `docker` 명령이 없어 실행하지 못했다. 대신 같은 checkout에서
`wasm-pack` 표준 build를 완료했다.

## 시각 증적 확인

각 renderer/HWP fixture 변경 PR은 source PR report asset만으로 수용 판단을 끝내지 않고,
`rhwp info --json`, Hancom 2020 PDF, `visual_sweep.py` review PNG와 metrics를 별도 asset으로 보존했다.
MCP는 한 번에 여러 job을 새로 넣지 않고, 이미 큐에 들어간 job의 상태만 polling해 성공분을 다운로드했다.

| 범위 | 결과 |
| --- | --- |
| MCP Hancom 2020 PDF + visual_sweep | #6088/#6097/#6098/#6119/#6120/#6131 대상 4개 문서 완료, `flagged_page_count=0` |
| local Hancom 2020 PDF + visual_sweep | #6089/#6093/#6094/#6096/#6100/#6103/#6105/#6113/#6137 완료, `flagged_page_count=0` |
| #6144 local visual_sweep | 실행 완료, target 헤더 crop 개선 확인. 다만 `line_order_overlap`, `column_line_band_drift` flag가 남아 코멘트 필요 |

- source report contact sheet: `mydocs/pr/assets/pr_6088_6144_source_after_contact_sheet.png`
- local visual_sweep contact sheet: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_batch_contact_sheet.png`
- MCP visual_sweep contact sheet: `mydocs/pr/assets/pr_6088_6144_mcp2020_visual_sweep_contact_sheet.png`
- 전체 ledger: `mydocs/pr/assets/pr_6088_6144_visual_evidence_ledger.tsv`
- local metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`,
  `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv`
- MCP metrics: `mydocs/pr/assets/pr_6088_6144_mcp2020_visual_sweep_metrics.tsv`
## 결론

현재 통합 후보는 #6142 merge 이후 최신 `upstream/devel` 기준으로 재정렬됐고, source PR CI가 통과한
non-draft PR을 기준으로 구성했다. #6093 충돌과 #6097 source-frame 차이는 메인터너 보정으로 처리했다.
#6144는 target 개선은 확인됐지만 visual_sweep 경고가 남아 코멘트에 명시한다.
