---
kind: report
status: final
canonical: mydocs/report/task_m100_6695_report.md
issue: 6695
last_verified: 2026-09-03
---

# #6695 PDF 오라클 단일 경로·Git LFS 폐지 최종 보고

## 1. 최종 판정

**계획한 Stage 1~4 구현과 로컬 검증을 완료했다.** PR #6696 생성 뒤 self-review에서 수동
Oracle Public Advisory의 embedded Python gate가 `os.environ`을 사용하면서 `import os`를 빠뜨린
오류를 발견했다. import를 복원하고 이 본문을 tiny 저장소에서 직접 실행하는 CI 계약 테스트를
추가해 로컬 재검증을 마쳤다. 정정 candidate의 원격 push와 최신 head CI는 별도 게이트다.

- 한컴 PDF 오라클은 `pdf/**` 1,178개, 1,079,000,023 bytes로 단일화됐다.
- `pdf-2010/`, `pdf-2020/`, `pdf-large/` 최상위 경로와 현재 tree의 LFS pointer는 0개다.
- 19개 전환 원장의 크기·SHA-256·`%PDF-` magic은 이동 전후와 독립 clone에서 모두 일치했다.
- PDF 파일은 모두 50 MiB 미만이며 최대 파일은 50,228,784 bytes다.
- workflow, CI fast-pass, Oracle Public, LLM verifier, 현행 운영 문서는 `pdf/**`만 활성 입력으로
  사용한다.
- 폐기 경로가 다시 나타나면 review-only로 오인하지 않고 full/fail-closed 정책으로 처리한다.
- 최신 `upstream/devel@b6b9384ed`를 일반 merge해 최종 검증 기준에 포함했다.

## 2. 자산과 provenance 보존

Stage 1에서 `pdf-2020/**` 1개와 `pdf-large/**` 18개의 실제 bytes를 원장으로 동결했다. LFS pointer
17개는 선언 OID·선언 크기와 로컬 object store의 실제 PDF를 대조했고, 일반 blob 2개도 Git index
bytes로 검산했다.

Stage 2에서는 hash가 같은 기존 `pdf/` 목적지 2개를 그대로 보존하고 source만 제거했다. 나머지
17개 목적지를 상대 구조를 유지해 만들었다. 이 중 15개는 기준선 Git history에 없던 실제 PDF
blob이고, 2개는 기존 일반 blob의 경로 이동이다. 따라서 공개 history rewrite나 force-push 없이
현재 tree만 정상화했다.

정본 규약은 다음과 같다.

1. 버전·크기와 무관하게 한컴 PDF 오라클은 `pdf/**`에 둔다.
2. 한컴 버전·입력 형식·폰트 조건은 파일명 suffix와 생성 증적으로 보존한다.
3. 디렉터리 이름만으로 버전을 추론하지 않는다.
4. PDF는 일반 Git blob이며 파일 하나는 50 MiB 미만이다.
5. 상한 초과 자료는 축소 fixture·페이지 발췌·외부 증적 방식을 먼저 합의한다.

과거 Git LFS 객체 17개, 57,128,473 bytes가 원격 저장소에서 자동 삭제된다는 주장은 하지 않는다.
원격 LFS purge는 GitHub Support와 별도 운영 판단이 필요한 비범위다.

## 3. 활성 소비자 정상화

Stage 3에서 다음 계층을 함께 바꿔 source와 생성물의 계약 불일치를 막았다.

- CI, CodeQL, Pages, Render Diff, Adapter Diff, Proptest Roundtrip workflow
- `ci-impact-classifier`, `ci-impact-policy`, trusted post-merge 재사용기와 mirror test
- Oracle Public resolver·coverage·page smoke·sweep·multiver source, fixture, catalog, report,
  transcript, issue-draft 예시
- LLM verifier producer 계약·fixture·16개 corpus shard
- sparse clone, layout advisory, Rust test 설명과 현행 GitHub·review·publish manual

Oracle Public 정본 재생성 결과는 sample 979개, 매칭 sample 565개, oracle pair 1,030개,
unmatched 414개다. PDF 1,178개는 전부 page count 측정에 성공했다. 파일명에서 버전을 명시적으로
탐지한 것은 1,068개이며, 접미사가 없는 110개는 `unknown`이다. 다중버전 stem은 291개이고 그중
page-count 불일치는 7개다.

