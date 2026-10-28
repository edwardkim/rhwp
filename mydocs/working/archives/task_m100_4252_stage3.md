# Stage 3 — task_m100_4252 게시 직전 최신 devel 재배치

- **이슈**: [#4252](https://github.com/edwardkim/rhwp/issues/4252)
- **브랜치**: `fix/issue-4252-nested-table-selection-path`
- **최종 재배치 기준**: `upstream/devel` `59b31e5ce33ab61c4a907e715803304437174af9`
- **작업 시각**: 2026-08-08 KST
- **원격 상태**: branch push·PR 생성·이슈 comment/close 미수행

## 1. 두 번째 재배치와 최신 호출부 적응

원격 push·Open PR 생성 승인을 받은 직후 기준 브랜치를 다시 fetch한 결과, Stage 2 기준
`e919655a7`에서 30개 commit 전진해 있었다. #4031·#4138·#4167·#4179·#4180·#4248 등
cursor·부분 표·Studio 입력 경로와 겹치는 변경이 포함되어 있어 그대로 게시하지 않고 로컬 3개
commit을 최신 기준 위로 다시 rebase했다. 텍스트 충돌은 없었다.

컴파일 래칫에서 #4248이 추가한 cursor probe의 `layout_partial_table()` 호출이 #4252의 새
`enclosing_cell_ctx` 인자를 전달하지 않은 것을 확인했다. 이 probe는 실제 본문 최상위 표를
`parent_para_idx/control_idx`로 직접 조판하므로 enclosing context는 `None`이다. 해당 인자를
명시해 함수 계약을 맞췄으며 geometry·probe 범위·캐시 정책은 변경하지 않았다.

## 2. 최종 기준 전체 게이트

| 검증 | 결과 |
| --- | --- |
| #4252 / #2007 focused integration | PASS, 5건 / 15건 |
| `cargo test --release --lib` | PASS, 3,348 passed·13 ignored |
| `cargo test --profile release-test --tests` | PASS, 전체 integration command exit 0 |
| Native Skia `skia --lib` | PASS, 58건 |
| Native Skia #2225 / p37 direct PDF | PASS, 2건 / 4건 |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --doc` | PASS, 4 passed·2 ignored |
| Studio `npx tsc --noEmit` | PASS, 새 WASM 바인딩 생성 후 |
| Studio `npm test` | PASS, 813/813; 가드레일에 따라 처음부터 sandbox 밖 실행 |
| E2E manifest | PASS, tracked 90개 / manifest 90행 |

WASM 재생성 전 TypeScript 검사는 최신 devel의 `setCaretPosition` API가 이전 `pkg` 선언에 없어
실패했다. 표준 Docker 빌드로 Rust API와 JS/TypeScript 바인딩을 함께 재생성한 뒤 동일 검사가
통과했으므로 소스 결함이 아닌 산출물 순서 문제로 판정했다.

## 3. 최종 WASM과 브라우저 검증

- 표준 빌드: `docker compose --env-file .env.docker run --rm wasm`
- `pkg/rhwp_bg.wasm`: 7,701,996 bytes
- SHA-256: `328457d78e7af88ced916a63811f8ad76cca12c6762777ac6aca6f69d3dadd5b`
- 수정 시각: 2026-08-08 21:46:32 KST
- 실행 중인 dev 서버 제공 해시: 로컬 산출물과 일치

| 검증 | 결과 |
| --- | --- |
| `e2e:issue-4252` | PASS, 17쪽·3단계 경로·외곽선 1·핸들 8·렌더 1회·경고 0 |
| 물리 2쪽 / 5쪽 bbox lookup | 중앙값 0.1ms / 0.4ms, 5 / 55 cells |
| 자식 표 선택 렌더 | 1.5ms, 1회 |
| `e2e:issue-4159` | PASS, 17쪽·종료선 1,309/1,316 픽셀 |
| #3137 입력 probe | PASS, operation p95 1.80ms·cursor 0.10ms·render 2.80ms·2rAF 22.40ms |
| #3137 구조 지표 | full repaint 0·long task 0·sync/input flush 0 |

증적은 `output/4252/`, `output/4159/`에 있으며 `output/`은 gitignore 대상이다.

## 4. 작업지시자 시각 판정과 게시 승인

Stage 2의 작업지시자 시각 판정은 이전 WASM
`23eeecee687a695427106415787a876ef88126f948519a0ca93f03d6ef0ff564`에 대한 결과다. 두 번째
재배치로 최종 WASM 해시가 달라졌으므로 현재 dev 서버에서 #4252 자식 표 선택·부모 caret을 다시
시각 확인했다. 작업지시자가 최종 바이너리의 판정을 통과시켰으며, 이미 받은 승인 범위에 따라 원격
branch push와 `devel` 대상 Open PR을 생성한다.
