---
문서: 외부 공개 문서 자기검열 감사 — 결정 요청 폼
작성일: 2026-04-22
기준: origin/main (폴라리스 일반화 커밋 93bf5f0 반영 후)
근거: 메모리 `feedback_external_docs_self_censor.md` 체크리스트
제외: 한컴 매뉴얼 (`mydocs/manual/hwp/Help/`) · 사용자 발언 인용 (`mydocs/report/x_hwp_viewer_voices.md`)
---

# 읽는 법

총 3개 카테고리 × 분류 (A/B/C) 에 따라 **기본 권고** 를 붙였습니다. 각 항목 옆 `**결정**: [ ]` 에 다음 중 하나를 기입해 주세요.

| 코드 | 의미 |
|---|---|
| **수정** | 제시된 대안 표현으로 변경 |
| **보류** | 현재 표현 유지 (논거가 있어 유지) |
| **삭제** | 해당 문장/절 자체 제거 |
| **자유** | 임의 지시 (예: "다른 표현으로: XXX") |

카테고리 맨 아래에 **전체 일괄 지시** (`**일괄 결정**: [ ]`) 도 가능합니다. 일괄 지시가 없으면 개별 결정만 반영됩니다.

---

# 카테고리 1 — 특정 상용 제품명 (사이냅소프트)

폴라리스 건과 같은 성격. 비교·경쟁 문맥에서 거명.

## 1.1 `mydocs/tech/webgian_replacement_strategy.md`

### [1.1.1] line 30 · 표 항목

```
| 사이냅 문서뷰어/에디터 | 사이냅소프트 | HWP 96~2024 뷰어, 문서 변환 서버, Document AI. 공공기관 뷰어 시장 점유. HWP 바이너리 프로그래매틱 생성 API 미제공. 서버 기반 SaaS 구조 |
```

**권고**: **수정** — "일부 상용 문서 뷰어/변환 솔루션" 으로 일반화
**결정**: [ 수정 ] "사이냅" 은 S사 로 문구 변경

### [1.1.2] line 46 · 결론 문장

```
- 사이냅소프트는 뷰어/변환에 강하나 HWP 바이너리 생성 API는 미제공
```

**권고**: **수정** — "일부 상용 문서 뷰어 솔루션은 뷰어/변환에 강하나 HWP 바이너리 생성 API는 미제공"
**결정**: [ 수정 ] "미제공" 을 "확인못함"

## 1.2 `mydocs/tech/project_vision.md`

### [1.2.1] line 83 · 생태계 표

```
| 사이냅소프트 | 문서 뷰어/변환/에디터 서버 | 뷰어/변환 특화, HWP 생성 API 미제공, 서버 의존 |
```

**권고**: **수정** — "일부 상용 문서 뷰어/변환 솔루션"
**결정**: [ 수정 ]

### [1.2.2] line 89 · 상세 설명 블록

```
- **사이냅소프트**: 사이냅 문서뷰어(HWP 96~2024 지원), 사이냅 에디터, 문서 변환 서버 등
  Document AI 솔루션 제공. 공공기관 문서 뷰어 시장 점유. 그러나 **뷰어/변환** 특화이며
  HWP 바이너리를 프로그래매틱하게 생성하는 API는 없음. 서버 기반 SaaS 구조.
```

**권고**: **수정** — 거명 없이 "일부 상용 문서 뷰어 솔루션" 으로 일반화
**결정**: [ 수정 ]

### [1.2.3] 영문판 `mydocs/eng/tech/webgian_replacement_strategy.md` line 30, 46, 113, 125

한글판과 1:1. 한글판 결정대로 따라감.
**결정**: [ 수정 ] (한글과 동일 적용)

### [1.2.4] 영문판 `mydocs/eng/tech/project_vision.md` line 81, 87, 222

한글판과 1:1.
**결정**: [ 수정 ] (한글과 동일 적용)

**일괄 결정** (카테고리 1 전체): [ ]

---

# 카테고리 2 — 최상급 주장 (단서 없는 "유일한" / "전무" / "세계 최초" / "최고")

## 2A. **반드시 재검토 권고** (외부 공개 강한 문서)

### [2A.1] `mydocs/tech/webgian_replacement_strategy.md:47`

```
- 오픈소스는 구조 파싱/텍스트 추출 수준이며, **편집+재조판+렌더링 구현체는 전무**
```

**권고**: **수정** — "공개된 범위에서 확인한 오픈소스 구현 중 편집+재조판+렌더링을 아우르는 사례는 드물다"
**결정**: [ 수정 ]

### [2A.2] `mydocs/tech/webgian_replacement_strategy.md:49`

```
- **WebAssembly 기반 클라이언트 전용(서버 불필요) HWP 저장은 세계 최초**
```

**권고**: **수정** — "우리가 확인한 범위 안에서 공개된 WebAssembly 기반 클라이언트 전용 HWP 저장 구현은 본 제품이 처음"
**결정**: [ 수정 ]

