# Stage 2 — task_m100_4252 최신 devel 재배치·PR 후보 검증

- **이슈**: [#4252](https://github.com/edwardkim/rhwp/issues/4252)
- **브랜치**: `fix/issue-4252-nested-table-selection-path`
- **재배치 기준**: `upstream/devel` `e919655a78d5928cdf7236152fce04d6aa6f6377`
- **재배치된 Stage 1 commit**: `36bb0696931b197b6a0e8332b4df7e9d7ad251ff`
- **작업 시각**: 2026-08-08 KST
- **원격 상태**: branch push·PR 생성·이슈 comment/close 미수행

> 이 문서는 첫 번째 재배치 기준 `e919655a7`의 결과다. PR 게시 직전 발생한 두 번째 최신 devel
> 재배치와 최종 후보 검증은 [Stage 3 보고서](task_m100_4252_stage3.md)를 따른다.

## 1. 재배치 결과

Stage 1의 기준 `fcc3b2135` 이후 `upstream/devel`이 138개 commit 진행되어 PR 후보 전체 게이트 전에
rebase했다. `mydocs/orders/20260808.md` 한 파일의 충돌은 최신 devel 기록과 #4252 기록을 모두 보존해
해소했고, 나머지 Stage 1 변경은 자동 병합됐다.

최초 release build에서 upstream이 부분 표 본체를 `layout_partial_table_resolved()`로 분리하면서
Stage 1의 `enclosing_cell_ctx` 인자가 wrapper까지만 남은 것을 컴파일 오류로 확인했다. 새 함수 경계에
같은 `Option<&CellContext>`를 추가해 wrapper가 받은 실제 중첩 경로를 resolved 조판까지 전달했다.
geometry·페이지 컷·paint 분기는 변경하지 않았다.

## 2. 전체 PR 후보 게이트

| 검증 | 결과 |
| --- | --- |
| `CARGO_INCREMENTAL=0 cargo build --release` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --release --lib` | PASS, 3,330 passed·10 ignored |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | PASS, 전체 integration command exit 0 |
| #4252 / #2007 integration | PASS, 각각 5건 / 15건 |
| Native Skia `skia --lib` | PASS, 58건 |
| Native Skia #2225 / p37 direct PDF | PASS, 2건 / 4건 |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo fmt --check`, `git diff --check` | PASS |
| `CARGO_INCREMENTAL=0 cargo test --doc` | PASS, 4 passed·2 ignored |
| Studio `npx tsc --noEmit` | PASS |
| Studio `npm test` | PASS, 802/802 |
| E2E manifest | PASS, tracked 89개 / manifest 89행 |

Studio 전체 테스트를 sandbox 안에서 처음 실행했을 때 `spawnSync()` 기반 드라이버 5건이 결과 JSON
또는 성공 마커 없이 실패했다. 작은 진단에서 자식 Node 생성의 `EPERM`을 확인했고, 동일 `npm test`를
sandbox 밖에서 재실행해 802/802 통과했다. 코드 실패가 아니라 Codex sandbox 제약이며, 다음부터는
해당 전체 테스트를 처음부터 sandbox 밖에서 실행하도록 활성 Codex 메모리에 기록했다.

## 3. 최종 WASM과 브라우저 검증

표준 `docker compose --env-file .env.docker run --rm wasm` 빌드가 wasm-pack과 wasm-opt를 포함해
완료됐다.

- `pkg/rhwp_bg.wasm`: 7,684,236 bytes
- SHA-256: `23eeecee687a695427106415787a876ef88126f948519a0ca93f03d6ef0ff564`
- 수정 시각: 2026-08-08 21:02:01 KST
- 실행 중인 `http://127.0.0.1:7700` dev 서버 제공 해시: 로컬 파일과 일치

최종 WASM의 브라우저 검증 결과는 다음과 같다.

| 검증 | 결과 |
| --- | --- |
| `e2e:issue-4252` | PASS, 17쪽·3단계 실제 경로·외곽선 1·핸들 8·선택 렌더 1회·경고 0 |
| 물리 2쪽 bbox lookup | 중앙값 0.1ms, 5 cells |
| 물리 5쪽 자식 표 bbox lookup | 중앙값 0.4ms, 55 cells |
| 자식 표 선택 렌더 | 1.1ms, 1회 |
| `e2e:issue-4159` | PASS, 17쪽·물리 3쪽 종료선 1,309/1,316 픽셀 |
| #3137 입력 probe | PASS, operation p95 2.00ms·render p95 2.90ms·2rAF p95 28.00ms |
| #3137 구조 지표 | full repaint 0·long task 0·sync/input flush 0 |

증적은 `output/4252/`와 `output/4159/`에 있으며 `output/`은 gitignore 대상이다.

## 4. 작업지시자 시각 판정과 다음 승인 게이트

Stage 1의 작업지시자 시각 판정은 이전 기준 WASM
`21663c57767b3bca3a5ac53598568fa7f12184b7f20df1adb86114d700c25225`에 대한 결과다. 재배치 후
WASM 해시가 달라졌으므로 현재 dev 서버에서 #4252 자식 표 선택·부모 caret을 별도로 다시 확인했다.
작업지시자가 해당 최신 바이너리의 시각 판정을 통과시켰다.

로컬 PR 후보의 코드·전체 게이트·WASM·자동 및 수동 시각 검증은 완료됐다. 원격 branch push와
Open PR 생성은 아직 수행하지 않았으며 작업지시자의 별도 승인을 기다린다.
