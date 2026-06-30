# PR #1675 사전 처리 판단 보고서 - HWPX package graph 직렬화 복원

- 작성일: 2026-06-30
- PR: https://github.com/edwardkim/rhwp/pull/1675
- 제목: `[Ghidra/Frida 근거] HWPX 패키지 그래프 직렬화 복원`
- 컨트리뷰터: [@humdrum00001010](https://github.com/humdrum00001010)
- base/head: `devel` <- `humdrum00001010:codex/pr1573-package-graph`
- 문서 작성 시점 head: `abc6a88d8cb648d6be14f54ccae8c0472e08f8ae`
- 처리 경로: Route A - 원 PR merge 후보, collaborator-mediated PR head 문서 포함
- 연결 이슈: 없음

## 1. 판단

**merge 수용 권고.**

PR #1675는 HWPX serializer의 package graph 산출을 보강해 한컴 호환 리더가 기대하는
`container.rdf`, `content.hpf`, master page part, section `idRef`, header/footer id 관계를 복원한다.
변경 범위는 HWPX 직렬화와 package checker에 한정되어 있고, 로컬 HWPX serializer 테스트와 samples/hwpx
roundtrip baseline을 통과했다.

이 보고서는 merge 전 사전 판단 보고서다. 실제 merge 여부, merge SHA, merge 시각, issue close 결과는
아직 확정하지 않는다.

## 2. 원인과 수정 요약

### 원인

- 기존 `META-INF/container.rdf`는 정적 상수라 `Contents/section0.xml`만 참조했다. multi-section HWPX를
  재직렬화하면 ZIP과 `content.hpf`에는 여러 section이 있어도 RDF package graph가 stale해질 수 있었다.
- IR에 보존된 master page가 ZIP package part로 쓰이지 않으면 `content.hpf` manifest와 section XML의
  `hp:masterPage idRef` 축이 완성되지 않는다.
- header/footer control의 HWPX `id`가 `0`으로 재작성되어 원본 id/idRef 관계를 잃을 수 있었다.

### 수정

- `src/serializer/hwpx/mod.rs`
  - 실제 section href 목록으로 `container.rdf`를 동적 생성
  - 모든 master page를 `Contents/masterpage{N}.xml`로 직렬화
  - `content.hpf` writer에 master page manifest 항목 전달
- `src/serializer/hwpx/package_check.rs`
  - `container.rdf` header/section coverage 검증 추가
  - master page ZIP entry, manifest, section `masterPageCnt`/`idRef` 검증 추가
- `src/serializer/hwpx/section.rs`
  - header/footer id를 `raw_ctrl_extra`에서 복원

## 3. 출처와 contributor credit

원 PR: #1675

source commit:

| source SHA | commit | author |
|---|---|---|
| `264e2abdbecab1efde3442024ddc4f571c015e7b` | `fix: restore HWPX package graph for Hancom` | @humdrum00001010 / phi hu |
| `abc6a88d8cb648d6be14f54ccae8c0472e08f8ae` | `Merge branch 'devel' into codex/pr1573-package-graph` | @humdrum00001010 |

Route A이므로 cherry-pick integration mapping은 없다. contributor branch의 기존 commit을 rewrite하지 않고,
review 문서만 별도 docs commit으로 PR head에 추가하는 계획이다.

## 4. 검증 결과

로컬 검증 기준: `local/pr1675`, head `abc6a88d8cb648d6be14f54ccae8c0472e08f8ae`

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo test --release --lib serializer::hwpx::` | 통과, 270 passed / 1 ignored |
| `cargo test --release --test hwpx_roundtrip_baseline` | 통과, 4 passed |

GitHub checks는 문서 작성 시점 PR head 기준으로 다음 상태였다.

| check | 문서 작성 시점 상태 |
|---|---|
| CI preflight | success |
| Build & Test | success |
| CodeQL preflight | success |
| Analyze (javascript-typescript) | success |
| Analyze (python) | success |
| Analyze (rust) | success |
| CodeQL | success |
| WASM Build | skipped |

문서 commit push 후에는 최신 PR head 기준으로 필요한 GitHub checks를 다시 확인해야 한다.

## 5. 시각 검증

시각 검증 산출물은 없음. 이 PR은 렌더링 결과나 golden을 변경하지 않고 HWPX package graph와 roundtrip
구조 보존을 수정한다. 검증 기준은 `package_check`, `hwpx_roundtrip_baseline`, serializer 단위 테스트다.

## 6. PR head 문서 push 계획

작업지시자 승인 후 아래 문서만 PR head에 추가한다.

- `mydocs/pr/archives/pr_1675_review.md`
- `mydocs/pr/archives/pr_1675_report.md`

예정 commit:

```text
docs: PR #1675 검토 기록
```

예정 push:

```text
git push https://github.com/humdrum00001010/rhwp.git HEAD:codex/pr1573-package-graph
```

## 7. merge 전 조건

- docs commit push 후 최신 PR head SHA 확인
- 최신 `mergeable` / `mergeStateStatus` 확인
- 최신 PR head 기준 관련 GitHub checks 확인
- PR diff에 review/report 문서 포함 확인
- GitHub review approval 또는 동등한 검토 의견 등록
- 작업지시자 merge 승인

## 8. issue close 계획

문서 작성 시점 PR metadata 기준 linked closing issue는 없다. merge 후에도 연결 이슈가 없는지 확인하고,
새로 연결된 이슈가 없다면 수동 close 작업은 수행하지 않는다. 이슈 close가 필요해지는 경우에는 별도
작업지시자 승인 후 처리한다.
