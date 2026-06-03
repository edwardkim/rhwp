# Task 397 — 3단계 완료 보고서: 비교 분석 및 rhwp 적용 방안

## 비교 대상 기술 요약

3가지 텍스트 레이아웃 엔진과 rhwp 현행 구조를 비교한다. 2단계까지의 분석 과정에서 linebender/parley가 추가 검토 대상으로 부상하여 함께 비교한다.

## 기술 비교표

| 항목 | SkParagraph | cosmic-text | parley | rhwp 현행 |
|------|-------------|-------------|--------|-----------|
| **언어** | C++ (Rust FFI) | 순수 Rust | 순수 Rust | 순수 Rust |
| **셰이핑** | HarfBuzz (C++) | harfrust | harfrust | 없음 (문자별 독립 측정) |
| **줄바꿈** | ICU (C++) | unicode-linebreak | ICU4X (Rust) | 자체 구현 |
| **BiDi** | ICU BiDi | unicode-bidi | ICU4X | 없음 |
| **폰트 폴백** | 플랫폼별 + 커스텀 | fontdb + 정적 리스트 | fontique | 없음 (내장 메트릭) |
| **폰트 읽기** | Skia 내부 | swash | skrifa (read-fonts) | 없음 |
| **WASM 호환** | ✗ (emscripten만) | △ (가능하나 미확인) | △ (순수 Rust, 가능성 높음) | ◎ (현재 동작) |
| **wasm32-unknown-unknown** | ✗ | △ | △ | ◎ |
| **편집 기능** | 없음 | Editor (커서/선택) | PlainEditor | 자체 에디터 |
| **리치 텍스트** | TextStyle 스택 | AttrsList 범위 | StyleProperty 범위 | CharShapeRef 범위 |
| **정렬** | L/R/C/Justify | L/R/C/Justified/End | L/R/C/Justified/End | L/R/C/Justify/Distribute |
| **줄간격** | StrutStyle (height/leading) | Metrics (line_height) | line_height | 4종류 (Percent/Fixed/BetweenLines/AtLeast) |
| **문단 여백/들여쓰기** | ✗ | ✗ | ✗ | ◎ (자체 구현) |
| **문단 간격** | ✗ | ✗ | ✗ | ◎ (spacing_before/after) |
| **탭 정의** | 제한적 | 고정 너비만 | 미확인 | ◎ (L/R/C/Decimal + leader) |
| **번호/글머리표** | ✗ | ✗ | ✗ | ◎ (자체 구현) |
| **HWP 장평** | ✗ | ✗ (stretch는 다른 개념) | ✗ | ◎ (ratios[7]) |
| **언어별 자간** | ✗ | ✗ (단일 값) | ✗ | ◎ (spacings[7]) |
| **언어별 폰트** | 폰트 폴백 의존 | 폰트 폴백 의존 | 폰트 폴백 의존 | ◎ (font_ids[7]) |
| **텍스트 장식** | ◎ | ✗ (외부) | 미확인 | ◎ (자체 렌더링) |
| **좌표→글리프 역매핑** | ◎ | ◎ (hit) | ◎ | ◎ (compute_char_positions) |
| **라이선스** | BSD-3 | MIT/Apache-2.0 | MIT/Apache-2.0 | MIT |
| **성숙도** | 매우 높음 (Chrome/Flutter) | 높음 (v0.18.2, COSMIC DE) | 성장 중 (v0.8.0, NLnet 지원) | — |
| **빌드 복잡도** | 매우 높음 (Skia C++) | 낮음 | 낮음 | — |

## rhwp 적용 시나리오 분석

### 시나리오 A: cosmic-text 전면 도입

셰이핑 + 줄바꿈 + 레이아웃을 모두 cosmic-text에 위임.

| 항목 | 평가 |
|------|------|
| **영향 범위** | text_measurement.rs, composer/, paragraph_layout.rs 전면 교체 |
| **마이그레이션 난이도** | 매우 높음 |
| **장점** | 셰이핑/BiDi/폰트 폴백 일괄 해결 |
| **리스크** | HWP 고유 기능(장평, 언어별 자간, 4종 줄간격, 탭, 번호) 재구현 필요. cosmic-text Metrics가 font_size+line_height 2개뿐이라 HWP 줄간격 모델과 근본적 불일치. WASM 폰트 로딩 문제. 기존 755개 테스트 대부분 파괴 |
| **결론** | **비권장**. 얻는 것보다 잃는 것이 많음 |

### 시나리오 B: SkParagraph(skia-safe) 전면 도입