LLM verifier corpus는 단일 `pdf` root로 122,400행을 재생성했다. 전수 verifier 오류는 0건이며
corpus에 폐기 경로 문자열이 남지 않았다. historical CHANGELOG·완료 계획·PR 기록은 당시 사실을
보존하고, memory 문서에만 현행 `pdf/README.md`로 연결되는 superseded 안내를 추가했다.

## 4. 독립 clone 검증

`GIT_LFS_SKIP_SMUDGE=1`과 local transport의 `--no-local --single-branch`를 사용해 별도 clone을
만들고 merge commit `65cf61abb`를 checkout했다. 이 clone에는 `.git/lfs/objects` 디렉터리가
없었다.

| 검사 | 결과 |
| --- | ---: |
| `pdf/**` 파일 | 1,178 |
| PDF bytes | 1,079,000,023 |
| `%PDF-` magic 정상 | 1,178 |
| LFS pointer | 0 |
| Stage 1 원장 SHA-256 | 19/19 일치 |
| 대표 경로 `filter` attribute | 모두 `unspecified` |
| clean clone worktree 변경 | 0 |

따라서 현재 PDF를 받기 위해 Git LFS smudge나 소유자 LFS bandwidth가 필요하지 않다.

## 5. 저장량 영향

측정 기준은 최신 `upstream/devel@b6b9384ed`와 merge 직후 task head `65cf61abb`다. 최종 보고서
자체의 작은 Markdown blob은 아래 수치에 포함하지 않았다.

| 항목 | 수치 |
| --- | ---: |
| 새 PDF 경로 | 17개 / 55,214,491 bytes |
| 기준선에 없던 고유 PDF blob | 15개 / 54,182,978 bytes |
| task의 새 blob 전체 | 151개 / 110,786,646 raw bytes |
| task의 새 tree | 70개 / 959,555 raw bytes |
| task의 새 commit | 5개 / 1,407 raw bytes |
| `git pack-objects` 증분 pack | 32,097,140 bytes |
| 독립 single-branch clone 전체 pack 참고값 | 2,049,373,496 bytes |

증분 pack은 Git의 압축·delta 결과라 raw PDF 합보다 작다. GitHub 서버가 만드는 실제 wire pack은
서버 버전·bitmap·delta 선택에 따라 달라질 수 있으므로 32,097,140 bytes를 GitHub 전송량의
불변값으로 주장하지 않는다. 확인된 교환관계는 명확하다.

- 일반 clone은 이 task의 압축 object를 추가로 받는다.
- 대신 현재 PDF를 받는 clone·fork·Actions가 별도 Git LFS object 17개를 요청하지 않는다.
- 과거 LFS storage와 이미 소비된 quota는 이 commit만으로 회수되지 않는다.

## 6. 최종 검증

최신 `devel` 병합 뒤 다음을 다시 실행했다.

```text
PDF repository policy: 7 tests + live tree, OK
CI impact classifier/policy: 44 + 37 tests, OK
trusted post-merge reuse: 18 tests, OK
workflow Python contract bundle: 179 tests, OK
Oracle Public Advisory embedded gate: 실제 실행 포함 3 tests, OK
workflow YAML parse: 21 files, OK
Oracle Public unittest: 139 tests, OK
LLM verifier unittest: 42 tests, OK
LLM verifier corpus: 122,400 rows, errorCount=0
Markdown link check: 655 docs, internal relative links OK
git diff --check: OK

node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
모두 성공
```

PDF bytes 자체를 재생성하지 않고 검산된 bytes를 경로 이동했으므로 별도 시각 판정은 요구하지
않았다. 이 작업의 회귀 기준은 시각 유사도가 아니라 byte-identical SHA-256 보존이다.

## 7. 완료 조건과 후속 경계

Issue #6695의 로컬 구현·검증 완료 조건은 충족됐다. PR #6696에서 남은 외부 절차는 다음과 같다.

1. push 직전 `upstream/devel`과 exact head·충돌 여부를 다시 확인한다.
2. 메인테이너 별도 승인 뒤 정정 candidate를 원격 branch에 push한다.
3. 새 Full CI 성공 뒤 self-review·오늘할일 trailing commit을 같은 PR에 포함한다.
4. 최신 trailing head CI 성공과 merge 승인을 받아 정상 merge commit으로 병합한다.
5. merge SHA의 `devel` CI를 확인한 뒤에만 Issue #6695 close와 작업 브랜치·임시 자산 정리를 수행한다.

원격 LFS storage purge는 Issue #6695 완료 조건이 아니며, 필요하면 비용·복구 위험을 별도로 검토한다.