### [2A.3] `mydocs/tech/webgian_replacement_strategy.md:50`

```
- **AI Agent용 HWP 생성 도구는 전무** (상용/오픈소스 모두) — 유일한 포지션
```

**권고**: **수정** — "공개된 범위에서 AI Agent 대상 HWP 생성 도구는 드물다 — 차별화된 포지션"
**결정**: [수정 ]

### [2A.4] `mydocs/tech/webgian_replacement_strategy.md:86`

```
- 현재 AI가 HWP를 직접 생성할 수 있는 도구는 **전무**
```

**권고**: **수정** — "공개된 범위에서 확인된 도구는 거의 없음"
**결정**: [수정 ]

### [2A.5] `mydocs/tech/webgian_replacement_strategy.md:87`

```
Rust 라이브러리 + WASM = AI 에이전트가 호출 가능한 유일한 HWP 생성 도구
```

**권고**: **수정** — "Rust 라이브러리 + WASM 조합은 AI 에이전트가 직접 호출할 수 있는 드문 HWP 생성 도구"
**결정**: [수정 ]

### [2A.6] `mydocs/tech/webgian_replacement_strategy.md:142`

```
- **HWP 네이티브 호환** — 한컴 이외 유일한 HWP 네이티브 파싱/렌더링/편집 엔진
```

**권고**: **수정** — "한컴 외 HWP 바이너리를 원격 서버 없이 파싱/렌더링/편집하는 드문 엔진"
**결정**: [수정 ]

### [2A.7] `mydocs/tech/project_vision.md:60`

```
> **"AI가 쓴 글을 HWP로 완성하는 유일한 도구"**
```

**권고**: **수정** — "AI가 쓴 글을 HWP로 바로 완성하는 희소한 도구" 또는 슬로건 자체 재검토
**결정**: [수정 ]

### [2A.8] `mydocs/tech/project_vision.md:667`

```
- **hwp_semantic + rhwp**: HWP 읽기(시맨틱 파싱) + 쓰기(네이티브 생성) 양쪽을 모두 갖춘 유일한 생태계
```

**권고**: **수정** — "... 양쪽을 모두 갖춘 드문 생태계"
**결정**: [수정 ]

### [2A.9] `mydocs/tech/hwp_errata_public_comparison.md:325`

```
rhwp는 정밀 렌더링(...)과 에디터 기능(...)을 모두 지원하는 **유일한 오픈소스 HWP 뷰어/에디터**이다.
```

**권고**: **수정** — "...를 아우르는 드문 오픈소스 HWP 뷰어/에디터" + "우리가 확인한 범위에서" 단서
**결정**: [수정 ]

### [2A.10] `mydocs/tech/hwpx_dvc_reference.md:214`

```
- Rust 포팅판이 **CI 자동화 가능한 유일한 HWPX 검증 도구**가 됨
```

**권고**: **수정** — "크로스-OS CI 자동화가 용이한 드문 HWPX 검증 도구"
**결정**: [수정 ]

### [2A.11] `mydocs/manual/hyper_waterfall_docs_guide.md:17`

```
이것은 관료주의가 아니라, AI 페어프로그래밍에서 **품질을 보장하는 유일한 방법**이기 때문이다.
```

**권고**: **수정** — "품질을 지키기 위해 우리가 찾은 방법"
**결정**: [수정 ]

## 2B. **기술적 내부 문맥 (외부 노출 약함)**

### [2B.1] `mydocs/tech/vscode_extension_design.md:157`

```
- 유일한 공통 의존성: `pkg/` (WASM 빌드 출력)
```

**권고**: **보류** (코드 의존성 서술, 비교 문맥 아님)
**결정**: [유지 ]

### [2B.2] `mydocs/tech/hwpx_hancom_reference.md:123`

```
m_bSnapToGrid(true), // ← 유일한 true 기본값
```

**권고**: **보류** (C++ 구조체 필드 주석, 역공학 기록)
**결정**: [유지 ]

### [2B.3] `mydocs/tech/layout_engine_research.md:286`

```
- Phase 3에서 Paginator 표 로직 제거, TypesetEngine이 유일한 경로
```

**권고**: **보류** (코드 경로 서술)
**결정**: [ 유지]

### [2B.4] `mydocs/troubleshootings/table_paste_file_corruption.md:49, 72`

```
- 빈 문서에서는 MSB=1이 필수 (표 컨트롤이 유일한 콘텐츠)
표 문단이 섹션의 유일한(=마지막) 문단이었기 때문이다.
```

**권고**: **보류** (기술 상황 서술)
**결정**: [ 유지]

### [2B.5] `mydocs/working/archives/task_41_final.md:14`

```
- 빈 문서: MSB=1 (표가 유일한 콘텐츠)
```

**권고**: **보류** (과거 archive + 기술 서술)
**결정**: [유지 ]

