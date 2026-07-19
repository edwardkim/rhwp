# Task M100 #2214 Stage 5 완료보고서 — 광역 게이트와 최종 결과 고정

## 0. 판정 요약

- **Stage 판정**: 완료
- **production 변경**: 없음. Stage 4 production source와 WASM 산출물을 그대로 검증
- **test 변경**: ignored matrix의 cold direct/path 44/50 대표값을 빠른 non-ignored GREEN으로 승격
- **최종 기준**: `upstream/devel@3c1cba96`, 병합 커밋 `d9da3b0b`
- **Rust 전체 게이트**: 3,075 passed / 0 failed / 23 ignored, lib 2,209 passed / 0 failed / 7 ignored
- **Studio 전체 게이트**: unit 214 passed, build·renderer contract 통과
- **브라우저 결과**: HWP/HWPX 각 3회 GREEN, stable flush 0·44번째 경계 flush 1
- **성능 결과**: stable operation p95 28.5~29.2ms, boundary operation 945.3~973.3ms
- **독립성 결과**: Stage 4 이후 구현 소스 9개와 WASM/JS/d.ts 산출물 3개 hash 동일
- **후속 범위**: boundary full pagination의 약 0.9초는 #2193 및 bounded/partial paginator 후속으로 유지

## 1. 최신 기준선 통합

Stage 5 도중 `upstream/devel`이 `4f9aaaff`에서 `3c1cba96`으로 5커밋 전진했다. 변경은
#2183 종료 보고·계획 보관과 `mydocs/orders/20260713.md` 갱신뿐이었다.

```text
d9da3b0b Merge remote-tracking branch 'upstream/devel' into issue-2214-page-local-repaint
```

오더 문서 충돌은 upstream의 #2183 완료, #2233·#2234 행과 현재 #2214 행을 모두 보존했다.
제품·테스트 source와 WASM 산출물에는 upstream 병합으로 인한 변화가 없었다.

## 2. Stage 4 산출물 동일성

`8efd562f`와 최종 검증 worktree의 production 경로를 `git diff --exit-code`로 대조했고 차이가
없었다.

### 2.1 production git blob

| 파일 | hash |
|------|------|
| `src/renderer/layout.rs` | `cd3d77efa1db9f68d3bd4a4be4a0da2fdc43a75c` |
| `src/renderer/layout/table_layout.rs` | `d5ae10da138c492ce4ba602f18bda9df29b201d3` |
| `src/document_core/commands/text_editing.rs` | `16c018169888ae1ffe47d1c137d5fa9ccc7609da` |
| `src/wasm_api.rs` | `8f5ec00036c47e2b5bc804e48b23036a3955fd72` |
| `rhwp-studio/src/core/wasm-bridge.ts` | `a05efff8f9b03e0219699d3be675c95a516302b9` |
| `rhwp-studio/src/engine/command.ts` | `5bce4dda4dfe221c0d22568ca42c3911e8ee2aa3` |
| `rhwp-studio/src/engine/history.ts` | `854af90c3e572ce22a04773d72660d5cb346459f` |
| `rhwp-studio/src/engine/input-handler.ts` | `c9c0f2541a6a135dcfb5877c71554703673925a1` |
| `rhwp-studio/src/engine/input-handler-text.ts` | `5ba4923fe13c2a403533ef9e2cf80c0fe551de9f` |

### 2.2 브라우저가 사용한 package SHA-256

| 산출물 | SHA-256 |
|--------|---------|
| `pkg/rhwp_bg.wasm` | `3e0f2432830acc6a829a24d73807582bdcafbc6610e7a4ec262437f05e8df8d9` |
| `pkg/rhwp.js` | `c0cf1254f922af0863f0c89bea0c47dd20dd07c1a212b4f5119b4fa6f3ae5a6f` |
| `pkg/rhwp.d.ts` | `72387ccea782846a00d56f2afa2e16e60957fd9fdf7efcdccd58f8998959b370` |

Stage 4 시작 시 고정한 값과 모두 같으므로 WASM 재빌드와 production 재수정 조건은 발생하지
않았다.

## 3. Rust 광역 게이트

| 검증 | 결과 |
|------|------|
| `cargo fmt --check` | 최종 재실행 통과, 2.67초, 파일 수정 없음 |
| `cargo clippy --all-targets --all-features -- -D warnings` | 최종 재실행 통과, 경고 0, 1.21초 |
| `cargo test --profile release-test --tests` | 3,075 passed / 0 failed / 23 ignored, 27.99초 |
| lib | 2,209 passed / 0 failed / 7 ignored, 6.20초 |
| `issue_2214_page_local_repaint` | 3/3 passed, cold 대표 test 포함, 4.88초 |
| `issue_2185_korean_break_unit` | 1/1 passed, 7.24초 |
| `issue_1949_giant_cell_render_perf` | 1/1 passed, 4.49초 |
| `issue_2063` | 1/1 passed, 1.56초 |

전체 테스트의 ignored 항목은 기존의 명시적 진단·장기 추적 항목이다. #2214의 필수 빠른
정확성 계약은 non-ignored 테스트로 실행됐고 intentional RED는 남지 않았다.

Clippy 최초 시도는 Skia 바이너리 조회 시 로컬 DNS 제한으로 중단됐지만, 네트워크 허용 뒤
동일 명령을 재실행해 통과했다. cold 대표 test 추가 뒤에도 fmt·Clippy·전체 test를 다시
실행해 같은 GREEN을 확인했다. 코드 경고나 컴파일 오류로 인한 실패가 아니었다.

