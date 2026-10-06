# Task #7280 Stage 41 — R2 책임 묶음 통합 검증

- 이전 절편: [Stage40](task_m100_7280_stage40.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) §7.
- baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 제품 head: `99d144e3f346eed2e0b92771dfd54034e662153f` (문서 head `c9c475a35`).
- 상태: R2 책임 묶음 통합 검증 완료. 제품·테스트·기준값 변경 없이 아래 범위의 동작 보존을 확인했다. 전체 리팩토링·PR 제출 게이트 완료는 아니다.

## 승인 범위와 순서

R2 문단/컨트롤 경계를 재점검하고 동일 입력·설정의 Native 및 fresh Docker WASM
출력을 baseline과 비교한다. 대표 영향 페이지의 내용 소유·기하·줄바꿈을 먼저 확인한 뒤
전체 회귀를 수행한다. 기존 한컴 차이는 별도 기록하며 이번 구조 이동에서 수정하지 않는다.
새 테스트는 `tests/cases/`와 별도 integration suite를 사용한다. 이번 검증에서는 기존 계약을
재사용하며 source-side 테스트/support, ignore, baseline을 늘리지 않는다.

baseline 및 head의 기존 detached review worktree를 사용하고 Cargo는 공유
`target/pr-review`에서 순차 실행한다. CLI 바이너리는 각 빌드 직후 별도 증적 경로에 보존한다.
Docker WASM은 review worktree의 `pkg`에 빌드해 주 checkout의 Studio 패키지를 덮어쓰지 않는다.
원격 push·PR·댓글·devel 통합은 범위 밖이다.

## 책임 경계

R2는 문단의 형식/fit/split 진입, 컨트롤 순서·TAC·float 소유/흐름 조회 및 조정이다.
표 행 스캔·분할/이어받기 본체는 R3, 각주/미주 본체와 별도 인덱스는 R4,
페이지/구역 수명과 상태 가시성의 최종 제한은 R5에 남긴다.
기존 호환 분기·측정 의존을 유지한 추출을 새 조판 규칙의 승인으로 해석하지 않는다.

## 증적

`output/7280/stage41/`에 명령·입력/바이너리 해시·출력 비교·회귀 로그를 남긴다.
완료한 실행과 미검증 범위는 후속 기록에 구분한다.

### Native 비교와 빌드 캐시 검증

baseline CLI를 `cargo build --locked --profile release-test --target-dir
/home/edward/mygithub/rhwp/target/pr-review --bin rhwp`로 실제 재컴파일했다(2분 44초).
head 첫 빌드는 0.18초로 끝났으나 보존한 바이너리가 baseline과 동일했다. 공유 target의
최신 실행 파일 재사용 위험으로 판단하여 이 실행의 `head-stale-discarded` 산출물을
검증에서 제외했다. review worktree의 `src/lib.rs` mtime만 갱신해 다시 빌드했으며,
로그에서 해당 head worktree의 `Compiling rhwp`를 확인했다(2분 58초). 파일 내용 변경은 없다.

- baseline CLI SHA-256: `7191580d6d147d0e470a21d68bb6a85cf7d734828c97ee88be065933c587bfbe`.
- head CLI SHA-256: `054aa5f25040220bec0940dad9a3f1c8471688bf24f5bc027200b8da4c3bbd31`.
- `targets.json`, `native.mjs`, `compare-native.mjs`, `native-comparison.json`:
  8개 입력/262쪽의 `dump-pages` 전체 JSON(소유·컷 포함), RenderTree 전체 JSON,
  SVG 원문이 baseline/head 사이에서 일치했다. 페이지 수만 비교한 것이 아니다.
- 별도 `tests/`, Cargo.toml, Cargo.lock은 baseline 대비 변경 없음.
- source-side 정책은 고정 baseline 대비 통과: 기존 4,205 tests / 298 modules /
  cfg support 28 유지. integration manifest 1,382 sources / 48 targets 유지.
- 바이너리/JSON 일치만으로 직접 시각 검증 또는 모든 미실행 분기 동등성을 선언하지 않는다.

### Native 직접 판독

5개 PDF 쌍의 12개 선택 쪽에 대해 baseline/head 각각 새로 compare·standalone overlay·review를
생성했다. `visual-native-comparison.json`은 각 쪽의 절대 경로와 보조값을 담으며,
12개 rhwp PNG 원문이 baseline/head 사이에서 모두 일치했다. 변경본 12개 review를 직접 열어 확인했다.
설정은 layer SVG, 기본 compat 2022, 96 dpi, 동일 Chrome 146.0.7680.31/webfont 정책이다.
PDF 원장(text-only/export-all-svg/layout-ledger)도 양쪽에 별도로 생성했다.

