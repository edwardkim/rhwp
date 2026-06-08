# Task M100-1330 Stage 4 검증 보고서

## 대상

- GitHub Issue: #1330 `rhwp-studio: 빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐`
- 브랜치: `issue-1330-bullet-marker-caret-size`

## 자동 검증

실행 결과:

| 항목 | 명령 | 결과 |
| --- | --- | --- |
| Rust format | `cargo fmt --all -- --check` | 통과 |
| #1330 회귀 테스트 | `cargo test --test issue_1330_bullet_marker_caret_size` | 통과, 1 passed |
| Rust lib tests | `cargo test --lib` | 통과, 1613 passed / 6 ignored |
| Rust full tests | `cargo test` | 통과 |
| Rust clippy | `cargo clippy -- -D warnings` | 통과 |
| WASM build | `wasm-pack build --target web` | 통과 |
| rhwp-studio build | `npm run build` (`rhwp-studio`) | 통과 |

참고:

- 문서상의 Docker WASM 빌드 명령은 현재 로컬 Docker daemon이 실행 중이 아니어서 사용할 수 없었다.
- 대신 로컬 `wasm-pack 0.15.0`과 설치된 `wasm32-unknown-unknown` 타깃으로 동일한 `pkg/` 산출물을 갱신했다.
- `pkg/`와 `rhwp-studio/dist/`는 git ignored 산출물이므로 PR 포함 대상이 아니다.
- `cargo test` 전체도 통과해 기여자 체크리스트의 Rust test/clippy 요구를 충족했다.

## 로컬 dev server

실행 명령:

```bash
cd rhwp-studio
npm run dev -- --host 127.0.0.1 --port 7700
```

상태:

- URL: `http://127.0.0.1:7700/`
- listen: `127.0.0.1:7700`
- 프로세스: `node` dev server 실행 중

## 브라우저 smoke check

Codex in-app Browser에서 확인했다.

| 항목 | 결과 |
| --- | --- |
| Page identity | `http://127.0.0.1:7700/`, title `rhwp-studio` |
| Blank page | 아님. toolbar/menu DOM 렌더 확인 |
| Framework overlay | 없음 |
| Console error/warn | 0건 |
| Screenshot evidence | 첫 화면 toolbar 및 편집 영역 렌더 확인 |

## 수동 확인 권장 흐름

1. `http://127.0.0.1:7700/` 접속
2. 기존 재현 문서 또는 `rhwp-studio/public/samples/footnote-01.hwp` 계열 문서 로드
3. 글머리표 문단 끝에서 Enter
4. 빈 글머리표 줄 marker/caret 크기 확인
5. 같은 줄에 텍스트 입력
6. 입력 전후 marker/caret 크기가 튀지 않는지 확인

브라우저 캐시가 의심되면 hard reload를 수행한다. 이번 dev server는 루트 `pkg/`의 최신 WASM 산출물을 직접 참조한다.
