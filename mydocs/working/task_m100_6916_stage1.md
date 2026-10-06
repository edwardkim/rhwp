# #6916 Stage 1 — Gym 의존 계보와 제품 경계 조사

- Issue: [#6916](https://github.com/edwardkim/rhwp/issues/6916)
- 조사일: 2026-09-09
- 제품 기준: `c72ad805cc60e4a5cf5689c18b44e7214cec68fe`
- 작업 브랜치: `task_m100_6916`, 수행계획 승인 기록: `e82810a86`
- 상태: 조사 완료, 구현계획 승인 대기. 제품 source·workflow·테스트는 변경하지 않았다.

## 1. 결론

확인한 제품 Rust source의 직접 Gym 파일 포함은 `src/mcp_serve.rs:913`의
`include_str!("../gym/README.md")` 1곳이다. `src/main.rs:14`가 해당 모듈을 항상 포함하므로
MCP를 실행하지 않는 사용자도 CLI를 빌드할 때 Gym README가 필요하다.

이는 Gym 과제·채점 엔진이 제품 런타임에 들어간다는 뜻은 아니다. **선택적으로 읽는 문서를
제공하려고 필수 컴파일 입력을 추가한 설계 결합**이다. URI를 없애지 않고 제품 소유의 짧은 안내로
본문 공급원을 바꾸면, 설치본의 오프라인 문서 읽기를 유지하면서 이 결합을 제거할 수 있다.

전체 저장소의 개발 감사까지 Gym 부재에서 모두 통과한다는 목표와는 구분한다. 현재 일반 CI에는
Gym 파일·과제의 존재를 확인하는 조직·프레임 감사가 있고, 이번에는 같은 저장소의 Gym 자체를
제거하지 않으므로 유지한다. 이 잔여 소비 경로를 숨기고 “모든 Gym 관련 의존이 0”이라고 보고하지 않는다.

## 2. 왜 이렇게 되었는가

| 시점·근거 | 의도와 실제 효과 |
| --- | --- |
| 2026-08-13 `358a195793be5f680381ca50201b417024dbbd9f`, [PR #4721](https://github.com/edwardkim/rhwp/pull/4721) | roadmap·Gym을 MCP 리소스로 공개했다. 원 PR은 단일 설치본에서도 문서를 읽게 하려고 `include_str!`를 사용했다고 설명한다. 읽기 opt-in과 빌드 선택성을 구분하지 않은 지점이다. |
| 2026-09-02 `b6e5b54008` | Gym workflow를 별도 벤치마크로 분리하고 CI classifier의 Gym-only 제품 worker 생략을 도입했다. 컴파일 입력 결합은 남았다. |
| 2026-09-05 `b1e6accb74072fe5e9490748b41839a9dac86efc`, #6689 | Oracle 부분 checkout에 누락된 제품 include 입력을 추가하면서 `gym/README.md`도 넣었다. 당시 동작을 복구한 변경이며 이번 경계 결정과 목적이 다르다. |

확인 명령은 `git log -S '../gym/README.md' -- src/mcp_serve.rs`, `git show 358a195793`,
`git log -S 'gym/README.md' -- .github/workflows/oracle-public-advisory.yml`, `git show b1e6accb7`,
`gh pr view 4721 --repo edwardkim/rhwp --json title,body,state,url`이다.

판정: 최근 변경으로 생긴 MCP 실행 회귀가 아니라 **도입 시점부터 존재한 빌드 결합과 독립성 검증 부재**다.
이전 sparse build 실패 증적은 [#6689 Stage 3](task_m100_6689_stage3.md)의 원인 계보에 이미 있다.
이번 Stage 1에서 동일한 무거운 실패 빌드를 반복하지 않았다. Gym 없는 후보의 실제 성공 증명은 Stage 3이다.

## 3. 소비 지점과 처리 결정

| 분류 | 실제 소비자 | 권고 |
| --- | --- | --- |
| 제품 필수 빌드 입력 | `src/mcp_serve.rs` → `gym/README.md` | 제품 소유의 짧은 안내로 교체 |
| 빌드 결합의 운영 파생물 | `.github/workflows/oracle-public-advisory.yml:83`, `scripts/tests/test_oracle_public_advisory_workflow.py` | sparse 입력과 그 계약을 함께 교체. 다른 필수 문서는 보존 |
| 일반 제품 기능 | `explore`, scaffold schema, GPU raster의 `gym` 관련 주석 | 직접 Gym 실행/파일 의존과 다르다. 기능과 feature 보존 |
| 선택적 평가 실행 | `gym/**`, `tools/agent_dispatch.py`의 `gym/packs` 조회·채점 안내 | 평가 도구 측 소비이므로 보존 |
| 전체 저장소 구조 감사 | `tools/frame_guard.py`, `mydocs/tech/agent_frame/frame.json`, `test_agent_frame.py`, `test_agent_org.py` | Gym 파일·과제 존재 검사는 남긴다. 제품 build/runtime/package와 구분 |
| Gym 자체 CI | `gym-release-gate.yml`의 Gym 관련 PR contracts, 수동 full | 유지. 일반 push/tag 트리거 없음. 채점 결과의 제품 gate화 금지 |
| 승격 시 workflow 검증 | `workflow_promotion_policy.json`의 Gym contracts-only 항목 | Gym workflow 자체가 승격 대상일 때 실행 배선 검증. full benchmark는 skipped여야 한다. 정책 변경 없음 |
| 일반 CI 영향 분류 | `ci-impact-classifier.cjs::isGymBenchmarkPath` | Gym-only에서 제품/CodeQL worker를 요구하지 않는 정책 유지 |
| 선택적 문서 링크·개발 안내 | agent knowledge map, `tools/sparse_clone_hint.py`의 Gym 역할, Gym 운영 문서 | 링크는 제품 파일 읽기 의존이 아니다. 보존하고 경계만 명시 |
| 과거 우회 설명 | `mydocs/tech/render_backend.md`의 sparse Gym 추가 안내 | 과거 증거는 남기되 #6916 이후 현재 처방과 구분 |

조사 범위: `src`, `crates`, `bindings`, root Cargo/build 설정, tracked 도구·scripts,
workflow, npm manifest, 확장 build 스크립트. `gym` 일치 항목을 실제 파일 열기·include·실행·주석으로
분류했다. 문자열 검색은 동적으로 구성하는 모든 의존이 없다는 완전한 증명이 아니므로 격리 빌드를 추가한다.

## 4. MCP 호환성

`served_resources()`는 DOC_RESOURCES에서 URI/name/title/description/mimeType/size를 만들고,
`read_resource()`는 같은 항목의 text를 반환한다. Gym 전용 handler는 없다.
기존 `tests/mcp_resources_contract.rs`는 모든 광고 URI의 읽기·MIME·비어 있지 않은 본문을 검사한다.
Gym URI 자체의 존속이나 기존 README 전체와 byte-identical한 본문은 고정하지 않는다.

| 대안 | 효과·비용 | 판단 |
| --- | --- | --- |
| A. 제품 소유의 선택적 안내를 내장하고 URI 유지 | Gym 없는 빌드·오프라인 읽기. 본문은 전체 README에서 짧은 안내로 변경 | 권고 |
| B. Gym URI 삭제 | 결합 제거는 간단하나 기존 클라이언트가 `-32002`를 받음 | 불필요한 공개 표면 단절 |
| C. feature/파일 존재/네트워크에 따라 전체 README 제공 | 빌드·설치별 리소스 차이와 fallback/실패 경로 증가 | 이번 최소 분리에 불필요 |

A에서도 본문·size는 바뀐다. 이를 “응답 전체가 동일”하다고 표현하지 않는다.
URI `rhwp://docs/gym`, name `gym-readme`, title, MIME, 목록→읽기와 JSON-RPC 봉투는 유지한다.
description은 선택적 도구 안내라는 실제 내용에 맞게 고친다. 안내 읽기에서 HTTP 요청·Gym 탐색·채점 실행은 하지 않는다.
외부 클라이언트가 README 문장을 파싱하는지는 확인하지 못했다. 본문 축소는 명시적으로 승인받을 변경이다.

## 5. 배포 영향

| 경로 | source 근거와 영향 | 후속 검증 |
| --- | --- | --- |
| 네이티브 CLI | release-binary.yml은 `cargo build --release --bin rhwp` 후 binary/LICENSE/README/README_EN을 포장 | Gym 없는 Linux release build, 설치 디렉터리에서 CLI/MCP 실행, 동일 포함물 포장 |
| WASM·@rhwp/core | lib 경로이며 MCP bin 모듈은 포함하지 않는다. Docker locked wasm-pack과 prepare-npm.sh의 files 목록 사용 | Gym 없는 소스로 Docker WASM 1회, 생성 pkg의 `npm pack --dry-run --json` |
| @rhwp/editor | 명시적 files 목록, JS/type/transport/문서. Gym 참조 없음 | source manifest 확인. 실행 코드 미변경이므로 별도 전체 frontend 재빌드 생략 권고 |
| VS Code/VSX | npm-publish.yml이 VS Code 디렉터리에서 빌드·vsce 포장, pkg WASM을 media로 복사 | 이번 MCP bin 수정이 확장 bundle에 미도달함을 확인. 전체 VSIX 재빌드·게시 생략 권고 |
| Chrome/Edge/Firefox | 각 build.mjs가 Studio 빌드와 확장/WASM 자산 복사 | 직접 Gym 입력 없음. UI·엔진 미변경이므로 스토어 패키지 전체 재빌드·게시 생략 권고 |

위는 로컬 source 경로 조사 결과다. 게시된 모든 플랫폼 패키지를 다운로드해 검사한 결과가 아니다.
Windows/macOS 네이티브 실행은 로컬 Linux 증적으로 대체해 통과 처리하지 않는다.

## 6. 이번 단계 검증

제품 source 미변경 상태에서 기존 경계의 출발점을 확인했다.

| 명령 | 결과 |
| --- | --- |
| `python3 -m unittest scripts.tests.test_oracle_public_advisory_workflow scripts.tests.test_gym_benchmark_validation scripts.tests.test_agent_frame` | 16 passed, exit 0 |
| `python3 -m unittest scripts.tests.test_agent_org` | 8 passed, exit 0 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 44 passed, exit 0. Gym-only 제품 worker 생략 사례 포함 |

Rust build·MCP runtime·Docker WASM·Gym canary는 이번 단계에서 실행하지 않았다.
이 테스트 성공은 기존 계약의 출발점일 뿐 제품 독립성 완료 증거가 아니다.

## 7. 다음 승인 요청

[구현계획](../plans/task_m100_6916_impl.md)의 A안과 Stage 2를 요청한다.
Gym README의 내장 본문 축소, Oracle sparse 입력 교체, 기존 계약 보존 테스트와 권위 문서 경계 보완에 한정한다.
원격 push·workflow 실행·PR·merge는 수행하지 않았다.
