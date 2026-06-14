# PR #1398 리뷰 처리 구현 기록

## PR 커밋

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1398 |
| 이슈 | #1385 |
| 원본 커밋 1 | `d0e866dafb5242a15f5d7c160f0292b4a7f41411` |
| 원본 커밋 2 | `0b04baaa90b76f5fe14b2d69a1f9e033cee2ccda` |
| 로컬 검토 브랜치 | `local/pr1398-review` |
| 상태 | merge 준비 가능 |

## Stage 1 - PR 상태 확인 - 완료

- PR #1398은 `devel` base이다.
- head는 `oksure:contrib/fix-1385-replace-all-raw-stream`이다.
- draft가 아니다.
- 작성자는 `CONTRIBUTOR`이다.
- `maintainerCanModify=true`이다.
- PR 본문에 `Closes #1385`가 있다.
- 변경 파일은 2개이다.

## Stage 2 - 이슈 확인 - 완료

이슈 #1385는 open 상태이며, 증상은 `replaceAll`/`replaceText` 변경이 메모리 모델에는 반영되지만
`exportHwp()` 저장 후 재오픈하면 원문으로 돌아가는 것이다.

이슈 코멘트에서 `set_field`는 0.7.15 기준 해소됐고, 범위는 `replaceAll`/`replaceText` 저장 유실로
좁혀졌다.

PR 작성자는 이슈에 원인 분석을 먼저 공유한 뒤 PR #1398을 열었다.

## Stage 3 - 로컬 검토 브랜치 구성 - 완료

`upstream/devel` 기준으로 새 검토 브랜치를 만들고 PR 원본 커밋 2개를 cherry-pick 했다.

```bash
git fetch upstream devel --prune
git fetch upstream pull/1398/head:local/pr1398-head
git switch -C local/pr1398-review upstream/devel
git cherry-pick d0e866dafb5242a15f5d7c160f0292b4a7f41411 0b04baaa90b76f5fe14b2d69a1f9e033cee2ccda
```

이후 upstream/devel에 아직 반영되지 않은 이전 리뷰 문서 커밋을 선별 적용했다.

```bash
git cherry-pick 6fec954d77e55b8a277c3b63c4edd522c1784965 316e0b78 784a4176 46c43ca0
```

최종 포함 커밋:

- `24891796 fix: replaceAll 치환 결과가 exportHwp 직렬화에서 유실 — raw_stream 캐시 무효화 (#1385)`
- `6c01b16d test: 리뷰 반영 — 글상자 순회 추가 + JSON 단언을 필드 파싱으로 교체 (#1385)`
- `9d0015b4 docs: PR #1374 처리 기록`
- `a7565e3a docs: PR #1374 처리 기록 EOF 정리`
- `bfb274bd docs: PR #1376 검토 기록`
- `1022e660 docs: PR #1376 코멘트 초안 한영 병기`

## Stage 4 - 로컬 검증 - 완료

```bash
cargo test --test issue_1385_replace_export_roundtrip
cargo build --lib
cargo test --lib
cargo clippy -- -D warnings
cargo test --doc
cargo test --test svg_snapshot
```

결과:

- `cargo test --test issue_1385_replace_export_roundtrip`: 2 passed
- `cargo build --lib`: 통과
- `cargo test --lib`: 1724 passed / 0 failed / 6 ignored
- `cargo clippy -- -D warnings`: 통과
- `cargo test --doc`: 0 passed / 0 failed / 1 ignored
- `cargo test --test svg_snapshot`: 8 passed

## Stage 5 - GitHub Actions 확인 - 완료

```bash
gh pr checks 1398 --repo edwardkim/rhwp
```

결과:

- `Build & Test`: pass, 13m 54s
- `CodeQL`: pass
- `Analyze (javascript-typescript)`: pass
- `Analyze (python)`: pass
- `Analyze (rust)`: pass
- `WASM Build`: skipped

## Stage 6 - merge 전 확인 필요 - 대기

PR #1398은 코드와 테스트 관점에서 merge 가능하다.

남은 작업:

1. #1374/#1376 리뷰 문서 커밋을 PR head에 반영한다.
2. PR diff에 의도한 문서와 #1398 코드만 포함되는지 확인한다.
3. 작업지시자 승인 후 merge한다.
4. merge 후 #1385 close 여부를 확인한다.