### [2B.6] `mydocs/working/task_m100_178_stage4.md:29`

```
- **유일한 차이: 재로드 IR 의 PageDef 가 모두 0 ...**
```

**권고**: **보류** (디버깅 로그)
**결정**: [ 유지]

### [2B.7] `mydocs/tech/equation_font_selection.md:91, 149`

```
시각적 일관성 최고
| **CMU Serif** | 최고 | CM 원본. ... |
```

**권고**: **보류** (내부 표 평가, 기술 선택지 비교)
**결정**: [유지 ]

### [2B.8] `mydocs/tech/equation_latex_comparison.md:108`

```
| **명령어 자동완성** | LyX, VS Code + LaTeX | 숙련자에게 최고 속도 | 명령어 암기 필요 |
```

**권고**: **보류** (타제품 평가 표지만 사실 서술)
**결정**: [ 유지]

### [2B.9] `mydocs/plans/task_m100_76.md:12`

```
- rhwp의 최초 일반 사용자 대상 앱 → **사용자 경험이 최고 우선 목표**
```

**권고**: **보류** (우선순위 서술, 최상급 대상은 내부 목표)
**결정**: [유지 ]

### [2B.10] `mydocs/working/archives/task_41_final.md:23`

```
| FIX-1 | **최고** | 표 문단 char_count_msb = false | ...
```

**권고**: **보류** (우선순위 등급 표시)
**결정**: [ 유지]

### [2B.11] `mydocs/working/task_397_step1.md:121`

```
SkParagraph는 텍스트 셰이핑/줄바꿈 품질은 최고 수준이나, ...
```

**권고**: **보류** — 타 라이브러리 칭찬, 공격성 없음. 단 "업계 최상위 수준으로 평가됨" 정도 완화 고려
**결정**: [유지 ]

## 2C. **영문판 대응**

카테고리 2A/2B 의 영문판은 한글판 결정에 따라 동시 변경.

**결정**: [수정 ] (한글 결정 따라감)

**일괄 결정** (카테고리 2 전체): [ ]

---

# 카테고리 3 — "100% 호환" / "한컴 수준" / "한컴 동일"

## 3A. **재검토 권고** (외부 공개 강한 문서)

### [3A.1] `mydocs/manual/branding_strategy.md:115`

```
기존 오픈소스 HWP 도구들은 정확도가 낮고, 상용 뷰어는 비싸다. rhwp는 한컴 수준의 정확도를 무료로 제공한다.
```

**권고**: **수정** — "상용 뷰어에 근접한 정확도", "기존 오픈소스 도구와 차이가 크다" 식으로 완화 + "기존 오픈소스는 정확도가 낮다" 는 정중한 표현으로
**결정**: [ 수정] "상용 뷰어에 근접한 정확도"

### [3A.2] 영문판 `mydocs/eng/manual/branding_strategy.md:115`

```
rhwp delivers Hancom-level accuracy for free.
```

**권고**: **수정** — "rhwp delivers accuracy approaching Hancom's, for free"
**결정**: [ 수정]

## 3B. **내부 목표·계획서 (허용)**

아래는 타스크 계획서·구현 목표 서술로, 외부 홍보 문구가 아니므로 보류 권장:

- `mydocs/tech/incremental_relayout_design.md:144, 148, 155` — v1.0.0 목표 전략 서술
- `mydocs/plans/task_246.md:1` — "한컴 수준 도형 완성" (타스크 제목)
- `mydocs/plans/archives/task_191.md:83` — 한컴 수준 고도화 목표
- `mydocs/plans/archives/task_199.md, task_199_impl.md, task_242.md` — 타스크 제목
- `mydocs/plans/task_264.md:18` — "한컴 동일 글머리표"
- `mydocs/plans/task_398.md:9` — v1.0.0 전략 서술
- `mydocs/orders/20260328.md, 20260318.md, 20260223.md, 20260311.md, 20260317.md` — 일일 기록
- `mydocs/working/task_241_stage2.md:32` — 책갈피 서식 서술
- `mydocs/working/archives/task_214_final.md:74` — "100% 호환" 은 코드 호환 서술
- 영문판 `mydocs/eng/plans/archives/task_194.md:5`, `mydocs/eng/tech/incremental_relayout_design.md:155` — 동일

**권고**: **보류** (내부 계획·목표 서술)
**결정**: [보류 ]

**일괄 결정** (카테고리 3 전체): [ ]

---

# 결정 후 절차

작업지시자 결정 완료 후:
1. 수정 건 일괄 반영 (main 직접 커밋)
2. origin/main push
3. devel cherry-pick + push
4. 대략적 규모: 2A·1 전체 수정 시 약 20 파일 변경 (한글 + 영문)

한 번에 답변 가능 (예: "1 일괄 수정, 2A 전체 수정, 2B 전체 보류, 3A 수정, 3B 보류"). 개별 미세 조정도 가능.
