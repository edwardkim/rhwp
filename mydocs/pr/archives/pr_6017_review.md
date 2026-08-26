# PR #6017 검토 기록

- PR: https://github.com/edwardkim/rhwp/pull/6017
- 제목: `fix(renderer): Square 호스트 빈 문단도 저장 사다리로 줄 예약을 판별한다 — 본문 전체 22.6px 상향 밀림 해소 (#5809 실측 ①)`
- 작성자: `planet6897`
- base/head: `devel` <- `fix/5809-square-owner-empty-para-charge`
- 검토 브랜치: `review/planet6897-6017-20260825`
- 검토 시각: 2026-08-25 00:32 KST

## 결론

수용 가능.

차단 결함은 발견하지 못했다. PR head `9c04179ea62c5d799d1aa6bcf7b5fdc8695697f3`은 GitHub CI
전 항목이 성공했고, 최신 `upstream/devel` 위 로컬 병합도 충돌 없이 완료됐다. 로컬에서는 신규 #5809
좌표 회귀와 기존 #2069 OLE enter/backspace 반증 회귀를 함께 확인했다.

메인터너 보정 필요 없음.

## 변경 요약

- `src/renderer/layout.rs`
  - 빈 non-TAC Picture/Shape host 문단의 저장 vpos 사다리 판정을 Square/Tight/Through wrap까지 확장.
  - 다음 문단이 가시 텍스트일 때만 Square 계열 저장 사다리 증언을 사용.
  - `TAG_IMPLEMENTATION_PROPERTY` line segment와 호스트 줄 규모를 크게 벗어나는 stale delta는 저장 증거에서 제외.
  - 사다리가 예약을 증언한 경우 전진량을 `next_seg.vertical_pos - seg.vertical_pos` 저장 델타로 사용해
    spacing-before 포함 줄 예약량을 보존.
- `tests/cases/issue_5809_square_host_empty_para_charge.rs`
  - `samples/issue5809/156518601_p1_square_host.hwpx` 1쪽에서 `경찰청(청장...` 줄 상단이 저장 사다리
    기준 `619.7px ± 2px`인지 고정.
- `samples/issue5809/156518601_p1_square_host.hwpx`
  - 원본 1쪽 축소 재현물. SHA-256:
    `ed9a0589d9223c2750f4fb8240548551d4aa70fd35d6243f405183d525ff1f1f`.

## 로컬 검증

최신 `upstream/devel` 위에서 PR head를 병합해 검증했다.

```bash
git merge-tree --write-tree upstream/devel upstream/pr6017-head
```

- 결과: 충돌 없음.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check
```

- 결과: 통과. 911 sources / 4266 static test attrs / 32 suites + 9 exceptions.

```bash
node scripts/run-rust-test.mjs issue_5809_square_host_empty_para_charge -- \
  --cargo-profile release-test --target-dir target/pr-review
```

- 결과: 1 test run, 1 passed.

```bash
node scripts/run-rust-test.mjs issue_2069_ole_object_selection -- \
  --cargo-profile release-test --target-dir target/pr-review
```

- 결과: 10 tests run, 10 passed.
- 목적: Square 사다리 확장이 기존 한셀 OLE enter/backspace 흐름을 되돌리지 않는지 확인.

```bash
cargo fmt --all -- --check
git diff --check
```

- 결과: 둘 다 통과.

## GitHub CI

`gh pr view 6017 --json statusCheckRollup` 기준:

- CI preflight: success
- Build & Test: success
- Lint (fmt, clippy, WASM check): success
- Native Skia tests: success
- build-test-archive-a/b/c: success
- default-feature archive shards: success
- Render Diff preflight / Canvas visual diff: success
- Proptest preflight / prop roundtrip: success
- Adapter inter-diff preflight / adapter inter-diff: success
- CodeQL analyze javascript-typescript/python/rust: success

## 시각 증적

`rhwp info --json samples/issue5809/156518601_p1_square_host.hwpx` 결과:

```json
{
  "format": "hwpx",
  "pageCount": 1,
  "version": "5.1.0.0",
  "lastSavedWith": null,
  "sizeBytes": 20951,
  "warnings": []
}
```

`lastSavedWith`가 `null`이므로 자동 MCP 선택 대상은 아니다. 다만 PR 본문이 한글 2022 실측 기준을 명시하고,
MCP 산출 PDF의 `pdfinfo`도 `Creator: Hwp 2022 0.0.0.0`로 확인되어, review용 기준 PDF는 HWP 2024 MCP
통합 service의 engine `2020`으로 산출했다.

- MCP job id: `d753c87e-87ea-420d-b8ce-bb8e2d2ecbd2`
- engine: `2020`
- engine_profile: `2020`
- hancom_version: `12.0.0.4605`
- backend: `hwp-managed-direct-dll-host`
- PDF: `pdf/pr6017/156518601_p1_square_host-2020.pdf`
- PDF SHA-256: `05f5e1c2f8a651f088c9872766ed2bf9b7c48dbdc08617a856dc510c68b44137`
- PDF size: 83,759 bytes
- PDF pages: 1

```bash
python3 scripts/visual_sweep.py \
  --key pr6017_issue5809_square_host \
  --hwp samples/issue5809/156518601_p1_square_host.hwpx \
  --pdf pdf/pr6017/156518601_p1_square_host-2020.pdf \
  --page 1 \
  --rhwp-bin target/pr-review/release-test/rhwp \
  --out output/visual_sweep_pr6017_issue5809
```

- summary: `mydocs/pr/assets/pr_6017_issue5809_visual_sweep_summary.json`
- 대표 review PNG: `mydocs/pr/assets/pr_6017_issue5809_square_host_p1_review.png`
- completed_pages: `[1]`
- flagged_page_count: `0`
- pixel match: `89.98028%`
- visual accuracy proxy: `10.25324%`

내용 픽셀 중심 자동 일치율은 보조 지표이며 사람 판정을 대체하지 않는다. review PNG 기준으로 첫 페이지의
표/제목/본문 흐름은 기준 PDF와 같은 큰 배치이며, #5809의 문제였던 Square host 뒤 본문 전체 상향 밀림은
재현되지 않았다.

## 검토 메모

- 새 사다리 판정의 `i32` 산술은 기존 함수에도 있던 성격의 문제다. 이 PR의 변경으로 새로 발생한 차단 결함으로
  보지는 않았다.
- #5809 수정 범위는 PR 본문처럼 실측 ① 축이며, 실측 ② 저장 사다리 의미론 축은 별도 후속 이슈 영역으로 남긴다.
