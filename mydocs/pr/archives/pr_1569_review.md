# PR #1569 처리 보고서 — Skia FontMgr/시스템 폰트 thread-local 캐시 (성능)

- PR: https://github.com/edwardkim/rhwp/pull/1569
- 제목: `perf: cache FontMgr + system font enumeration thread-locally in SkiaLayerRenderer::new`
- 작성자: humdrum00001010 (누적 9 PR)
- 연결: 없음 (독립 성능 개선)
- base ← head: `devel` ← `humdrum00001010:perf/skia-fontmgr-thread-local-cache-devel`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge.** `SkiaLayerRenderer::new()` 가 페이지마다 시스템 폰트를 전수 열거(~8.4ms/call)하던
것을 thread-local 캐시로 제거. 컨트리뷰터의 두 핵심 주장(렌더 출력 byte-identical, 성능 개선)을
메인테이너가 직접 검증해 통과 확인. CI 전부 pass + 충돌 0건.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/skia/renderer.rs` | `SkiaLayerRenderer::new()` 에 thread_local `(FontMgr, SystemFontFamilies)` 캐시 (+16/-6) |

native-skia feature 전용. wasm/NIF/배포 무영향.

## 3. 코드 검토

- `thread_local!` 로 `(FontMgr, SystemFontFamilies)` 를 스레드당 1회 계산, 이후 `new()` 는 clone
  (FontMgr=refcount bump, families=`HashSet<String>` clone ~수십 µs).
- 시스템 폰트는 프로세스 수명 동안 불변 → 캐시 안전. thread-local 이라 멀티스레드 렌더에서도
  각 스레드 독립 캐시로 안전.
- 폰트 매칭 입력 불변 → 렌더 출력 byte-identical.

## 4. 컨트리뷰터 주장 직접 검증 (메인테이너)

### 주장 1: 렌더 출력 byte-identical — **검증 통과**

- before(devel, 캐시 전) / after(PR, 캐시 후) 바이너리를 각각 `--release --features native-skia` 빌드.
- 동일 문서(`samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx`, 30페이지) `export-png` 렌더.
- **md5 해시 30/30 완전 일치 (diff 0)** → 캐시가 폰트 매칭 결과를 바꾸지 않음을 바이트 단위로 확인.
- CI `Canvas visual diff` pass 와 일치. 증빙: `output/poc/pr1569/{before,after}_hashes.txt`.

### 주장 2: ~8.4ms/page 성능 개선 — **검증 통과**

- 30페이지 wall-clock: before 평균 3.91s → after 평균 3.64s.
- 차이 ≈ 0.27s / 30p = **~9ms/page 절감** → PR 주장(~8.4ms/page)과 일치.

## 5. 기타 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 시뮬레이션 | 0건 |
| `cargo test --release --features native-skia --lib` | 1984 passed / 0 failed |
| `cargo clippy --features native-skia --lib -- -D warnings` / fmt | clean |

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1569_review.md`
- 검증 증빙: `output/poc/pr1569/` (before/after PNG 해시)