| 사례 / 실제 쪽 | 직접 확인한 경계 | PDF 보조 일치율(%) / 남은 기존 차이 |
| --- | --- | --- |
| square-host / 1 | 좁은 어울림 본문과 후속 전폭 문단 | 9.77 / 글꼴·굵기 차이 |
| square-body / 1 | 그림 왼쪽 문단과 아래 전폭 복귀 | 46.43 / 글꼴·제목 차이; 축소 이미지 fixture |
| square-table / 5,6,7,8 | 표 왼쪽 prefix, 아래 tail, 다음 쪽 꼬리 줄 소유 | 30.78,10.40,19.19,12.09 / 7쪽 기존 세로 간격 차이 |
| night-guard / 1 | float 표 이후 본문 표와 하단 | 11.45 / PDF 하단 붙임 문구가 baseline/head 모두 보이지 않음 |
| deferred-picture / 126,127,155,156,182 | 직전 본문/각주 유지, 다음 쪽 그림 56/64, 캡션과 뒤 본문 | 9.59,19.48,8.89,14.15,6.83 / 기존 글머리표·그림 크기·182쪽 세로 위치 차이 |

직접 판독한 PNG는 `output/7280/stage41/native/head/<key>/<key>/review/review_NNN.png`,
비교와 standalone overlay는 같은 key 아래 `compare/compare_NNN.png`,
`overlay/overlay_NNN.png`다. 원본/PDF SHA-256은 `native/{base,head}/provenance.json`에 있다.
PDF 출처는 #4090의 MCP PrintToPDFEx 기록, #6309/#6314 및 #6572 검토 기록,
#3820 Stage9의 HWP 한컴 2020 기준 PDF 기록을 대조했다.
보조값은 사람 판정 정확도가 아니며 기존 차이를 조판 정답·해결 완료로 승인하지 않는다.

책임 재점검: `typeset.rs`의 문단/표 문단/어울림 façade는 각각 paragraph·controls 조정자를
호출하고 원래 구역 순서/continue를 유지한다. `controls/wrap_flow.rs`의 밴드 종료는 높이 확정
명령으로 남고, `deferred_picture.rs`는 후보만 반환하여 실제 큐/쪽 전이는 부모가 수행한다.
R3 `scan_block_table_split_rows`/`typeset_block_table_inner`, R4 `typeset_endnote_paragraphs`,
R5 `push_new_page`와 기존 `TypesetState` 필드 접근은 그대로 남는다. 모든 상태가 이미 캡슐화됐거나
전체 엔진이 순수 Query/Command로 분리됐다는 뜻이 아니다.

### 재현 명령과 환경

```bash
# 주 checkout; base/head CLI는 각 고정 worktree에서 빌드 직후 bin/에 보존
node output/7280/stage41/native.mjs base
node output/7280/stage41/native.mjs head
node output/7280/stage41/compare-native.mjs
node output/7280/stage41/visual.mjs base ledger
node output/7280/stage41/visual.mjs head ledger
node output/7280/stage41/visual.mjs base native
node output/7280/stage41/visual.mjs head native
node output/7280/stage41/compare-visual.mjs native

# 각 review worktree의 pkg에 생성, main checkout의 pkg는 비변경
docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=4 wasm

# head review worktree, 기본 feature 전체 회귀
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --tests --test-threads 8 --no-fail-fast
```

Host/Docker rustc 1.93.1, Docker wasm-pack 0.15.0, Node v24.15.0, Python 3.12.3.
Docker image는 `rhwp-wasm:latest`의
`sha256:4824b312625eba7b8146aa1c7eaca95710618beb75c4cec15470cdb76bc2d99b`를 사용했다.
동일 named WASM target은 baseline → head 순차 사용하고 head의 `Compiling rhwp`를 확인했다.
Native와 Docker는 서로 다른 target/cache다. head WASM 최적화 중 test 자원 경합을 줄이기 위해
해당 실행 컨테이너에만 4 CPU quota를 적용했다. 공유 캐시/target을 삭제하지 않았다.

nextest는 0.9.137이며 권장 0.9.140 및 observation 설정 경고는 baseline과 같다.
이 실행은 고정 기준과의 로컬 회귀 비교이며 원격 CI 전체와 동일한 환경이라고 주장하지 않는다.

### Fresh WASM 빌드

