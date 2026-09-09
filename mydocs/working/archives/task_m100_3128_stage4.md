# #3128 Stage 4 — 최신 devel 전체 검증

- **Issue**: #3128
- **기록일**: 2026-08-18 KST
- **기준**: `upstream/devel@0bc05ef81`
- **상태**: 완료, commit·push·PR 게시 승인 완료

## 1. 최신 devel 재기준화

로컬 변경과 새 문서를 `git stash --include-untracked`로 보존하고 `upstream/devel`을 fetch했다. 기존
branch는 자체 commit이 0개이고 최신 devel보다 287 commits 뒤였으므로 rebase로 fast-forward한 뒤
stash를 복원했다. renderer 소스는 자동 병합됐고 충돌은 없었다.

`mydocs/orders/20260818.md`는 upstream에 같은 이름으로 새로 생겨 stash의 untracked 파일만 자동
복원되지 않았다. upstream 내용을 보존하고 M050 #3128 행을 수동 병합했다. stash는 복원 안전망으로
계속 보존했다.

## 2. 최신 테스트 정책 정합

최신 devel은 새 integration source를 `tests/cases/`에 두고 generated suite로 자동 배정한다. 이에 따라
전용 테스트를 `tests/cases/issue_3128_terminal_nested_table_geometry.rs`로 이동했고
`regression_suite_004`에 배정됐다. source-side unit test 증가는 허용되지 않으므로 초기 composer 단위
회귀는 제거하고 같은 결과를 공개 render-tree integration 수용 테스트로 고정했다.

코드 삽입으로 기존 `cfg(test)` module 5곳과 support item 2곳의 줄 번호가 이동해
`tests/suites/unit-test-tiers.json`을 `--generate`로 재계산했다. static test 4,225건,
298 modules, 각 maximum과 tier는 바뀌지 않았다.

정책 검사 결과:

- Rust test-suite manifest 자체 테스트: 16 passed
- suite manifest check: 661 sources, 2,854 static test attrs, 32 suites + 9 exceptions
- unit-tier 자체 테스트: 11 passed
- unit-tier `--check --base-ref upstream/devel`: 4,225 tests, 정책 위반 0

## 3. focused 검증

| 대상 | 결과 |
| --- | ---: |
| #3128 전용 generated-suite 회귀 | 2 passed |
| #2308 normalized derived state | 5 passed |
| #1891 roundtrip/page-count | 4 passed |

최신 devel에서도 82쪽, p34 좌표와 `연동시스템 등` 줄바꿈이 유지됐다.

## 4. 전체 release·Clippy

```bash
cargo nextest run \
  --cargo-profile release-test \
  --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
cargo clippy --all-targets -- -D warnings
```

- release-test: **6,895 passed, 0 failed, 38 skipped**, 1 slow, 실행 92.209초
- Clippy: warnings 0, 통과

host에 `cargo-nextest`가 없어 검증 전 v0.9.143을 설치했다. 최신 devel 의존성 다운로드에는 승인된
외부 네트워크를 사용했다.

## 5. Native Skia 공식 범위

자동 sharding 이후 독립 test target 이름이 사라져 source runner로 해당 generated suite를 정확히
필터링했다.

| 범위 | 결과 |
| --- | ---: |
| `native-skia` lib `skia` filter | 58 passed |
| `issue_2225_missing_picture_placeholder` / suite 026 | 2 passed |
| `render_p37_direct_pdf_export` / suite 012 | 4 passed |

Skia binary는 `skia-bindings 0.99.0` 공식 GitHub release에서 내려받았다.

## 6. WASM

표준 `docker compose ... wasm` 경로를 먼저 시도했다. 호스트에는 standalone `docker-compose` CLI는
있지만 daemon이 실행되지 않았고 Docker Desktop 애플리케이션도 설치되어 있지 않아 컨테이너 gate는
시작할 수 없었다.

대체 경로로 고정판 `wasm-pack 0.15.0`의 최적화 빌드를 실행했다.

```bash
wasm-pack build --target web --out-dir pkg
```

- wasm32 release compile: 통과
- wasm-bindgen 0.2.127: 통과
- wasm-opt: 통과
- `rhwp_bg.wasm`: 8,679,328 bytes

이 결과는 **네이티브 최적화 WASM 통과**이며 Docker 표준 환경 통과로 표현하지 않는다. 생성된 `pkg/`와
로컬 `.env.docker`는 결과 확인 후 삭제했으며 둘 다 재생성 가능한 산출물이다.

## 7. 최신 devel 시각 재검증

HWP 2024 PDF 34쪽을 96dpi로 다시 비교했다.

- 페이지 수: 82
- outer continuation: y=75.6, h=389.1
- inner child: y=77.1, h=370.9
- 후속 직접편익 표: y=508.6
- pixel match: 90.03412%
- ink match: 11.11311%

재기준화 전 최종 결과와 수치가 정확히 동일하며 side-by-side 판정에서도 약 60px 하강 오차가 다시
생기지 않았다.

## 8. 결론

최신 devel의 focused·전체 release·Clippy·Native Skia·네이티브 최적화 WASM·시각 gate를 통과했다.
최종 정적 검사까지 통과했다. 작업지시자가 단계별 commit, push와 한국어 제목·본문의 `devel` 대상
PR 생성을 승인했으므로 게시 단계로 전환한다.
