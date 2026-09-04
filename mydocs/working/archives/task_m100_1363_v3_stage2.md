# Stage 2 (v3) — scratch 측정 부작용 격리 실증

Stage 1 이 배선한 scratch `measure_endnote_para_advance` 의 상태 변이가 측정에만 머물고 실제
렌더에 새지 않음을 **구조 분석 + 단위 테스트 + 전 suite**로 실증했다. 격리 메커니즘(매 호출
`LayoutEngine::new()`)은 Stage 1 설계로 이미 확보돼 있어 production 코드 변경 없이 검증·가드만
추가했다.

## 1. 격리 경계 — 3계층 분리 (구조 증명)

| 엔진 | 가변 상태 | 측정 영향 |
|------|----------|----------|
| `self` = `TypesetEngine` | 측정은 `scratch.layout_partial_paragraph()` 호출 — `self` 필드 **무접근** | 없음 |
| 실제 렌더 `LayoutEngine`(별도 인스턴스) | `auto_counter`/`numbering_state`/`layout_overflows`/`last_item_content_bottom` 등 **인스턴스-로컬 RefCell/Cell** | 없음 |
| scratch `LayoutEngine` | 매 호출 fresh `new()` → 호출 간 누적 없음. 노드는 버리는 `tree`/`col_node` | 측정 전용 |

핵심: 세 엔진은 **서로 다른 인스턴스**이며 numbering/overflow 상태는 모두 인스턴스-로컬이라
Rust 소유권상 공유 가변 상태가 존재하지 않는다.

### 1.1 전역 가변 상태 부재
- `static mut`/전역 카운터/`lazy_static` **부재**(layout·typeset 경로 grep 0건).
- 유일 `thread_local` = `text_measurement.rs::JS_MEASURE_CACHE` — **`#[cfg(target_arch =
  "wasm32")]` 게이트**. 네이티브 빌드(전 테스트·`export-svg`)에는 **존재하지 않음** → 네이티브
  격리는 완전. WASM 에서도 결정적 read-through(같은 font+char→같은 폭) 순수 메모이즈라 정확성
  누수 없음(캐시 채움만).

## 2. 단위 테스트 — 측정 결정성·무누수 (신규 가드)

`src/renderer/typeset.rs` `tests::test_measure_endnote_advance_side_effect_free`:

- **(a) 양수·유한**: 실제 텍스트 para 측정이 advance 를 만든다.
- **(b) 결정성**: 동일 엔진 5회 반복 호출 → 동일값(호출 간 scratch 상태 무누적).
- **(c) 인스턴스 독립**: 독립 `TypesetEngine` 2개 → 동일값(전역 가변 상태 누수 없음).

→ `cargo test --lib test_measure_endnote_advance_side_effect_free` **1 passed**.

## 3. 전 suite·A3 거동 검증

### 3.1 기본(B) 무회귀 — A3 순수 opt-in ✓
전체 `cargo test` 기본 레벨 **123/123 바이너리 ok**(Stage 1 검증 유지). scratch 가 공유 상태를
오염했다면 default 경로도 깨졌을 것이나 무회귀 → **누수 부재의 반증적 증거**.

### 3.2 A3 vs B 콘텐츠 비교 — 차이는 layout-경계 한정 (numbering 무오염)
`3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`, `export-svg`(네이티브, `--bin rhwp`):

- 페이지 수: B=23, **A3=25**(단줄 +6px trailing 누적으로 미주가 2쪽 더 흘러넘침 — Stage 1
  divergence 의 페이지化, 예상).
- 가시 텍스트 char freq delta: **거의 전부 `�`(수식 PUA 글리프)·수식 내부 digit**, 각 ±1. 수식
  para 가 A3 의 다른 단-분할 경계에서 재렌더되며 생기는 **layout 차이**이지 상태 오염 아님.
- 본문 한글·미주 번호 digit 은 분할 경계 외 동일. numbering 자릿값 왜곡(중복/누락) 징후 없음
  (왜곡 시 default 도 깨짐 — §3.1 과 모순).

> CLI `cargo run` 은 `--bin rhwp` 명시 필요(`font-metric-gen`/`rhwp` 2개 바이너리). Stage 1
> 조사 중 일부 디버그 런이 binary 미지정으로 무출력이었던 점 정정 — 측정 경로는 테스트로 실재
> 동작 확인됨.

## 4. 결론 + 다음 (Stage 3)

- **격리 실증 완료**: 구조(별도 인스턴스·전역 무) + 단위 테스트(결정성·독립성) + 전 suite 무회귀.
  scratch 측정은 numbering/overflow/콘텐츠를 오염하지 않는다.
- **production 코드 무변경**: 측정 격리 메커니즘은 Stage 1 의 per-call `new()` 로 충분. Stage 2 는
  회귀 가드 테스트 1건만 추가.
- **Stage 3(split/fit 정합)**: Stage 1 이 드러낸 단줄 **+6px trailing-ls** 를 노트 간 적층 규칙에
  맞춰 측정값에 반영 + fit 게이트(`a2_overflow_with_para`)·`split_endnote_to_fit` 를 정확 측정으로
  구동 → A3 overflow 3건(`exam_3_09_2022`/`3_11_2022`/`3_09_2024_sep2020`) 재정합 목표.
  `trailing_model_no_ssot` 제약(전면 통일 금지) 준수.
