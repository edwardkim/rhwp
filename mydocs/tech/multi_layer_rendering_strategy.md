# 다층 레이어 렌더링 아키텍처 — 기술 조사 보고서

**작성**: 2026-05-02 (Task #516 Stage 5+ 영역 확장 검토)
**상태**: 작업지시자 검토 / 후보 선택 대기
**관련 이슈**: [#516](https://github.com/edwardkim/rhwp/issues/516) (web Canvas 워터마크), [#514](https://github.com/edwardkim/rhwp/issues/514) (BehindText 출력 누락 — 완료), [#515](https://github.com/edwardkim/rhwp/issues/515) (s0:pi=16 표 좌표)

---

## 1. 결정 배경

Task #516 의 Stage 4 까지 정정 (CSS filter 적용) 후 작업지시자 시각 판정에서 3 결함이 식별:

| # | 결함 | 본질 |
|---|------|------|
| 1 | 엠블럼 (JPEG) 의 흰색 배경 투명 처리 실패 | JPEG 알파 채널 부재 + 단일 평면 렌더링 한계 |
| 2 | BehindText 그림 위에 텍스트 클릭 불가 (이미지 hit) | 단일 Canvas 평면의 hit-test 한계 |
| 3 | 워터마크 효과 미적용 | 회색조/bc 만 적용, blend (multiply 등) 미적용 |

세 결함의 공통 근본은 **rhwp-studio 의 web 렌더가 단일 Canvas 2D 평면** 이라는 구조적 제약. HWP 의 z-order 모델 (BehindText / InFrontOfText / 어울림 / 위아래) 을 단일 픽셀 평면에 평면화하면서 wrap 모드의 의미가 손실.

작업지시자 결정: **다층 레이어 아키텍처 도입**. 단순한 2-3 층 분리가 아니라 **다층 확장 가능 (multi-layer extensible) + GPU 가속 활용** 형태로 설계.

본 보고서는 후보 3 안을 비교하여 작업지시자가 선택할 수 있도록 정리한다.

---

## 2. 평가 기준

| 기준 | 가중치 | 설명 |
|------|-------|------|
| **다층 확장성** | 🟥 핵심 | BehindText/InFrontOfText 외 future wrap 모드, comments overlay, ruler 등 추가 가능 |
| **GPU 가속** | 🟥 핵심 | 특히 워터마크 / 그림 효과 / 페이지 전환 시 |
| **HWP 의미 정합** | 🟧 높음 | wrap 모드 (BehindText / InFrontOfText / 어울림) z-order 자연 매핑 |
| **hit-test 정확성** | 🟧 높음 | 텍스트 위 클릭 / 이미지 위 클릭 분리 |
| **blend mode 지원** | 🟧 높음 | multiply / screen / overlay (워터마크 / 마커 / 형광펜) |
| **DPR / 줌 / 인쇄 정합** | 🟧 높음 | High-DPI 화면 + 100% ~ 300% 줌 + 브라우저 인쇄 |
| **구현 비용** | 🟨 중간 | 도입 + 회귀 점검 비용 |
| **dependency 영향** | 🟨 중간 | binary 크기, 신규 crate / npm package |
| **기존 인프라 정합** | 🟨 중간 | PageLayerTree (이미 존재), Canvas visual diff 인프라 (PR #498) |
| **브라우저 호환** | 🟨 중간 | Chrome / Firefox / Safari / Edge 모두 |

---

## 3. 후보 3 안

### 후보 A — Multi-Canvas Layered Composition (HTMLCanvasElement × N)

**구조:**
```
<div class="page-container">
  <canvas class="layer-background"></canvas>      <!-- 페이지 배경, 머리말/꼬리말 -->
  <canvas class="layer-behind-text"></canvas>     <!-- BehindText 그림 (워터마크 등) -->
  <canvas class="layer-flow"></canvas>             <!-- 본문 텍스트 + 표 + 어울림 그림 -->
  <canvas class="layer-in-front"></canvas>        <!-- InFrontOfText 그림 (직인) -->
  <canvas class="layer-annotations"></canvas>     <!-- 선택 영역, 주석, 검색 하이라이트 -->
  <div class="hit-test-layer"></div>               <!-- 투명 DOM 오버레이 (선택 사항) -->
</div>
```

**렌더링 경로:**
- 각 Canvas 가 z-index 로 적층, CSS `mix-blend-mode` 가능
- BehindText 그림 Canvas 에 워터마크 그릴 때 `globalCompositeOperation = 'multiply'` 또는 CSS `mix-blend-mode: multiply`
- 본문 Canvas 의 텍스트는 자기 평면이라 hit-test 정상
- pointer-events 로 hit-test 우선순위 제어

**GPU 가속:**
- Canvas 2D 자체는 브라우저 구현 (Skia/Direct2D 등) 의 GPU 가속에 의존 — 직접 제어 불가
- `mix-blend-mode` 는 GPU compositor 단계에서 처리 (Chrome Compositor)
- DPR scaling 은 각 Canvas 의 `width = page_w * scale` 로 자동
- 다중 Canvas 합성은 브라우저 compositor 가 GPU 로 처리 (single-pass)

**다층 확장:**
- ✅ Layer 추가가 단순 (DOM `<canvas>` 추가)
- 각 wrap 모드가 자기 Canvas 를 가지므로 future wrap 모드 (예: HWP 의 `behind-table`) 추가 시 신규 Canvas 만 추가
- z-index 정수로 명시적 z-order

**구현 영향:**

| 영역 | 영향 |
|------|------|
| Rust (rhwp lib) | PageLayerTree 의 LayerNode 트리에 wrap 모드별 분류 추가 (이미 wrap 정보는 있음, 실제 layer 분리 정책만 추가) |
| WASM API | `render_page_to_layered_canvas(page, &[Canvas; N], wrap_modes)` 같은 N-Canvas 인자 메서드 추가. 또는 `render_layer(canvas, layer_kind)` 호출을 N 번 호출 |
| rhwp-studio | page-renderer 의 단일 Canvas → 다중 Canvas DOM 생성 로직, hit-test 정책 |
| Canvas visual diff (PR #498) | 영향 — 합성 결과를 단일 Canvas 로 합쳐서 diff 하거나 layer 별 diff 분리 (별도 결정) |

**장점:**
- 🟢 가장 단순, Canvas 2D 표준만 사용
- 🟢 pointer-events / mix-blend-mode 모든 브라우저 호환
- 🟢 print 시 브라우저가 z-index 알아서 처리
- 🟢 PageLayerTree 의 기존 Layer 모델과 직접 매핑

**단점:**
- 🟧 Canvas 메모리 N 배 (page_w × page_h × 4 byte × N layers × DPR²)
  - 예: A4 (794×1123 px) × 3 layers × DPR 2² = ~10.7 MB / 페이지
  - 다중 페이지 시 누적
- 🟨 layer 간 동기화 복잡 (페이지 줌 시 N Canvas 모두 resize)
- 🟨 Canvas 2D 자체의 효과 (filter / blend) 제한 — `ctx.filter` 만으로 본 task 같은 워터마크 처리 가능하나 복잡한 효과는 한계

**GPU 가속 정도**: 🟨 중간 (브라우저 compositor 의존, 직접 제어 X)

---

### 후보 B — WebGPU Single-Texture Compositor (Custom GPU Renderer)

**구조:**
```
<canvas class="page" data-context="webgpu"></canvas>
```
단일 `<canvas>` + WebGPU context. 내부적으로 layer 별 texture 를 GPU 메모리에 보관하고 GPU shader 로 합성.

**렌더링 경로:**
- 각 layer (BehindText / Text / InFrontOfText) 를 GPU texture 로 렌더링
- `GPUDevice` 의 fragment shader 로 layer 합성 (blend mode + alpha)
- 텍스트 / 표 / 그림 모두 vertex/index buffer 로 GPU 전송
- hit-test 는 별도 CPU 측 R-tree / quadtree 로 처리 (GPU 경로와 분리)

**GPU 가속:**
- 🟥 가장 직접적 — vertex/fragment shader 로 모든 합성
- DPR scaling 자유롭게 (texture 크기 = render target 크기)
- blend mode 는 shader 에서 직접 정의 (multiply / screen / overlay 등 모든 모드)
- 페이지 전환 시 GPU texture swap 으로 빠름

**다층 확장:**
- ✅ Layer 추가가 GPU texture 추가 + shader 합성 추가
- 무제한 layer 가능 (GPU 메모리 한계 내)
- 임의의 z-order + blend mode 조합 가능

**구현 영향:**

| 영역 | 영향 |
|------|------|
| Rust (rhwp lib) | WebGPU 바인딩 (wgpu crate) 또는 shader 작성 (WGSL). PageLayerTree → GPU vertex buffer 변환 |
| WASM API | `render_page_to_webgpu(canvas, ...)` 신규 메서드 + WebGPU context 관리 |
| rhwp-studio | WebGPU support 점검 (Chrome 113+, Firefox 141+, Safari 26+) + 폴백 정책 (지원 안 하는 브라우저는 후보 A 폴백) |
| 텍스트 렌더링 | **MSDF/SDF 폰트 또는 별도 SDF 렌더러** 필요 — Canvas 2D 의 fillText 못 씀. 또는 텍스트만 별도 canvas 에서 렌더하고 texture 화 |

**장점:**
- 🟢 가장 빠른 GPU 가속 (직접 shader 제어)
- 🟢 무제한 layer + 자유로운 blend
- 🟢 줌/회전/transform 이 GPU matrix 로 즉시 처리
- 🟢 future-proof (WebGPU 가 Web 표준의 다음 세대)

**단점:**
- 🟥 **구현 비용 매우 높음** — 텍스트 렌더러 자체 작성 필요 (Canvas 2D fillText 사용 불가)
- 🟥 WebGPU 미지원 브라우저 (Safari 26 미만, 일부 모바일) — 폴백 필수
- 🟥 wgpu crate WASM 빌드 + binary 크기 증가 (~수 MB)
- 🟥 PageLayerTree 의 paint op (text/path/image) 를 모두 GPU buffer 로 변환하는 시스템 필요 (큰 추가 인프라)
- 🟧 메모리 (GPU texture 다수, page 별)
- 🟧 디버깅 / 시각 회귀 검증 인프라 (현재 svg_snapshot / Canvas visual diff) 부적합

**GPU 가속 정도**: 🟥 매우 높음 (직접)

---

### 후보 C — Hybrid: HTML Overlay + Canvas + CSS Compositor (Browser-Native)

**구조:**
```
<div class="page-container">
  <canvas class="layer-flow"></canvas>             <!-- 본문 텍스트 + 표 + 어울림 그림 (Canvas 2D) -->
  <div class="overlay-behind">                      <!-- BehindText 그림 -->
    <img src="data:..." style="
      position:absolute;
      filter:grayscale(1) brightness(.5) contrast(1.7);
      mix-blend-mode:multiply;
      pointer-events:none;" />
  </div>
  <div class="overlay-front">                       <!-- InFrontOfText 그림 -->
    <img src="data:..." style="position:absolute;" />
  </div>
  <svg class="layer-annotations"></svg>            <!-- 선택, 주석 (SVG) -->
</div>
```

**렌더링 경로:**
- 본문 (텍스트 + 표 + 어울림 그림) 은 Canvas 2D — 기존 코드 재사용
- BehindText / InFrontOfText 그림은 HTML `<img>` 또는 SVG `<image>` 로 분리 — DOM 레벨 layer
- Annotations (선택 / 검색 / 주석) 은 SVG layer
- 각 layer 가 자기 매체 (Canvas / DOM img / SVG) 의 native 처리 활용

**GPU 가속:**
- HTML img / SVG 의 transform / filter / mix-blend-mode 는 브라우저 GPU compositor 가 자동 처리
- Chrome 의 경우 layer promotion (`will-change: transform` 또는 transform 적용 시) → 별도 GPU layer 로 합성 → multi-layer 합성이 GPU 단계에서 single-pass
- Canvas 2D 는 그대로 (Skia GPU 가속, 브라우저 의존)

**다층 확장:**
- ✅ Layer 추가가 가장 자연스러움 — DOM 트리에 element 추가
- 각 layer 의 매체를 자유롭게 선택 (Canvas / img / SVG / video / iframe)
- z-index + position 으로 자연 z-order
- future overlay (검색 하이라이트, 주석, 협업 커서 등) 추가 즉시 가능

**구현 영향:**

| 영역 | 영향 |
|------|------|
| Rust (rhwp lib) | PageLayerTree 의 BehindText/InFrontOfText 그림을 별도 노드로 분리 (이미 wrap 정보 있음) |
| WASM API | `render_page_layered(page, container_div) -> { mainCanvas, behindImages, frontImages }` 같은 다중 매체 출력. 또는 별개 함수 (`get_behind_images(page)`, `render_main_canvas(page, canvas)`) |
| rhwp-studio | page-renderer 가 단일 Canvas → 다중 element 컨테이너 생성. 그림 export 시 base64 data URL 로 `<img>` 생성 |
| 그림 효과 적용 | CSS `filter` (Stage 4 정정 결과 와 동일 매핑) — 다만 `<img>` 에 적용되어 효과가 더 정확 (브라우저 GPU 처리) |

**장점:**
- 🟢 **HWP wrap 모델과 가장 자연스러운 매핑** — BehindText/InFrontOfText 가 native HTML 레이어
- 🟢 hit-test 자연스러움 — pointer-events: none 으로 그림 layer 무시 → 텍스트 클릭 정상
- 🟢 mix-blend-mode 표준 — multiply / screen / overlay 모두 가능
- 🟢 GPU 가속이 자동 (브라우저 compositor)
- 🟢 메모리 효율 — Canvas 1개 + img element (브라우저 native 디코딩 + 캐싱)
- 🟢 print 시 브라우저가 native 처리 (모든 layer 인쇄)
- 🟢 디버깅 / 회귀 검증 인프라 (Canvas visual diff PR #498) 와 정합 — 본문 Canvas 는 그대로
- 🟢 ios/Android Safari 호환 (Skia/WebGPU 폴백 필요 없음)

**단점:**
- 🟨 DOM 노드 증가 (그림 수만큼 `<img>`)
- 🟨 layer 간 픽셀 정확 정렬 (Canvas 픽셀 ↔ DOM rem/px) 시 1px 미세 차이 위험 — 동일 좌표계 사용으로 차단
- 🟨 현재 PageLayerTree paint op 와 다른 형태 (paint op 는 모두 Canvas 그리기로 간주) — 분리 정책 추가 필요
- 🟢 → 🟨 GPU 가속 직접 제어 불가 (후보 B 와 비교 시) — 그러나 본 사용 사례에 필요한 수준은 충분

**GPU 가속 정도**: 🟧 높음 (브라우저 compositor 자동, 직접 제어 X)

---

## 4. 후보 비교 매트릭스

| 평가 기준 | A: Multi-Canvas | B: WebGPU | C: HTML Hybrid |
|-----------|----------------|-----------|----------------|
| 다층 확장성 | 🟧 좋음 (Canvas 추가) | 🟢 매우 좋음 (texture 무제한) | 🟢 매우 좋음 (DOM 자유) |
| GPU 가속 | 🟨 브라우저 compositor 의존 | 🟢 직접 (shader) | 🟧 브라우저 compositor 자동 |
| HWP 의미 정합 (wrap 모드) | 🟧 좋음 (1 wrap = 1 canvas) | 🟧 좋음 (texture 매핑) | 🟢 매우 좋음 (DOM 레이어 = wrap) |
| hit-test 정확성 | 🟧 pointer-events 로 가능 | 🟨 별도 CPU 인덱스 필요 | 🟢 pointer-events 자연스러움 |
| blend mode | 🟧 mix-blend-mode 가능 | 🟢 shader 로 자유 | 🟢 mix-blend-mode 자연스러움 |
| DPR / 줌 / 인쇄 | 🟧 N Canvas 동기화 | 🟧 GPU texture 재할당 | 🟢 DOM 자동 |
| 구현 비용 | 🟧 중간 (1-2 주) | 🟥 매우 높음 (1-2 개월) | 🟧 중간-낮음 (1 주) |
| dependency | 🟢 표준 | 🟥 wgpu (binary +수 MB) | 🟢 표준 |
| 기존 인프라 정합 | 🟧 PageLayerTree 활용 가능 | 🟨 paint op 변환 시스템 필요 | 🟢 PageLayerTree 자연스러움 |
| 브라우저 호환 | 🟢 모든 모던 브라우저 | 🟨 Safari 26 미만 폴백 필요 | 🟢 모든 모던 브라우저 |
| 메모리 | 🟧 N 배 Canvas | 🟧 GPU texture | 🟢 효율 (img native 캐싱) |
| Canvas visual diff (PR #498) 정합 | 🟧 합성 후 diff 또는 layer diff | 🟨 별도 캡처 정책 필요 | 🟢 본문 Canvas 그대로 |
| **워드프로세서 사용 사례** (M100) | 🟧 충분 | 🟥 과다 (비용 대비 효과 부족) | 🟢 충분 + 즉각 적용 |
| **DTP 사용 사례** (M200+ Appendix 참조) | 🟨 한계 (메모리 누적) | 🟢 **이상적** (인쇄/협업/대용량) | 🟧 좋음, 인쇄 품질에서 GPU 직접 제어 부족 |
| **종합 (M100 일정 우선)** | 🟧 **준수** | 🟥 **이상적이나 비용 과대** | 🟢 **가장 균형** |
| **종합 (M200+ DTP 정체성)** | 🟨 한계 | 🟢 **이상적** | 🟧 좋음 (점진적 B 마이그레이션 토대) |

---

## 5. 권장안 + 결정 시나리오

### 시나리오 1 — **현재 우선순위 (M100 v1.0.0, 이슈 #516 본 task) 일정 고려**
권장: **후보 C (HTML Hybrid)**

근거:
- 본 task 의 결함 3건 모두 즉시 해결
- 구현 비용 1 주 내 가능
- HWP wrap 모델과 가장 자연스러운 매핑
- 기존 PageLayerTree 인프라 활용
- 브라우저 호환 / 인쇄 / DPR 모두 자동
- GPU 가속은 브라우저 compositor 가 자동 — 본 사용 사례에 충분

### 시나리오 2 — **장기 GPU 가속 본격화 (M200 v2.0.0 이상 협업/실시간)**
권장: **후보 B (WebGPU)** 단계적 도입

근거:
- 협업 편집 / 실시간 리렌더링 / 대용량 문서 (수천 페이지) 시 GPU 직접 제어 가치 발생
- 현재 단계 (M100) 에서는 비용 대비 효과 부족, 그러나 v2.0.0+ 에서는 필요
- WebGPU 표준 정착 후 (2026~2028) 도입 가능
- 후보 C 의 Canvas 부분만 단계적으로 WebGPU 로 마이그레이션 가능 (단기 후보 C → 장기 후보 B)

### 시나리오 3 — **단순 빠른 정정만 필요 (본 task 의 3 결함 만)**
권장: **후보 A (Multi-Canvas)**

근거:
- 가장 익숙한 Canvas 2D 만 사용
- 구현이 가장 단순
- 단 다층 확장성 / GPU 가속 / hit-test 정확성에서 후보 C 보다 약함

### 통합 권장: **후보 C → 후보 B 단계적 마이그레이션 경로**

본 보고서의 권장 경로 (작업지시자 확정 — 2026-05-02):

1. **단기 (M100 v1.0.0, 본 task #516 의 Stage 5+)**: **후보 C 도입 ✅ 확정**
   - HTML overlay (`<img>` for BehindText / InFrontOfText)
   - 본문 Canvas 유지
   - 결함 1, 2, 3 모두 해결
   - PageLayerTree 의 wrap 모드별 layer 분리 정책 추가
   - **현재 단계에서 채택**

2. **중기 (M150~ v1.5.0)**: 후보 C 의 본문 Canvas 를 Canvas 2D + OffscreenCanvas + WebWorker 로 확장
   - 메인 thread 부하 분산
   - GPU 가속은 여전히 브라우저 compositor 의존

3. **장기 (M200 v2.0.0, DTP 정체성 본격화)**: **후보 B (WebGPU) 도입 ✅ 확정**
   - **DTP (Desktop Publishing) 도구로의 정체성 강화** (Appendix 참조)
   - 본문 Canvas → WebGPU (텍스트 SDF 렌더러 도입)
   - HTML overlay 는 보존 (DOM 레이어가 더 자연스러운 영역)
   - 폴백 경로: 후보 C 유지 (브라우저 호환)

---

## 6. 본 task #516 의 적용 결정 (확정)

**작업지시자 확정 (2026-05-02): 옵션 2 — 본 task 의 Stage 5+ 로 후보 C 직접 도입**

| 옵션 | 본 task #516 처리 | 다층 레이어 적용 task |
|------|-------------------|---------------------|
| 옵션 1 | Stage 4 까지로 마무리 (CSS filter + 헬퍼 + dump + JSON) | 신규 task 등록 — 다층 레이어 + 후보 C 도입 |
| **옵션 2 ✅** | **본 task 의 Stage 5+ 로 후보 C 직접 도입** | **본 task 가 다층 레이어 도입 첫 사이클** |
| 옵션 3 | Stage 4 까지로 마무리 + 별도 선행 task (다층 레이어 인프라) | 인프라 우선 |

**확정 근거:**
- DTP 정체성 관점에서 다층 레이어는 단순 정정이 아니라 **rhwp 의 정체성 인프라** (Appendix 참조)
- 본 task #516 의 결함 1/2/3 (배경 투명/hit-test/multiply blend) 이 후보 C 도입의 자연스러운 첫 사이클이 됨
- M200 (v2.0.0) 의 후보 B 마이그레이션 경로 보존 — 후보 C 의 본문 Canvas 분리가 향후 WebGPU 전환의 디딤돌

---

## 7. 메모리 / 의존성 / 외부 PR 관계

- 메모리 `feedback_v076_regression_origin` — 다층 레이어 도입은 다수 fixture 영향 가능. 작업지시자 시각 판정 게이트 필수
- 메모리 `reference_authoritative_hancom` — 한컴 2010 + 2022 가 정답지
- PR #456 (PageLayerTree replay 전환, @seo-rii) — 이미 Layer 트리 인프라 구축. 본 보고서의 후보 C/A 활용 가능
- PR #498 (Canvas visual diff 파이프라인, @seo-rii) — 다층 레이어 도입 후 별도 합성 / 비교 정책 필요
- 이슈 #514 (PCX) — 완료. 다층 레이어 도입 후 BehindText 알파 채널 처리가 더 자연스럽게 됨
- 이슈 #515 (s0:pi=16 표 좌표) — 별개 결함, 본 보고서와 무관

---

## 8. 다음 단계 (작업지시자 확정 후)

1. **본 task #516 의 Stage 5+ 로 후보 C 도입** — 옵션 2 확정
2. 후보 C 상세 구현 계획서 작성 (Stage 5+ 의 sub-stages 분리)
3. M200 (v2.0.0) DTP 정체성 task 의 후속 등록 (별도 사이클, 후보 B WebGPU)

---

## Appendix — 역사적 맥락: 다층 레이어가 rhwp 의 정체성 인프라인 이유

### A.1 아래아한글의 출시 시점 정체성

**아래아한글 (HWP) 1.0**: 1989년 출시. 동시대 출판/조판 도구:

| 도구 | 출시 / 정체성 | 시장 |
|------|-------------|------|
| **QuarkXPress** 1.0 | 1987 (Mac) → 3.0 (1992) DTP 표준 점유 | 잡지/책/광고/카탈로그 출판 |
| Aldus PageMaker | 1985 | DTP 양대산맥 |
| MS Word for Mac / Windows | 1985 / 1989 | 일반 워드프로세서 |
| **HWP 1.0** | 1989 (한컴) | **DTP + 워드프로세서** |

작업지시자 통찰 (2026-05-02):

> 아래아한글은 개발 목적이 쿽 익스프레스를 대체하는 것이었으니, 사실 엄밀히 따지면 '워드프로세서' 만을 충족하기 위한 프로젝트가 아니었다.

이는 HWP 의 다음 특성에 흔적이 남는다:

| HWP 특성 | DTP (QuarkXPress) 정합 |
|---------|----------------------|
| 글상자 (TextBox) | text frame 직역 |
| 개체 자유 배치 (BehindText / InFrontOfText / 어울림) | box 자유 배치 + z-order |
| 누름틀 / 양식 / 차례 / 색인 | 구조화 문서 + DTP 적 자동화 |
| 단(column) 정의 + 절(section) 분리 | 잡지/책 다단 조판 |
| 페이지 분할 정밀 제어 | 인쇄용 양식 / 회보 / 보고서 |
| 머리말/꼬리말/바탕쪽 | DTP 의 마스터 페이지 원형 |

### A.2 rhwp 의 정체성 재해석

rhwp 는 단순 "HWP 뷰어/에디터" 가 아니라 **"한국형 DTP 엔진의 오픈소스 재현 + 워드프로세서 기능"** 의 통합 도구. 이는 rhwp 의 장기 비전 (M200+ v2.0.0) 에서 다음 영역을 자연스럽게 포섭:

- **마스터 페이지 + 도큐먼트 레이어** (잡지/책/회보 조판)
- **인쇄용 layer** (Trapping / Bleed / Crop marks)
- **색공간 layer** (sRGB → CMYK / Pantone, 출판 인쇄 호환)
- **Linked text frames** (텍스트 흐름 박스 연결)
- **Picture box 자유 변형** (자르기 + 회전 + 효과)
- **협업 편집 layer** (실시간 커서 / 코멘트 / 변경 추적)
- **출판 전용 검토 layer** (Preflight / 색공간 검증)

### A.3 다층 레이어가 핵심 인프라인 이유

위 영역들은 **모두 layer 모델 위에 자연스럽게 올라가는 기능**. 단일 평면 Canvas 는 이를 표현 불가. 따라서:

- **단기 (M100)**: 후보 C 도입 — BehindText/InFrontOfText 의 자연스러운 처리. 본 task #516 의 결함 1/2/3 해결과 동시에 인프라 토대 마련
- **중기 (M150)**: OffscreenCanvas / WebWorker 로 부하 분산 (협업 편집의 사전 준비)
- **장기 (M200 v2.0.0)**: 후보 B (WebGPU) 도입 — DTP 정체성 본격화. SDF 텍스트 + GPU shader 합성 + 색공간 변환 + Pantone / CMYK 처리

### A.4 단순 워드프로세서 도구와의 차별화

후보 B (WebGPU) 의 비용 (1-2 개월 구현 + 텍스트 렌더러 자체 작성) 은 **워드프로세서 한정 사용 사례에서는 합리화 부족**. 그러나 **DTP 정체성 본격화 시점 (M200)** 에서는:

- 출판 품질 (subpixel 정확도, 색공간 정확성, 인쇄 호환)
- 실시간 협업 (대용량 페이지의 즉시 리렌더링)
- 미래 기능 (AI 기반 레이아웃 / 출판 자동화)

위 가치 영역에서 GPU 직접 제어가 핵심 자산이 됨. 후보 B 는 비용 대비 효과가 비로소 합리화됨.

### A.5 정리

| 단계 | 정체성 | 후보 |
|------|--------|------|
| M100 v1.0.0 (현재) | 워드프로세서 + DTP 인프라 토대 | **후보 C** ✅ |
| M150 v1.5.0 | 협업 편집 사전 준비 | 후보 C + OffscreenCanvas |
| **M200 v2.0.0** | **DTP 정체성 본격화** | **후보 B (WebGPU)** ✅ |
| M300+ | 한컴과 대등 + 공공 자산 | 후보 B 완성 + Pantone/CMYK |

이 단계적 경로는 **후보 C 가 후보 B 의 디딤돌** 이라는 점에서 정합. 본 보고서의 통합 권장 경로는 단순 비용 회피가 아니라 **rhwp 의 본질적 정체성을 단계적으로 실현하는 길**.

---