baseline Docker 빌드 exit 0(7분 50초), head Docker 빌드 exit 0(10분 23초).
두 실행 모두 실제 `Compiling rhwp` 및 wasm-opt 완료를 확인했다.

- baseline `rhwp_bg.wasm`: `70f6cda1825458bc5246f360cc635259154613ddb7a91fc4f64c2967a0fb585e`.
- head `rhwp_bg.wasm`: `7949e1290df287d00aa2a9884c91eb7d3bce6bf011055af1a732a73c1e06258a`.
- JS binding은 양쪽 동일: `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- `wasm-base.log`, `wasm-head.log`, `tools-and-wasm-base.sha256`, `wasm-head.sha256`에 보존.
- 실제 Chrome `HwpDocument.renderPageSvg`/`getPageRenderTree` 경로로 각각 새로 생성한다.
  Native tree를 WASM 결과로 대신하지 않는다. Studio UI 조작/Canvas backend 전체 검증은 아니다.

```bash
node output/7280/stage41/visual.mjs base wasm
node output/7280/stage41/visual.mjs head wasm
node output/7280/stage41/compare-visual.mjs wasm
```

WASM 비교 완료: **5개 입력/238쪽**의 raw SVG와 RenderTree가 baseline/head 사이에서 일치한다.
선택 12쪽의 새 PNG도 baseline/head 및 head Native/WASM 사이에서 원문 일치했다.
`visual-wasm-comparison.json`에 실제 검증 쪽수·경로·보조값을 남겼다.
head의 square-host/body·night-guard 개별 review, square-table/deferred-picture의 review contact
sheet를 직접 열어 Native에서 판독한 배치 경계와 대조했다. 개별 비교/overlay/review 파일은
`output/7280/stage41/wasm/head/<key>/<key>/` 아래 Native와 같은 파일명으로 보존했다.
기존 PDF 차이는 위 표와 동일하며, 12쪽의 보조값도 동일하다.

이 검증은 편집 전 원본을 여는 대표 경로다. 모든 반례 조합의 branch coverage,
Studio 실제 Canvas·편집 후 재조판 전수, 미선택 페이지의 직접 판독을 대신하지 않는다.
원본 262/238쪽의 데이터 비교 범위와 12쪽의 시각 확인 범위를 구별한다.

## 전체 회귀 결과와 판정

고정 제품 head의 기본 feature 전체 nextest는 exit 0으로 완료했다.

- **10,096건 통과 / 실패 0건 / 기존 제외 50건**, 78 binaries.
- 빌드 9분 26초, 테스트 510.173초.
- run ID: `bde502e7-08be-4f70-a3ad-03a42a9d9fdd`.
- 로그: `output/7280/stage41/nextest-full.log`.
- `compare-regression.mjs` / `regression-comparison.json`: Stage1 baseline의
  10,096개 PASS 이름과 정확히 일치한다. baseline 테스트 시간은 442.237초였으며,
  자원 경합 조건이 달라 이 시간 차이를 성능 회귀 또는 개선으로 판정하지 않는다.
- `prepare.log`, `manifest.log`, `unit-tier.log`, `fmt.log`의 검증도 통과했다.

R2 구조 이동의 기존 회귀 계약과 대표 Native/fresh WASM 출력 보존은 **충족**이다.
위 PDF 차이의 해결이나 한컴 피델리티 전체 통과는 주장하지 않는다. 제품·테스트·샘플·기준값·
ignore를 변경하지 않았으며, 검증 후 두 review worktree의 tracked 변경도 없다.
이번 절편은 결과 문서만 커밋하며 파생 suite·출력 산출물은 stage하지 않는다.

## 다음 범위

다음은 R3 표 포맷·행 스캔·블록 분할·이어받기 책임 분리다. 기존 알고리즘과 분기 의미를
보존하면서 Query/Command 및 소비 지점을 분리하고, 새 회귀 계약이 필요하면
`tests/cases/`의 별도 integration suite에 둔다. 이번 단계에서는 R3 구현을 시작하지 않았다.
R4 각주/미주 본체, R5 구역·페이지 수명/상태 접근 제한, R6 기여자 가이드·복잡도 재계측도 남는다.

구현계획 §7.3의 최종 제출 head 기준 native/WASM/workspace Clippy 묶음,
workspace build·Native Skia 검증과 PR base 재고정은 후속 제출 게이트다.
이번 검증을 그 전체 게이트의 대체 증거로 사용하지 않는다. 원격 push·PR·댓글은 수행하지 않았다.