## 4. 30-case 진단 매트릭스

다음 명령은 exit 0, 1 passed / 0 failed로 83.87초에 완료됐다.

```text
cargo test --profile release-test --test issue_2214_cache_matrix_probe -- --ignored --nocapture
```

HWP 15개와 HWPX 15개 관찰 레코드 모두 page count 115, tree exact=true였다. 28자 case는 cut
37→37, 44·50자 case는 cut 37→38이었다. 대표 warm tree query는 HWP 16.877ms, HWPX
18.286ms였고 every-edit warm tree는 각각 17.029ms, 16.731ms였다. full flush는 대부분
1.17~1.34초였으며 HWPX cold batch 28의 최대 관찰값은 1.816초였다.

이 테스트의 명시적 assertion은 초기 page count이며 case별 예상값을 아직 hard assertion으로
고정하지 않았다. 따라서 이 결과는 계획대로 **diagnostic completion**으로만 분류한다. page
count, exact tree/cursor, cut·bounds의 필수 correctness는 non-ignored #2214 native test와
브라우저 focused gate가 담당한다.

## 5. Studio·브라우저 광역 게이트

| 검증 | 결과 |
|------|------|
| `npm test` | 214 passed / 0 failed |
| `npm run build` | TypeScript + Vite production build 통과 |
| `npm run e2e:renderer-contract` | 통과 |
| focused HWP/HWPX 기본 runner | 각 3회, 6/6 GREEN |
| IME/iOS raw stable/boundary | 8/8 GREEN |
| Canvas crop | 43→44 변경 10,074 pixel, 경계 뒤 네 시점 diff 0 |
| `git diff --check` | 통과 |
| `git status --short` | Stage 5 커밋 전 계획된 test·보고서·오더만 변경 |

최종 focused 결과는 다음과 같다.

| 형식 | run | keyboard stable p95 | operation stable p95 | boundary operation | boundary flush |
|------|----:|--------------------:|---------------------:|-------------------:|---------------:|
| HWP | 1 | 48.3ms | 28.8ms | 973.3ms | 927.2ms |
| HWP | 2 | 47.9ms | 29.2ms | 948.9ms | 904.0ms |
| HWP | 3 | 47.2ms | 29.0ms | 954.5ms | 909.2ms |
| HWPX | 1 | 47.1ms | 28.5ms | 959.2ms | 913.7ms |
| HWPX | 2 | 47.9ms | 29.0ms | 952.0ms | 905.3ms |
| HWPX | 3 | 47.2ms | 29.2ms | 945.3ms | 898.9ms |

양 형식 6회 모두 1~43번째 stable 입력에서 flush 0, 44번째 flow 경계에서 mutation → flush →
cursor 순서의 flush 정확히 1회, 45~50번째 추가 flush 0이었다. 최종 text 180자, page count
115, 5줄, bounds 971.5, exact tree/cursor가 유지됐다.

각 형식의 IME/iOS raw stable 입력은 flush 0, flow 경계 입력은 flush 1이었다. desktop Chrome에서
iOS 분기를 강제한 로직 회귀이며 실제 iOS 기기 동작을 대체하지는 않는다.

## 6. 영구 GREEN 수용 계약 점검

- Stage 2의 warm cache RED는 non-ignored `issue_2214_page_local_repaint`와 crate-internal
  `issue2214` GREEN으로 이관됐다.
- 새 `issue_2214_cold_representative_queries_are_exact`는 HWP/HWPX의 cold 44자 direct와
  cold 50자 path query, exact tree/cursor와 115쪽을 non-ignored로 고정한다.
- 기존 warm 44자 path/direct와 50자 flow·focused browser 계약을 합쳐 cold/prewarm,
  direct/path, 44/50자 대표 범위를 빠른 GREEN으로 검증한다.
- 115쪽 structured cut chain과 cache-only/full-flush 분리는 non-ignored native assertion이다.
- 기본 브라우저 runner는 `--diagnose` 없이 HWP/HWPX 각 3회를 수행한다.
- `--diagnose`는 timeline·대조 PNG 수집용 optional 도구이며 최종 acceptance 조건이 아니다.
- #2214 필수 계약에는 expected RED나 필수 ignored test가 남지 않았다.

## 7. 남은 범위

- 실제 flow 경계의 full pagination은 정확성을 우선해 약 0.9초가 걸린다. #2193 종합 성능과
  bounded/partial paginator 설계 후속에서 대체한다.
- 실제 iOS contentEditable·가상 키보드·포커스는 기기 회귀가 필요하다.
- boundary flush 실패 시 30쪽 초과 문서의 retry/error UX는 별도 오류 복구 계약이 필요하다.
- #2215 드래그 selection은 별도 이슈다.

font metric, line-break semantic, parser/serializer, paginator와 Canvas production renderer는
Stage 5에서 변경하지 않았다. 이슈 close, push, PR 생성·통합도 수행하지 않았다.

## 8. Stage 5 tracked 산출물

- `tests/issue_2214_page_local_repaint.rs`
- `mydocs/working/task_m100_2214_stage5.md`
- `mydocs/report/task_m100_2214_report.md`
- `mydocs/orders/20260713.md`

위 test와 문서만 Stage 5 커밋에 포함하고, 검증용 `pkg/`와 `output/poc/task2214/`는 커밋하지
않는다.
