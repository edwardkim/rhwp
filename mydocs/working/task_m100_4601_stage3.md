# Task M100 #4601 Stage 3 — v0.8.3 버전·릴리즈 문서 갱신

- 이슈: [#4601](https://github.com/edwardkim/rhwp/issues/4601)
- 기준: `upstream/devel@572786d02`
- 작성일: 2026-08-11 KST
- 상태: 로컬 릴리즈 후보 작성 완료

## 1. 버전 동기화

제품 표면을 `0.8.2`에서 `0.8.3`으로 올렸다.

| 구분 | 파일 |
| --- | --- |
| Rust | `Cargo.toml`, `Cargo.lock`의 root `rhwp` package |
| VS Code | `rhwp-vscode/package.json`, root lockfile 2곳 |
| Web editor·Studio | `npm/editor/package.json`, `rhwp-studio/package.json`, root lockfile 2곳 |
| Chrome·Edge | Chrome manifest/package, root lockfile 2곳 |
| Firefox | Firefox manifest/package, root lockfile 2곳 |
| Safari | Safari manifest |
| MCP registry 표지 | `server.json`의 server·OCI·PyPI·npm 버전 |

README 한국어·영어와 `ROADMAP.md`의 현재 버전, GitHub Action·install script 예시도
`v0.8.3`으로 맞췄다. `tools/set_package_version.py 0.8.3 --check`와
`cargo metadata --locked --no-deps`가 모두 `0.8.3` 일치를 확인했다.

Scoop·Homebrew·Winget·AUR의 `0.8.2`는 의도적으로 유지했다. 이 파일들은 아직 존재하지
않는 `v0.8.3` 자산의 SHA-256을 미리 지어낼 수 없으므로, 태그 후 `SHA256SUMS.txt`를
입력으로 하는 후속 PR에서 갱신한다.

## 2. 릴리즈 노트와 스토어 문서

- `CHANGELOG.md`, `CHANGELOG_EN.md`: 암호 문서, 조판·렌더링, 편집, CLI·MCP,
  보안·성능, 패키지·배포와 기여자 16명을 같은 구조로 기록했다.
- `rhwp-vscode/CHANGELOG.md`: outline 탐색과 v0.8.3 엔진 반영을 VS Code 사용자
  관점으로 분리했다.
- Chrome 한국어·영어 설명, Edge reviewer note, Firefox AMO reviewer note를
  `mydocs/feedback/*0.8.3*`로 새로 만들었다.

`git diff v0.8.2..upstream/devel --` 세 확장 manifest 결과가 0줄이고, Chrome·Firefox·
shared runtime 경로의 변경 파일도 package manifest와 lockfile뿐임을 확인했다. 따라서
권한·host permission·content script 선언은 v0.8.2와 동일하고 새 외부 endpoint도 없다는
사실을 reviewer note에 명시했다.

## 3. 라이선스 인벤토리

`cargo metadata --locked` 실측으로 `THIRD_PARTY_LICENSES.md`를 다시 대사했다.

- package entry 247개, 외부 source package 243개, workspace package 4개
- root 직접 의존성 고유 46개와 문서 표 46개가 정확히 일치
- AES/CBC/DES/HMAC/PBKDF2/SHA/Ed25519 등 암호화·서명 의존성을 새로 등재
- Rust·Studio·확장·VS Code의 기존 버전 드리프트를 현재 lockfile 값으로 정정

Volexity HWP5 암호 알고리즘 포팅의 BSD-3-Clause 전문과 기존 폰트·참조 프로젝트
고지는 유지했다.

## 4. 에이전트 계약과 생성물

새 release 바이너리를 빌드하고 자기서술을 실측했다.

| 항목 | 결과 |
| --- | --- |
| `rhwp --version` | `rhwp v0.8.3` |
| CLI / JSON 계약 | 83 / 52 |
| MCP 도구 | 무상태 66 + 세션 16 = 82 |
| IR schema | 1.0, 정의 41 |
| capabilities schema | 1.3, 정의 21 |
| plan schema | 1.1, 정의 11 |
| recordFields 합집합 | 261, 실측-only 3개 포함 사전 264 |

`bindings/node` 생성기를 이 바이너리로 실행해 `envelopes.ts`의 snapshot version을
`0.8.3`으로 갱신했고 `gen:check`를 통과했다. IR 구조와 52개 봉투의 필드 구성에는
드리프트가 없었다.

같은 실측을 `agent_knowledge_map.md`의 표면 규모·MCP 전수 지도·프로필 수치에 반영하고,
canonical `version_policy.md`에서 뒤처진 plan schema 1.0 표기를 1.1로 바로잡았다.

## 5. AMO 재현 소스

새 production `include_str!`와 workspace member 때문에 종전 source archive 명령은
현재 Cargo tree를 완전히 재현하지 못한다. `publish_guide.md`와 v0.8.3 AMO note의
명령에 다음을 추가했다.

- `Cargo.lock`, `llms.txt`, 에이전트 지식 지도·실패 사전·recipes
- `bindings/Native`, `tools/rhwp-subsecond`, `tools/batch-convert`
- production embedded resource인 logo, blank template, NotoSansKR font

필터 archive 구조 스모크는 ZIP 무결성 검사를 통과했고 크기는 약 37.2MB로 AMO 200MB
상한 이하다. 최종 커밋 기준 archive와 Firefox 실제 재빌드는 Stage 4에서 다시 검증한다.

## 6. 빌드 도구 audit 관찰

Node 바인딩 `npm audit`은 개발 전용 전이 의존성에서 2건을 보고했다.

- low: `tsup`/`vitest` 경유 esbuild 0.27.7의 Windows dev-server 로컬 파일 읽기
- high: `tsup` → postcss → nanoid 3.3.16의 size=0 custom generator 무한루프

`npm audit --omit=dev`는 취약점 0건이다. 두 경로는 배포되는 `@rhwp/node` runtime
dependency에 포함되지 않고 이번 코드가 해당 API를 호출하지 않는다. 릴리즈 준비에서
lockfile을 임의로 흔들거나 `npm audit fix`를 실행하지 않고, 후속 의존성 갱신 대상으로
분리한다.

## 7. 다음 단계

Stage 4에서 네이티브 전체 회귀, 공식 Docker WASM 빌드와 Studio 적용, 프론트·바인딩,
확장·패키징 gate를 실행한다. 이 단계가 끝나기 전에는 원격 push·PR·tag·publish를 하지
않는다.
