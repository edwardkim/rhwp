---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6195 review - #6171 Hanyang font alias

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6195
- 작성자: `planet6897`
- 원 PR head: `0b5a810b22dcbe117dc0b963edcf2625aa145850`
- 통합 검토 브랜치: `review/open-prs-6178-6198-20260827`
- 기준: `upstream/devel@529ab90c25d5`
- 적용 문서: `maintainer_general`, `intake_and_review`, `local_validation`,
  `multi_pr_update_branch`, `visual_fixture_evidence`

## 검토 판단

**수용 가능**. 한양견고딕·한양견명조 alias를 설치 face 이름으로 연결해 SVG/font fallback에서
해당 글꼴이 빈칸이나 대체 glyph로 빠지는 문제를 줄인다. golden SVG 갱신도 함께 포함되어 있고,
대표 crop에서 한글 글자가 oracle과 같은 위치와 굵기로 보이는 것을 직접 확인했다.

## 증적과 검증

- 전체 회귀: `8438 passed`, `43 skipped`, `10 slow`
- clippy: `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` pass
- native-skia:
  - lib tests `167 passed`
  - `issue_2225_missing_picture_placeholder`: `2 passed`
  - `render_p37_direct_pdf_export`: `4 passed`
- WASM: `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` pass
- 시각 증적 직접 확인:
  - `mydocs/report/hanyang-gyeongothic-alias-6171/after_p1.png`
  - `mydocs/report/hanyang-gyeongothic-alias-6171/oracle_p1.png`
- 증적 SHA: `mydocs/pr/assets/pr_6178_6182_6188_6193_6195_6198_visual_evidence_sha256.tsv`

## 후속

병합 후 원 PR과 관련 이슈에는 대표 crop과 native/WASM 검증을 근거로 alias 반영 완료를 기록한다.
