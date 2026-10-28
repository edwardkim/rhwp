# Task M100 #6381 Stage 2 완료보고 — fail-closed 구현

- **이슈**: [#6381](https://github.com/edwardkim/rhwp/issues/6381)
- **구현 commit**: `21c3bd43c`
- **CLI 문서 commit**: `025ce6806`
- **상태**: focused·정적 검증 완료, Stage 3 장기 검증 승인 대기

## 1. 구현 결과

- 네 setter의 성공 여부를 개별 기록하고 실패 원인을 stderr에 남긴다.
- mutation에 성공한 대상은 실제 control이 Picture인지, caption이 존재하는지, 방향·세로 정렬·폭 8504·
  간격 850이 정확한지 확인한다.
- 한 대상이라도 실패하면 출력 폴더를 만들거나 SVG를 렌더하기 전에 exit 1로 종료한다.
- page가 0개인 문서도 SVG 성공으로 오인하지 않고 exit 1로 종료한다.
- 네 대상이 모두 통과한 경우에만 기존 page 순회, SVG 파일명과 `완료` 출력을 수행한다.
- 성공 경로의 기존 `caption=Some(...)` stdout 형식을 유지했다.

renderer·layout·document model·Render Diff workflow는 변경하지 않았다. 고정 대상 자동 탐색이나 새로운
일반 사용자 명령도 추가하지 않았다.

## 2. focused green 검증

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --test regression_suite_004 \
  -E 'test(/issue_cli_test_caption_no_panic/)' --no-fail-fast
```

- nextest run: `e4363ad1-5d3f-434d-832a-b367d4290ad0`, 3/3 pass
- 성공 stdout 형식 보존 뒤 재실행: `db0f754a-dd33-4ece-bc6d-a332c8af2ab6`, 3/3 pass

## 3. 정적·문서 preflight

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check
node scripts/rust-unit-test-tiers.mjs --check
cargo fmt --all -- --check
python3 scripts/check_markdown_links.py
git diff --check
```

- integration manifest: 1,031 sources / 4,531 static test attrs / 48 targets 확인
- unit tier: 4,221 tests / 299 modules 확인
- Markdown 605개 상대 링크 이상 없음
- format·diff check 통과
- `--prepare`가 만든 `tests/generated/`, `tests/suites/manifest.json`은 ignored 검증 증적이며 stage하지 않았다.

## 4. 다음 승인 게이트

저장소 long-gate 규칙에 따라 다음 Stage 3은 작업지시자 승인 뒤 실행한다.

1. `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`
2. 전체 integration nextest (`--tests --no-fail-fast`)
3. 최종 보고서와 PR 본문 초안 준비

remote push와 PR 생성은 Stage 3 승인과 별개이며, 다시 별도 승인을 받아야 한다.
