# PR #2205 리뷰 - 잘림 보기 기본값 및 영속화

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/2205 |
| 제목 | `rhwp-studio: 짤림보기 기본값을 켜짐(오버플로 표시)으로 변경` |
| 작성자 | `planet6897` |
| base | `devel` |
| head | `local/task2204` |
| 관련 이슈 | https://github.com/edwardkim/rhwp/issues/2204 |
| 규모 | 문서 작성 시점 참고값: 4 files, +92/-17 |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE` |
| maintainer modify | `true` |
| merge 결과 | `94befd4f969ef5d214343f154120bdf5c42aa71f`로 merge 완료 |
| 이슈 결과 | #2204는 GitHub Actions closing keyword 처리로 자동 close 완료 |

문서 작성 중 PR head가 `7c26d290cbad8592fd4a1cb398e2b978b371de04`로 갱신됐으며, 해당 트리는 최신
`upstream/devel` 위에 원 코드 커밋 2개를 체리픽한 로컬 검증 브랜치와 동일했다. 최신 head의 GitHub
Actions가 모두 통과한 뒤 2026-07-11에 merge했다.

## 변경 범위

- `rhwp-studio/src/command/commands/view.ts`
  - `clipEnabled`를 모듈 상태로 승격하고 저장된 `clipView`의 반대값으로 초기화한다.
  - `syncClipMenu`가 메뉴 active 상태와 내부 상태를 함께 동기화한다.
  - 토글 시 WASM 상태, 사용자 설정, 메뉴 상태를 같은 값에서 갱신한다.
- `rhwp-studio/src/core/user-settings.ts`
  - `ViewSettings.clipView`의 기본값, 기존 설정 정규화, 저장 setter를 추가한다.
- `rhwp-studio/src/main.ts`
  - 파일 로드와 새 문서가 공유하는 초기화 시퀀스에서 저장값을 WASM과 메뉴에 복원한다.
- `rhwp-studio/tests/user-settings.test.ts`
  - 기본값과 `localStorage` 저장을 검증한다.

Rust/WASM 구현과 문서 레이아웃 알고리즘은 변경하지 않는다. 기존 `setClipEnabled` 바인딩에 전달하는 기본값과
사용자 선택의 복원 시점만 바뀐다.

## Findings

blocking finding은 없다.

### P3. command/WASM 연동 자동 회귀 테스트 공백

추가된 테스트는 `clipView` 기본값과 설정 저장을 검증하지만 다음 연결 계약은 자동 테스트로 직접 고정하지 않는다.

- 메뉴 active와 `clipEnabled`의 역논리
- 토글 직후 WASM `PageLayerTree.buildOptions.clipEnabled` 반영
- 새로고침 또는 새 문서 초기화 후 저장값 복원

이번 리뷰에서는 브라우저 실동작으로 모두 확인했으므로 merge blocker로 보지 않는다. 이후 보기 설정 명령 테스트를
확장할 때 이 세 계약을 함께 고정하는 것이 좋다.

## 검증 결과

검증 트리는 PR 최신 head와 파일 단위 차이가 없다.

- `git diff --stat upstream/pr-2205 HEAD`
  - 차이 없음
- `npm test` (`rhwp-studio`)
  - 186 passed, 0 failed
- `npm run build` (`rhwp-studio`)
  - TypeScript 및 Vite production build 통과
- 최신 `devel` 위 원 코드 커밋 2개 체리픽
  - 충돌 없음
- GitHub Actions
  - 최신 head `7c26d290c` 기준 CI, CodeQL, Render Diff, Native Skia, Canvas visual diff 모두 통과
  - `Build default-feature tests`: 10m 20s 통과
  - Rust CodeQL 분석: 9m 08s 통과

## 브라우저 실동작 검증

`http://localhost:7700`의 Vite 앱에서 실제 문서를 로드한 상태로 확인했다.

| 동작 | 메뉴 active | PageLayerTree `clipEnabled` | 결과 |
|---|---:|---:|---|
| 잘림 보기 켜짐 | `true` | `false` | 오버플로 표시 계약 일치 |
| 잘림 보기 끔 | `false` | `true` | 용지 클립 계약 일치 |
| 끔 상태에서 새로고침 | `false` | `true` | 저장값 복원 통과 |
| 다시 켜짐으로 복원 | `true` | `false` | 기본 동작 복원 통과 |

메뉴 상태만 확인하지 않고 WASM이 생성한 PageLayerTree의 `buildOptions.clipEnabled`와
`outputOptions.clipEnabled`가 같은 값으로 바뀌는 것까지 확인했다.

## 시각 검증 판정

사용자 화면의 기본 clip 상태가 바뀌므로 렌더 영향은 있다. 다만 문서 geometry, pagination, paint 연산을 변경하는
PR이 아니며 기준 PDF와의 fidelity를 주장하지 않는다. 이번 변경의 핵심은 메뉴 상태와 기존 WASM clip option의
연결 계약이므로, 브라우저에서 실제 메뉴 토글과 PageLayerTree 출력 옵션을 대조한 동등 검증으로 판정했다.
별도 PDF visual sweep은 수행하지 않았다.

## PR 주장 검증

- `clipView=true`가 `clipEnabled=false`로 변환되는 역논리는 코드와 브라우저 결과가 일치한다.
- 파일 로드와 새 문서 공통 초기화 경로에서 설정을 복원한다는 설명은 `initializeDocument` 호출 구조와 일치한다.
- 토글 상태는 reload 뒤에도 유지된다.
- 순수 TypeScript 변경이며 WASM 재빌드가 필요 없다는 설명이 맞다.
- 관련 이슈의 요구사항인 기본 켜짐, 메뉴 active, WASM 상태 동기화를 충족한다.

## 최종 권고

PR 목적은 달성했고 blocking finding은 없었다. 최신 PR head의 GitHub Actions가 모두 통과한 뒤
`94befd4f969ef5d214343f154120bdf5c42aa71f`로 merge했다. command/WASM 연동 자동 테스트 공백은
비차단 후속 보완 후보로 남긴다.

## 후속 처리

이 문서는 옵션 2 후속 문서 PR #2208로 보존했다. 원 PR 본문의 `closes #2204`는 merge 직후 GitHub
Actions가 처리해 #2204를 자동 close했다. 원 PR 후속 코멘트에는 archive 리뷰 문서와 옵션 2 기록 PR 링크를
남긴다.
