# PR #599 검토 보고서

**PR**: [#599 render: add native Skia PNG raster backend](https://github.com/edwardkim/rhwp/pull/599)
**작성자**: @seo-rii (이전 PR #498 P3 단계의 동일 컨트리뷰터)
**상태**: OPEN, **mergeable=MERGEABLE** (GitHub UI 기준)
**관련**: refs #536 (멀티 렌더러 지원 트래킹 이슈, M100 OPEN)
**선행 PR**: PR #498 (P3 — Canvas visual diff, CLOSED + cherry-pick 처리됨)
**처리 결정**: ⏳ **검토 중** (1차 검토)
**검토 시작일**: 2026-05-06

## 1. 검토 핵심 질문

1. **본질 식별 정합성** — `PageLayerTree` 를 native Skia 로 replay 해서 PNG 를 만드는 첫 raster backend 추가가 본 환경 정체성 (DTP 엔진 + 다층 레이어, `project_dtp_identity`) 정합한가?
2. **PR base skew 분석** — PR base (`eaac8bd`, 5/4 PR #563 처리 후속 시점) 가 본 devel 보다 73 commits 뒤처짐. 단순 머지 시 본 사이클 cherry-pick 처리분 (PR #589/#561/#567/#564/#570/#575/#580/#584/#592/#593) 사실상 revert?
3. **Skia 본질 commit 단위 cherry-pick 가능성** — 9 commits 의 변경 영역이 본 사이클 처리분과 0 중첩하는가?
4. **`native-skia` feature gate** — 기본 빌드 영향 없이 opt-in 가능한가?

## 2. PR 정보

| 항목 | 값 | 평가 |
|------|-----|------|
| 제목 | render: add native Skia PNG raster backend | 정합 |
| author | @seo-rii (이전 컨트리뷰터들과 다른 영역 — Skia/렌더러 전문) | 본 사이클 첫 비-jangster77 외부 PR |
| changedFiles | 10 / +1,917 / -2 | (PR diff 표시) |
| 본질 변경 | `src/renderer/skia/` 신규 + `layer_renderer.rs` 신규 + `image_conv.rs` 신규 + Cargo feature + CI workflow | 광범위 신규 영역 |
| **mergeable** | MERGEABLE (UI) | 그러나 PR base skew 73 commits |
| 선행 | refs #536 (M100 OPEN), PR #498 (P3, CLOSED) | P4 단계 |

## 3. PR 의 9 commits 분석 — Skia 본질 commits

| commit | 설명 | 영역 |
|--------|------|------|
| `61baeb9` feat: layer raster renderer contract | `layer_renderer.rs` 신규 | 0 base 충돌 |
| `6b4bbe6` feat: native skia layer png renderer | Skia mod 신규 + Cargo feature + README + queries/rendering | 0 |
| `93ed6ad` test: cover native skia raster guards | Skia renderer 테스트 | 0 |
| `4cf787f` fix: use font fallback in native skia text | Skia renderer 폰트 fallback | 0 |
| `11181f4` test: cover current native skia paths | Skia renderer 테스트 추가 | 0 |
| `82d4007` ci: test native skia feature | CI workflow + README + queries/rendering | 0 |
| `6e009dc` feat: improve native skia image replay | Skia image_conv 신규 + renderer 강화 | 0 |
| `015ff45` docs: clarify native skia render path | README only | 0 |
| `010f6cf` fix: cap native skia raster pixel count | layer_renderer + Skia renderer | 0 |

→ **9 commits 모두 본 사이클 처리분과 0 중첩**. Skia 본질 영역 (`src/renderer/skia/` + `layer_renderer.rs` + `image_conv.rs` + Cargo feature + CI + README + queries/rendering) 으로 완전 격리.

## 4. PR base skew 분석 (PR #571 동일 패턴)

### 4.1 PR base 시점

- PR base: `eaac8bd` (5/4 PR #563 처리 후속)
- 본 devel HEAD: `7bce24d` (PR #593 처리 후속, PR base 대비 **73 commits ahead**)
- PR base 와 본 devel 사이의 73 commits = 본 사이클 모든 cherry-pick (PR #589/#561/#567/#564/#570/#575/#580/#584/#592/#593)

### 4.2 PR base diff 의 광범위 deletion

GitHub UI 의 PR base diff 가 표시하는 영역 (단순 머지 시 사실상 revert 되는 영역):

| 영역 | Revert 분 | 누락 처리 |
|---|---|---|
| `tests/issue_554.rs` | -113 (삭제) | PR #589 Task #554 |
| `integration_tests.rs` | -157 | test_521/test_552/test_544/test_548/test_574 (PR #561/#564/#584) |
| `paragraph_layout.rs` | -105 | PR #570 Task #568 + PR #592 Task #588 |
| `table_layout.rs` | -159 | PR #561 Task #548 + PR #575 Task #573 + PR #580 Task #577 |
| `style_resolver.rs` | -5 | PR #584 Task #574 |
| `mydocs/pr/archives/*` | -광범위 | 본 사이클 처리/검토 보고서 모두 |
| `examples/inspect_574.rs` | -197 | PR #584 진단 스크립트 |
| `mydocs/orders/2026050{3,4,5}.md` | -광범위 | 본 사이클 orders |
| `mydocs/manual/memory/...` | -5 | 메모리 동기화 |

→ **PR #571 base skew 패턴과 완전히 동일** — 단순 머지 시 본 사이클 73 commits revert.

### 4.3 본질 cherry-pick 가능성

그러나 **commit 단위로 cherry-pick** 시 — Skia 본질 commits 는 본 사이클 처리분과 **0 중첩 (분석 §3)** → 충돌 없이 깨끗 적용 가능.

→ **PR #571 (close + 분리 PR 권장) 과 다른 처리 방향**. 본 PR 은 Skia 본질만 명확히 격리되어 있어 핀셋 cherry-pick 처리 가능.

## 5. 본 환경 직접 검증 (임시 브랜치 cherry-pick test)

`pr599-cherry-test` 임시 브랜치에서 9 commits 순차 cherry-pick:

| 단계 | 결과 |
|------|------|
| 9 commits 순차 cherry-pick | ✅ 모두 충돌 0 (auto-merge) |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo clippy --release --lib` | ✅ 0건 |
| **`cargo test --features native-skia skia --lib`** | ✅ **20 passed** (Skia 본질 + queries/rendering 테스트 모두 GREEN) |

→ **commit 단위 cherry-pick 정합 입증** + Skia 본질 영역 결정적 검증 통과.

## 6. PR 본문의 자기 검증 결과

| 검증 | PR 본문 결과 | 본 환경 재검증 (임시) |
|------|---------|----------|
| `cargo test` | ✅ | ✅ 1134 passed |
| `cargo clippy -- -D warnings` | ✅ | ✅ 0건 |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ | ⏳ 본격 검증 권장 (WASM 영향 0 확인) |
| **`cargo test --features native-skia skia --lib`** | ✅ | ✅ 20 passed |

## 7. 본 PR 의 비목표 명시 — 정합 평가

PR 본문 §"비목표" 명시:
- CanvasKit renderer (browser/WASM 미포함)
- C ABI / CLI PNG export 미포함
- resource interning/cache 미포함
- complex text shaping parity 미포함
- 완전한 equation/raw-svg/form native replay 미포함 (현재 placeholder/fallback)
- Skia visual regression fixture pipeline 미포함

→ P4 단계의 명확한 범위 한정. **점진적 멀티 렌더러 마이그레이션** 패턴 (`project_dtp_identity` 정합 — DTP 엔진 토대 점진 구축).

## 8. 메인테이너 정합성 평가

### 정합 영역
- ✅ **본 환경 정체성 정합** — DTP 엔진 + 다층 레이어 / WebGPU / 마스터 페이지 토대 (`project_dtp_identity` 메모리 룰)
- ✅ **9 commits 모두 본 사이클 처리분과 0 중첩** — `src/renderer/skia/` + `layer_renderer.rs` + `image_conv.rs` 신규 영역
- ✅ **commit 단위 cherry-pick 충돌 0** — auto-merge 정합
- ✅ **`native-skia` feature gate** — 기본 빌드 영향 없음 (opt-in)
- ✅ **결정적 검증 정합** — cargo test --lib 1134 passed (회귀 0) / clippy 0 / Skia 테스트 20 passed
- ✅ **CI workflow 추가** — native Skia 테스트 자동화 (`82d4007 ci: test native skia feature`)
- ✅ **광범위 fallback 처리** — equation / raw-svg / form / invalid image / placeholder 처리
- ✅ **raster size guard** — invalid page dimension / scale / dpi reject + max dimension / pixel count 제한
- ✅ **문서화** — README / README_EN 에 native Skia 경로 + 비목표 정리
- ✅ **선행 PR 정합** — PR #498 (P3) 의 후속 P4 단계 (refs #536)

### 우려 영역
- ⚠️ **PR base skew (73 commits)** — GitHub UI MERGEABLE 표시지만 base diff 가 본 사이클 cherry-pick 모두 revert 표시. **단순 머지 절대 금지** + commit 단위 cherry-pick 필수
- ⚠️ **광범위 신규 영역** — Skia 의존성 (`tiny-skia` 또는 `skia-safe`?) 추가, `native-skia` feature 활성화 시 빌드 시간 영향
- ⚠️ **외부 컨트리뷰터의 광범위 신규 영역 흡수** — feature gate 로 격리되어 있어도 향후 유지보수 부담 평가 필요
- ⚠️ **시각 판정 불필요** — PR 본문 명시 "Skia 경로는 아직 완성형 renderer라기보다는 ... 검증하는 초기 backend". 시각 판정 게이트는 P5+ 단계에서

## 9. 1차 검토 결론

### 정합 평가
- ✅ **본질 cherry-pick 가능 (commit 단위)** — 9 commits + 충돌 0
- ✅ **결정적 검증** — 1134 passed (회귀 0) + Skia 20 passed + clippy 0
- ✅ **본 환경 정체성 정합** — DTP 엔진 토대
- ✅ **feature gate 격리** — opt-in `native-skia`
- ⏳ **WASM 영향 검증** — `cargo check --target wasm32-unknown-unknown` 본격 검증 권장
- ⏳ **메인테이너 검토** — 광범위 신규 영역 의존성 / 빌드 시간 영향 평가

### 권장 처리 방향 (작업지시자 결정 대기)

#### 옵션 A — 핀셋 cherry-pick (권장)
- 9 commits 순차 cherry-pick (단순 머지 절대 금지)
- 본 환경 결정적 재검증 + WASM 빌드 영향 검증
- (시각 판정은 P4 의 비목표 영역, 결정적 검증으로 충분)
- 통과 시 devel merge + push + PR close 처리
- **PR #571 base skew 처리와 다른 방향**: PR #571 은 본질 영역까지 본 사이클 처리분과 충돌해서 close + 분리 PR 권고였지만, 본 PR 은 Skia 영역이 0 중첩 → cherry-pick 가능

#### 옵션 B — 추가 정정 요청
- WASM 영향 / 빌드 시간 / 의존성 영역에 우려 발견 시 추가 정정 요청

#### 옵션 C — close + 본 환경 직접 처리
- (Skia 본질 영역의 외부 흡수에 우려 시)

→ **작업지시자 결정 대기**. 옵션 A 권장 — Skia 본질 영역 격리 + commit 단위 cherry-pick 정합 + 결정적 검증 통과 + 본 환경 정체성 정합.

## 10. 옵션 A 진행 결과 (작업지시자 승인 후)

### 10.1 핀셋 cherry-pick (commit 단위)

| 단계 | 결과 |
|------|------|
| 9 commits 순차 cherry-pick | ✅ 모두 충돌 0, author seo-rii 보존 |
| 본 사이클 처리분과 중첩 | 0 (Skia 신규 영역 완전 격리) |

### 10.2 메인테이너 후속 정정 (`876d820`)

PR #599 본질만으로는 한컴 fixture 가 정상 표시되지 않아 5개 영역 추가 정정:

1. **Skia 한글 폰트 fallback chain** (`renderer.rs`) — Noto Sans KR / Nanum 등
2. **`--font-path` 동적 로딩** (`with_font_paths` API) — SVG 패턴 정합
3. **char 단위 fallback** (공백 두부 정정) — NBSP/U+2007/U+200B 방지
4. **VLM 옵션** (AI 파이프라인 연동) — `--vlm-target claude` + `--scale` + `--max-dimension`
5. **`export-png` CLI 명령** (`main.rs`) — native-skia feature gate
6. **매뉴얼** (한글 + 영문 동기화)

### 10.3 결정적 검증 (모두 통과)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed / 2 ignored (회귀 0) |
| `cargo test --features native-skia skia` | ✅ **20 passed** |
| `cargo clippy --release --lib --features native-skia` | ✅ 0건 |
| `cargo build --release` | ✅ Finished |
| `cargo build --release --features native-skia` | ✅ Finished |
| Docker WASM 빌드 | ✅ **4,581,465 bytes** (PR #593 baseline +0 — feature gate 정합 입증) |

### 10.4 광범위 페이지네이션 sweep

| 통계 | 결과 |
|---|---|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 | **1,614** |
| Export 실패 | 0 |

→ Skia feature 가 default 빌드에 미포함이라 기본 SVG 경로 영향 0 확인.

### 10.5 VLM 옵션 게이트웨이 검증

| 옵션 | 출력 dimension | 평가 |
|---|---|---|
| (기본) | 1123 × 1588 | native |
| `--scale 2.0` | 2246 × 3175 | 정확 4배 ✓ |
| `--scale 0.5` | 562 × 794 | 정확 1/4배 ✓ |
| `--max-dimension 1024` | 725 × 1024 | longest=1024 ✓ |
| `--vlm-target claude` | 898 × 1269 | **1.14 MP ≤ 1.15 MP** ✓ |

### 10.6 후속 이슈 등록

- [#613](https://github.com/edwardkim/rhwp/issues/613) — VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA)
- [#614](https://github.com/edwardkim/rhwp/issues/614) — DPI 메타데이터 옵션 (`--dpi`)

### 10.7 다음 단계

1. ✅ 본 1차 검토 보고서 작성
2. ✅ 본 환경 결정적 재검증
3. ✅ 광범위 페이지네이션 sweep
4. ✅ Docker WASM 빌드
5. ✅ 메인테이너 후속 정정 (한글 fallback + char-fallback + font-path + VLM 옵션 + CLI + 매뉴얼)
6. ✅ 후속 이슈 #613/#614 등록
7. ⏳ devel merge + push + PR close + 처리 보고서 + 컨트리뷰터 후속 커멘트

## 11. 메모리 정합

- ✅ `project_dtp_identity` — DTP 엔진 + 다층 레이어 / WebGPU / 마스터 페이지 인프라 토대 (M200+ 후보 B WebGPU 합리화 근거)
- ✅ `feedback_per_task_pr_branch` — refs #536 (P4 단계 단일 본질) 정합
- ✅ `feedback_pr_comment_tone` — 본 검토 톤 차분/사실 중심
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/6) 누적 21번째 PR
- ✅ **PR #571 패턴과 다름** — PR #571 은 base skew 로 close + 분리 PR 권고였지만 본 PR 은 본질 영역 0 중첩으로 핀셋 cherry-pick 가능

---

**검토자**: 클로드 (페어 프로그래밍 파트너)
**1차 검토 단계 — 작업지시자 승인 대기**.
