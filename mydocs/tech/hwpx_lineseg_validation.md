---
kind: decision
status: active
canonical: mydocs/tech/document_ir_lineseg_standard.md
last_verified: 2026-07-17
---

# HWPX lineseg 비표준 감지 · 고지 · 보정 (#177)

- **작성일**: 2026-04-18
- **관련 이슈**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **관련 Discussion**: [#188](https://github.com/edwardkim/rhwp/discussions/188), [#184](https://github.com/edwardkim/rhwp/discussions/184)

## 1. 배경

HWPX의 `<hp:linesegarray>` 요소는 문단의 각 줄 좌표(`vertpos`, `vertsize`, `textheight`, `baseline`, `spacing`, `horzsize`, `flags`)를 담는다. OWPML 명세상 **각 줄마다 별도 `<hp:lineseg>`** 가 있어야 한다.

### 관찰된 비표준 패턴

rhwp 개발 중 다음 현상이 확인되었다:

```
한컴 HWPX (원본) ─▶ rhwp 로 열기 ─▶ (편집 없음) ─▶ rhwp 로 저장
                                                      │
                              ┌───────────────────────┤
                              ▼                       ▼
                    한컴에서 열기              rhwp로 다시 열기
                    ✅ 줄 넘김 정상           ❌ 한 줄에 겹쳐 조판
```

**원인 분석** (실측 기반):

- 한컴 HWPX 일부 버전은 **긴 문단도 `<hp:linesegarray>` 안에 lineseg 1개만** 넣는다
- 한컴 렌더러는 이 1개 lineseg 를 무시하고 **textRun 단위 reflow** 로 여러 줄 생성
- rhwp 렌더러는 명세를 신뢰하여 1개 lineseg 대로 한 줄에 모든 텍스트 배치 → 겹침

이는 한컴이 자기 파일 호환성을 위해 **방어적 reflow 로직**을 내장한 결과이며, 외부에 공개되지 않은 **숨겨진 기술부채**다. Discussion #188 에 상세 분석.

### rhwp 의 대응 원칙

Discussion #188 에서 천명한 대로:

1. **표준 준수 입력은 정확히 렌더** — 명세를 정답으로 취급
2. **비표준 입력은 감지하고 사용자에게 고지** — 조용한 보정 거부
3. **자동 보정은 사용자 명시 선택 후에만** — 기본 선택은 권장안(자동 보정)이나 선택 자체는 명시적으로 요청
4. **rhwp 자신도 비표준을 새로 생산하지 않음** — Serializer 는 원본 lineseg 를 그대로 보존

## 2. 감지 규칙

`DocumentCore::validate_linesegs()` 가 `DocumentCore::from_bytes` 시점에 자동 실행되며, `reflow_zero_height_paragraphs` 호출 **이전**에 원시 IR을 기준으로 검증한다. 결과는 `DocumentCore::validation_report` 에 저장된다.

### R1: `LinesegArrayEmpty`

```
조건: !text.is_empty() && line_segs.is_empty()
메시지: "lineseg 배열이 비어있음"
```

텍스트가 있는데 lineseg 가 전혀 없는 경우. 파서가 `<hp:linesegarray/>` 를 빈 요소로 읽었거나, 아예 누락된 경우.

### R2: `LinesegUncomputed`

```
조건: line_segs.len() == 1 && line_segs[0].line_height == 0
메시지: "lineseg 가 미계산 상태 (line_height=0)"
```

기존 `needs_line_seg_reflow` 와 동일 조건. 한컴이 빈 lineseg 뼈대만 넣은 경우.

### R3: `LinesegTextRunReflow`

```
조건: line_segs.len() == 1
      && !text.contains('\n')
      && text.chars().count() > 40
메시지: "lineseg 가 문단당 1개 (한컴 textRun reflow 의존)"
```

**가장 빈번하고 중요한 규칙**. 한컴 HWPX의 긴 문단이 lineseg 1개로만 선언된 경우. rhwp 가 그대로 렌더하면 겹침.

휴리스틱 threshold 40자는 한글 한 줄 너비(~30자)에 여유를 두어 false positive 를 줄인다.

## 3. 실측 false positive 측정

`tests/hwpx_roundtrip_integration.rs::task177_false_positive_measurement` 로 9개 샘플에 대해 측정한 결과 (2026-04-18):

| 샘플 | 총 경고 | LinesegArrayEmpty | LinesegUncomputed | LinesegTextRunReflow |
|---|---:|---:|---:|---:|
| blank_hwpx | 0 | 0 | 0 | 0 |
| ref_empty | 0 | 0 | 0 | 0 |
| ref_text | 0 | 0 | 0 | 0 |
| ref_table | 0 | 0 | 0 | 0 |
| ref_mixed | 0 | 0 | 0 | 0 |
| hwpx-02 | 15 | 0 | 0 | 15 |
| form-002 | 53 | 0 | 0 | 53 |
| 2025-q1 | 4 | 0 | 0 | 4 |
| 2025-q2 | 3 | 0 | 0 | 3 |

### 해석

- **레퍼런스 샘플 5건 모두 0건** — 단순 문서에서 false positive 없음
- **겹침 재현 파일 hwpx-02 에서 15건 감지** — 실제 문제 케이스 식별
- **대형 실문서에서 소수 감지** — 긴 요약 문단 일부만 해당
- **form-002 에서 53건** — 양식 컨트롤 문서 특성상 긴 안내 문단이 많음

R3 휴리스틱이 실제 문제 케이스를 잘 식별하면서 단순 문서에 오탐이 없음을 실측으로 확인.

## 4. 고지 플로우

### rhwp 엔진 단계

1. `DocumentCore::from_bytes(bytes)` 호출 시 IR 파싱 직후 `validate_linesegs` 자동 실행
2. 경고가 `DocumentCore::validation_report` 에 누적
3. `reflow_zero_height_paragraphs` 가 뒤이어 실행 (기존 자동 보정 로직, 유지)

### WASM API 노출

- `getValidationWarnings()` — JSON 리포트 반환
- `reflowLinesegs()` — 사용자 명시 reflow 실행, 처리된 문단 수 반환

### rhwp-studio UI

- 문서 로드 완료 후 `wasm.getValidationWarnings()` 호출
- `count > 0` 이면 `ValidationModal` 표시:
  - 제목: "HWPX 비표준 감지"
  - 본문: 경고 개수 + 종류별 요약 + 상세 `<details>` 토글 (최대 50건)
  - 버튼: **[자동 보정 (권장)] · [그대로 보기]**
  - **기본 포커스 = [자동 보정]** (작업지시자 지시)
  - Enter 키 = 자동 보정 실행
  - Escape · × · 오버레이 클릭 = 취소
- 사용자가 [자동 보정] 선택 시:
  1. `wasm.reflowLinesegs()` 호출 — 모든 경고 대상 문단 reflow
  2. `canvasView.loadDocument()` 재렌더
  3. 상태바에 `(비표준 lineseg N건 자동 보정됨)` 표시

### 비침습 원칙

- 경고 0건이면 **모달 미생성** — 대부분 사용자 경험에 방해 없음
- 모달은 한 번만 표시, 사용자 선택 후 닫힘
- 사용자가 [그대로 보기] 선택 시 rhwp 는 명세대로 렌더 (겹침 발생해도 숨기지 않음)

## 5. 보정 로직

`DocumentCore::reflow_linesegs_on_demand` 는 `needs_reflow_broadly` 판정을 통과한 문단에 대해 `reflow_line_segs` (렌더러 composer) 호출:

- R1 조건: `line_segs.is_empty() && !text.is_empty()`
- R2 조건: `line_segs.len() == 1 && line_height == 0`
- R3 조건: R1/R2 외에 `line_segs.len() == 1 && !contains('\n') && len > 40`

셀 내부 문단도 동일 처리. reflow 후 `composed` · `dirty_sections` 갱신 + `paginate()` 재수행.

## 6. Serializer 원본 보존 정책 (#177 Stage 2)

**rhwp는 비표준 lineseg 를 새로 생산하지 않는다**:

- `Paragraph.line_segs` 가 비어있지 않으면 → IR의 9개 필드 그대로 출력
- IR 이 비어있을 때만 fallback 으로 정적값 생성 (`Document::default()` 호환)

즉 rhwp 저장본은 **원본 한컴 파일의 lineseg 를 훼손 없이 보존**한다. 원본이 비표준이어도 rhwp가 그것을 악화시키지 않는다.

## 7. 검증 구조체 API

### Rust

```rust
// src/document_core/validation.rs
pub struct ValidationReport {
    pub warnings: Vec<ValidationWarning>,
}

pub struct ValidationWarning {
    pub section_idx: usize,
    pub paragraph_idx: usize,
    pub cell_path: Option<CellPath>,
    pub kind: WarningKind,
}

pub enum WarningKind {
    LinesegArrayEmpty,
    LinesegUncomputed,
    LinesegTextRunReflow,
}
```

### JavaScript / TypeScript

```typescript
// rhwp-studio/src/core/wasm-bridge.ts
export interface ValidationReport {
  count: number;
  summary: Record<string, number>;
  warnings: Array<{
    section: number;
    paragraph: number;
    kind: 'LinesegArrayEmpty' | 'LinesegUncomputed' | 'LinesegTextRunReflow';
    cell: { ctrl: number; row: number; col: number; innerPara: number } | null;
  }>;
}
```

## 8. 향후 확장 가능성

- **추가 감지 규칙**: lineseg 간 `textpos` 증가량이 UTF-16 경계와 불일치, `horzsize` 가 문단 너비와 무관한 기본값 등
- **설정**: 사용자 선호 저장 (자동 보정 항상 적용 vs 항상 묻기)
- **rhwp validate CLI (#185)**: Spec mode / Compat mode 분리 검증에 본 규칙 포함
- **대형 문서 UX**: 경고 50건 초과 시 CSV 내보내기 버튼

## 9. 관련 파일

- `src/document_core/validation.rs` — ValidationReport 구조
- `src/document_core/commands/document.rs::validate_linesegs` — 감지 로직
- `src/document_core/commands/document.rs::reflow_linesegs_on_demand` — 사용자 요청 보정
- `src/serializer/hwpx/section.rs::render_lineseg_array_from_ir` — 원본 보존
- `src/wasm_api.rs::get_validation_warnings` / `reflow_linesegs` — WASM API
- `rhwp-studio/src/ui/validation-modal.ts` — UI 모달
- `rhwp-studio/src/main.ts::initializeDocument` — 로드 훅
- `tests/hwpx_roundtrip_integration.rs::task177_*` — 회귀·측정 테스트
