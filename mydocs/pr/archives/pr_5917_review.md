---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5917 self-review — HWP 2024 원격 MCP 클라이언트

## 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 세 자식 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `40851096496514429a95d2d0ed2d512df51d2a2e`

## 작성 시점 metadata

| 항목 | 값 |
| --- | --- |
| PR | [#5917](https://github.com/edwardkim/rhwp/pull/5917) |
| 작성자 | `@jangster77` |
| base / head | `devel` / `feat/hwp2024-mcp-client` |
| 상태 | Open, non-draft |
| 규모 | +345 / -29, 8 files, 3 commits |
| mergeability | `MERGEABLE`, `mergeStateStatus=CLEAN` — 2026-08-22 23:57 KST 참고값 |
| 관련 이슈 | 없음 — 작업지시자가 직접 요청한 배포·문서화 범위 |

## 변경 범위와 commit map

| commit | 역할 |
| --- | --- |
| `bd856bd43` | 의존성 없는 Node.js HWP 2024 원격 MCP 클라이언트 artifact 추가와 HWP 2020 artifact 이름 변경 |
| `df49e8d3e` | HWP 저장 버전별 2020/2024 서비스 선택, 인증 범위, 비동기 우선 사용법 문서화 |
| `408510964` | manual index와 PR 시각·fixture 가이드의 상호 참조 및 성공 조건 통일 |

- HWP 2024 client는 원격 MCP 서버 호출과 base64/blob 입출력만 담당하며 한컴 DLL·폰트·변환 엔진을
  포함하지 않는다. Node.js 22 내장 API만 사용하고 SDK, Zod 등 외부 runtime dependency가 없다.
- HWP 2022 이하 저장본은 `hwp-convert-2020`, HWP 2024 저장본은 `hwp-convert-2024`로 라우팅한다.
- 두 서비스 모두 긴 변환에 안전한 비동기 `start → status → download` 흐름을 우선 권장한다.
- MCP endpoint와 token은 공개 문서·artifact에 내장하지 않고 `.env.local`로만 주입한다. 접근 범위는
  maintainer·collaborator와 MCP 관리자가 인증한 사용자로 제한한다.
- 기존 HWP 2020 tarball은 내용 변경 없이 2022 이하 대상임을 드러내는 이름으로 이동하고 모든 추적 참조를
  갱신했다.

## artifact 검토

| 파일 | SHA-256 | 판정 |
| --- | --- | --- |
| `tools/hwp-convert-mcp-2022-client-20260805-071707.tar.gz` | `d25c101f4326cfb8148db2ff8a7096ab3f9b8c87a4b2f9849a3fc3de8e32fddc` | 기존 HWP 2020 client rename, `--help` 성공 |
| `tools/hwp-convert-mcp-2024-client-20260822-225818.tar.gz` | `55b75bb002818a42a18f0f289f3f8d669ac63af7185d2f6eece062ab326b120c` | 원격 HWP 2024 client, `--help` 성공 |

새 client의 동기 HWP→HWPX 변환은 성공했고 client/server 결과 크기와 SHA-256이 일치했다. 비동기
HWP→PDF도 `queued → succeeded → success` 상태 전이와 최종 결과의 크기·SHA-256 일치를 확인했다.
실측에 사용한 endpoint, token, 로컬 비공개 입력 경로는 저장소와 이 기록에 남기지 않는다.

## 문서와 시각 검증 판정

- 상대 Markdown 링크를 확인했고 JSON fenced example 11개를 모두 파싱했다.
- `mcp_hwp2020Convert_usage.md`와 `mcp_hwp2024Convert_usage.md`는 서로 다른 서비스 주소·artifact·도구명을
  유지하면서 선택 기준, 인증 범위, 비동기 우선 순서를 같은 구조로 설명한다.
- `visual_fixture_evidence.md`는 HWP 2020 기준 PDF와 HWP 2024 저장본 원격 변환의 성공 조건을 분리한다.
- 렌더러, golden, fixture와 제품 UI를 변경하지 않으므로 새 screenshot·시각 sweep은 적용하지 않는다.

## 완료한 검증

- 최신 `upstream/devel@b9eb551070f7e6b8ecf4272efebe1d2094fbeb9e` 기준 `0 behind / 3 ahead`와
  충돌 없는 rebase no-op을 확인했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare`와 `--check`를 통과했다.
- 별도 LF 검증 worktree에서 `cargo fmt --all`과 `cargo fmt --all -- --check`를 exit 0으로 통과했다.
  파생 integration suite와 검증 worktree는 정리했다.
- 두 tarball CLI의 `--help`, 변경 Markdown 상대 링크, JSON example 11개, `git diff --check`를 통과했다.
- Rust 제품 소스·테스트, WASM, Studio, renderer를 변경하지 않아 로컬 `cargo test`, clippy, WASM과 시각
  검증은 실행 대상에서 제외했다.

## GitHub Actions와 남은 조건

code candidate `408510964`에서 CI archive build/shard, Lint, Native Skia, frontend package,
Build & Test aggregate, CodeQL Rust·JavaScript/TypeScript·Python, Proptest roundtrip, Adapter inter-diff가
모두 성공했다. WASM Build와 frontend unit gate는 변경 범위 정책에 따라 정상 skip됐다.

- CI: <https://github.com/edwardkim/rhwp/actions/runs/32579476408>
- CodeQL: <https://github.com/edwardkim/rhwp/actions/runs/32579476285>
- Proptest: <https://github.com/edwardkim/rhwp/actions/runs/32579476307>
- Adapter inter-diff: <https://github.com/edwardkim/rhwp/actions/runs/32579476283>

이 review와 오늘할일만 담은 trailing commit을 push한 뒤 exact code candidate 재사용 여부, 최신 required
checks와 `mergeable=CLEAN`을 다시 확인한다.

## 위험과 후속

- 원격 endpoint 가용성, 인증서, token 권한과 서버 측 동시성은 client artifact 밖의 운영 조건이다.
- 파일 저장 버전이 불명확하면 2020/2024 서비스를 자동 판별하지 않는다. 문서의 저장 버전 확인 후 명시적으로
  선택해야 한다.
- 실제 변환 엔진과 폰트는 원격 Windows 서비스에 있으므로 client만으로 완전 offline 변환하지 않는다.
- 공개 artifact에 endpoint·token이 포함되지 않았고 Git diff에서도 비밀값을 발견하지 않았다.

## 권고

코드 self-review, 실제 동기·비동기 원격 변환, 로컬 계약과 code candidate GitHub CI에서 차단 결함을
발견하지 않아 **조건부 squash merge를 권고**한다. 최종 조건은 trailing 문서 commit을 포함한 최신 PR
head의 required checks 성공과 `mergeStateStatus=CLEAN` 재확인이다.
