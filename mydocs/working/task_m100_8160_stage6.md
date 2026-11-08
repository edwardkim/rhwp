# #8160 단계 6 — PR #8188의 VS Code WASM 소비 경로 보완

## 관측과 원인

로컬 검증 뒤 Open [PR #8188](https://github.com/edwardkim/rhwp/pull/8188)을 생성했다.
최초 제출 head는 `80153145479473545c678e2a55cf663565c729b5`다.
이 head의 [Frontend package gate](https://github.com/edwardkim/rhwp/actions/runs/40046356353/job/120729157912)는
Studio·브라우저 확장 빌드까지 통과한 뒤 `Compile VS Code extension`에서 실패했다.

새 공유 CanvasKit 모듈 `anisotropic-image.ts`·`metafile-print.ts`는
`@wasm/rhwp.js`를 사용한다. VS Code webview의 기존 TypeScript paths와 webpack alias는
`@rhwp-wasm`만 지원해 `TS2307`이 발생했다. 주 저장소의 `npm run typecheck`로 같은 실패를
재현했다. 앞선 로컬 검증에서 VS Code 확장 소비 경로를 빠뜨린 것이며 CI 실패를 외부 원인으로
돌리거나 성공한 다른 검사로 상쇄하지 않는다.

## 보완과 실제 검증

webview TypeScript와 webpack의 `@wasm`을 기존 `@rhwp-wasm`과 같은 `pkg`에 연결했다.
공통 renderer·WASM API·원본·PDF·회귀 기대값·실루엣 임계값은 바꾸지 않았다.

검증 전용 worktree에서 실제 source0c 표준 최적화 pkg를 연결해 실행했다.
[명령·종료 코드와 산출물 해시](../pr/assets/issue_8160/vscode-wasm-consumer-correction.json)에
다음 결과를 보존한다.

| 검사 | 결과 |
| --- | --- |
| VS Code `npm run compile` | TypeScript 2개 project와 실제 webpack host/webview 빌드 통과 |
| Studio / Chrome / Firefox 실제 build | 모두 exit 0; 각각 17.195 / 7.727 / 6.453초 |
| bindings·editor embed·extension dist·VS Code outline·font assets | 20/20 통과, 실패·skip 0 |
| 실제 VS Code webview bundle | Print sampling/replay API 네 이름 포함; unresolved alias 없이 번들 생성 |
| 배포 WASM | frozen pkg와 바이트 동일; SHA-256 `a814fb7a95f29f6d9de397f72a756ec19ce0fa2b7c151e00e879d58c80db06d4` |

배포물 검사 첫 시도는 Studio·Chrome·Firefox build 전이라 dist 부재로 4개가 실패했다.
모든 build를 완료한 뒤 정식 20개 묶음을 실행해 통과했다. 검사·기대값은 수정하지 않았다.
그 이전 VS Code compile의 독립 실패 로그와 보완 후 실제 로그 모두 private 증거에 보존한다.

이번 보완은 VS Code의 모듈 해석 설정뿐이다. 기존 제품 source0c에서 실제 산출한
Native/fresh WASM 384쪽 raw100 증거와 전체 Rust 검증의 생산자 SHA를 새 값으로 바꾸지 않는다.
보완 commit을 push한 뒤 정확한 새 head의 CI를 확인한다. 최초 head의 실패를 통과로 기록하지
않으며, code head CI 성공 전 review·오늘할일 trailing commit을 만들지 않는다.