| 항목 | 평가 |
|------|------|
| **영향 범위** | 렌더링 파이프라인 전체 |
| **마이그레이션 난이도** | 극도로 높음 |
| **장점** | 최고 품질의 텍스트 렌더링 |
| **리스크** | `wasm32-unknown-unknown` 미지원으로 WASM 빌드 불가. C++ 의존성으로 빌드 복잡도 폭증. HWP 고유 속성 매핑 한계 |
| **결론** | **불가**. WASM 호환성 요건 충족 불가 |

### 시나리오 C: 셰이핑 엔진만 선별 도입

cosmic-text 또는 parley에서 사용하는 **harfrust**(순수 Rust HarfBuzz)를 직접 도입하여 텍스트 셰이핑만 교체. 줄바꿈, 문단 레이아웃, 페이지네이션은 rhwp 자체 구현 유지.

| 항목 | 평가 |
|------|------|
| **영향 범위** | text_measurement.rs (TextMeasurer 구현부), font_metrics_data.rs |
| **마이그레이션 난이도** | 중간 |
| **교체 대상** | 내장 폰트 메트릭 582개 → harfrust 실시간 셰이핑 |
| **유지 대상** | composer/ (줄바꿈), paragraph_layout.rs, pagination/, 모든 HWP 고유 로직 |
| **장점** | (1) 커닝/리가처 정확한 글리프 측정 → 편집 시 줄바꿈 정확도 향상 (2) 미등록 폰트 휴리스틱 제거 (3) Faux Bold 보정 불필요 (4) 기존 아키텍처 최소 변경 |
| **리스크** | (1) WASM에서 폰트 데이터 로딩 방안 필요 (2) HWP 장평(ratios)을 셰이핑 후 스케일링으로 에뮬레이션 필요 (3) WASM 바이너리 크기 증가 (~1-3MB) |
| **WASM 전략** | 네이티브: harfrust + 시스템 폰트. WASM: harfrust + JS Canvas 폰트 or 번들 폰트 |
| **결론** | **권장 검토 대상**. 핵심 문제(텍스트 측정 부정확)를 최소 영향으로 해결 |

### 시나리오 D: 현행 유지 + 선별적 개선

외부 엔진 도입 없이 현행 구조에서 점진적 개선.

| 항목 | 평가 |
|------|------|
| **개선 항목** | (1) 내장 폰트 메트릭 확대 (2) WASM JS Canvas 측정 정확도 향상 (3) 줄바꿈 알고리즘 보강 (4) 편집 시 LINE_SEG 재계산 로직 강화 |
| **장점** | 기존 아키텍처/테스트 완전 보존. WASM 호환성 문제 없음. 점진적 진행 가능 |
| **리스크** | 근본적 한계(셰이핑 없음, 커닝 없음) 해소 불가. 폰트 메트릭 유지 비용 지속 증가. 편집 기능 고도화 시 동일 유형 버그 반복 |
| **결론** | **단기 유효**, 장기적으로는 시나리오 C로의 전환 필요 |

## 시나리오별 요약 비교

| 시나리오 | 셰이핑 | WASM | HWP 호환 | 난이도 | 권장 |
|----------|--------|------|----------|--------|------|
| A. cosmic-text 전면 | ◎ | △ | ✗ | 매우 높음 | ✗ |
| B. SkParagraph 전면 | ◎ | ✗ | ✗ | 극도로 높음 | ✗ |
| **C. 셰이핑만 도입** | **◎** | **○** | **◎** | **중간** | **◎** |
| D. 현행 유지+개선 | ✗ | ◎ | ◎ | 낮음 | △ (단기) |

## 최종 권장안

### 단기 (현재~): 시나리오 D — 현행 유지 + 선별적 개선

- 편집 시 LINE_SEG 재계산 로직의 버그 수정에 집중
- 내장 폰트 메트릭 보강
- 현재 발생하는 조판 버그들을 하나씩 해결

### 중기 (후속 타스크): 시나리오 C — harfrust 선별 도입 PoC

1. harfrust + skrifa(또는 swash) 단독 도입 가능성 PoC
2. TextMeasurer에 HarfrustMeasurer 구현체 추가
3. 네이티브 환경에서 기존 EmbeddedTextMeasurer와 A/B 비교
4. WASM 폰트 로딩 전략 수립 (번들 vs fetch vs JS Canvas 하이브리드)
5. 성공 시 점진적 전환

### 참고: parley 관찰

parley(v0.8.0)는 harfrust + ICU4X(줄바꿈/BiDi) + fontique(폰트 폴백) + skrifa(폰트 읽기)를 통합한 스택으로, cosmic-text의 대안이자 보다 모듈화된 구조를 제공한다. NLnet 지원으로 활발히 개발 중이며, 향후 안정화되면 시나리오 C의 구현 기반으로 재검토할 가치가 있다.
