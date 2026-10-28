# #6695 Stage 3 — 활성 소비자와 CI 계약 현행화

## 1. 판정

**Stage 3 구현·로컬 검증 완료**다. 메인테이너 결과 승인 전이며 원격에는 push하지 않았다.

- workflow·CI 정책의 현행 PDF 오라클 입력은 `pdf/**` 하나다.
- `pdf-2020/**`·`pdf-large/**`는 fast-pass에서 제외되어 다시 등장하면 full/fail-closed로 간다.
- Oracle Public 정본은 `pdf/` 1,178개를 읽어 1,030쌍을 재생성했다.
- 오라클 버전은 파일명에서만 탐지하며, 접미사가 없는 110개는 `unknown`으로 보존했다.
- LLM verifier 122,400행은 단일 `pdf` root 계약으로 재생성·전수 검증했다.
- 현행 운영·검증·배포 문서는 새 경로를 사용하고, historical memory는 본문을 보존한 채
  superseded 안내를 추가했다.

## 2. CI와 workflow 경계

다음 workflow의 PDF 경로 계약을 `pdf/**` 하나로 축소했다.

- CI, CodeQL, Pages push `paths-ignore`
- CI, CodeQL, Render Diff, Adapter Diff, Proptest Roundtrip의 review-only 판정
- Oracle Public Advisory sparse checkout과 실행 인수

Oracle Public Advisory에서는 `include_large` 입력, checkout LFS 옵션, 세 root 분기를 제거했다.
비교기는 항상 `--pdf-dirs pdf`를 받는다. 반대로 폐기 경로를 review-only로 허용하지 않았다.
Node 회귀 테스트는 `pdf-2020/**` 또는 `pdf-large/**`가 들어오면 full 정책으로 닫히는 것을
명시적으로 고정한다.

CI mirror source인 `scripts/ci-impact-classifier.cjs`, `scripts/ci-impact-policy.cjs`, trusted
post-merge 재사용기도 같은 경계로 맞췄다. workflow와 mirror가 서로 다른 경로 목록을 갖지 않는다.

## 3. Oracle Public 정본 재생성

`oracle_resolver.py`, `coverage_report.py`, `page_smoke.py`, `sweep_runner.py`,
`multiver_index.py`의 기본 root를 `("pdf",)`로 바꾸고 fixture·unit test도 같은 계약으로 갱신했다.

단일 경로 전환에서 가장 중요한 의미 변경은 디렉터리 기반 버전 추론 제거다. 과거에는 접미사가
없는 `pdf/foo.pdf`를 `2022`로 추론했지만, 이제 `pdf/` 안에 여러 한컴 버전이 함께 있으므로 그
추론은 거짓 provenance가 된다. 따라서 파일명에 2010·2018·2020·2022·2024 접미사가 있을 때만
`explicit`으로 기록하고 나머지는 `unknown`으로 둔다.

| 항목 | 재생성 결과 |
| --- | ---: |
| PDF 파일 | 1,178 |
| page count 측정 | 1,178 |
| 명시적 버전 | 1,068 |
| unknown 버전 | 110 |
| sample | 979 |
| 매칭 sample | 565 |
| oracle pair | 1,030 |
| unmatched sample | 414 |
| 다중버전 stem | 291 |
| page-count 불일치 stem | 7 |

정본 generator로 pair index, coverage, catalog, transcript, issue-draft 예시를 다시 만들었다.
더 이상 generator manifest에 속하지 않는 과거 draft 11개와 `by_root/pdf-2020.jsonl`,
`by_root/pdf-large.jsonl`은 제거했다. 현재 by-root 색인은 `pdf.jsonl` 하나다.

## 4. LLM verifier와 현행 소비자

LLM verifier의 producer 계약·fixture·generator를 단일 root로 바꾸고 16개 shard,
122,400 data row를 정본 generator로 재생성했다. 전수 verifier는 오류 0건이며 corpus 안에
`pdf-2020`·`pdf-large` 문자열이 남지 않았다.

현행 source/test의 sparse clone preset, layout advisory, issue #1052 Rust test 설명,
`samples/issue2006` 안내와 현재 기술 문서의 오라클 경로도 실제 이동 목적지로 갱신했다.
GitHub 운영, review-only fast-pass, 시각 증적, publish manual은 단일 경로·일반 blob·50 MiB
상한을 일관되게 설명한다. 과거 CHANGELOG·완료 계획·PR 기록은 수정하지 않았다.

## 5. 검증 결과

```text
Node ci-impact classifier/policy: 44 + 37 tests, OK
trusted post-merge reuse: 14 tests, OK
workflow Python contract 묶음: 124 tests, OK
Oracle Public unittest: 139 tests, OK
LLM verifier unittest: 42 tests, OK
LLM verifier corpus: 122,400 rows, errorCount=0
changed workflow YAML parse: 7 files, OK
PDF repository policy: OK (1178 PDFs, each < 52428800 bytes, no LFS pointers)
git lfs ls-files: 0 files
Markdown link check: 653 docs, internal relative links OK
git diff --check: OK
```

`actionlint` 실행 파일은 현재 환경에 없어 workflow 구조는 project mirror/unit test와 PyYAML parse로
검증했다. Rust 제품 코드는 바꾸지 않았지만 Rust integration test의 오라클 경로 주석 한 줄을
현행화했으므로, push 전 Stage 4에서 AGENTS.md의 전체 native/WASM/workspace lint 묶음을 실행한다.

## 6. 다음 단계 경계

메인테이너가 Stage 3 결과를 승인하면 Stage 4에서 다음을 수행한다.

1. LFS smudge 없이 clean clone/worktree를 만들어 1,178개가 실제 PDF인지 독립 확인한다.
2. Stage 1 원장의 19개 SHA-256과 새 Git blob을 전수 대조한다.
3. AGENTS.md의 Rust native·WASM·workspace lint/build 묶음을 순차 실행한다.
4. raw blob 증가량과 pack delta를 측정해 일반 clone 비용과 LFS bandwidth 제거 효과를 기록한다.
5. 최종 보고서를 작성하고 메인테이너 승인 뒤 commit·push·PR을 각각 별도 승인받는다.
