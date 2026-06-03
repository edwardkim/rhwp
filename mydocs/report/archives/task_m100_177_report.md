# 최종 결과 보고서: HWPX lineseg 비표준 감지·고지·보정

- **타스크**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task177` (→ `local/devel` merge 예정)
- **시작일**: 2026-04-18
- **완료일**: 2026-04-18
- **관련 Discussion**: [#188](https://github.com/edwardkim/rhwp/discussions/188), [#184](https://github.com/edwardkim/rhwp/discussions/184)

## 1. 타스크 개요

### 배경

이슈 원본은 "HWPX 저장 시 line_seg 부정확해서 한컴에서 겹친다"는 가정이었으나, 작업지시자의 재현 실험에서 **정반대 현상**이 드러났다:

```
한컴 HWPX (원본) → rhwp 로 열기 → 편집 없음 → rhwp 로 저장
    ↓
    ├─▶ 한컴으로 열기: ✅ 줄 넘김 정상
    └─▶ rhwp 로 다시 열기: ❌ 한 줄에 겹쳐 조판
```

즉 한컴이 자기 파일 호환성을 위해 **방어 로직(reflow)** 을 내장하여 비표준 lineseg 를 조용히 보정하는 반면, 명세를 엄격히 따르는 후발주자 rhwp 가 피해를 봤다. 이는 Discussion #188 에 상세 기록.

### 원칙 재정립 (작업지시자 확정)

한컴의 "조용한 방어" 는 기술부채 은폐의 한 형태. rhwp 는 이런 숨김을 거부한다:

1. **표준 준수 입력은 정확히 렌더** — 명세를 정답으로 취급
2. **비표준 입력은 감지하고 사용자에게 고지** — 조용한 보정 거부
3. **자동 보정은 사용자 명시 선택 후에만** — 기본 선택은 권장안(자동 보정)이나 선택 자체는 명시적으로 요청
4. **rhwp 자신도 비표준을 새로 생산하지 않음** — Serializer 는 원본 lineseg 를 그대로 보존

## 2. 목표 달성 요약

| 목표 | 상태 |
|---|---|
| 비표준 lineseg 3가지 패턴 감지 | ✅ R1/R2/R3 |
| IR 순수성 유지 (경고는 별도 구조) | ✅ `ValidationReport` |
| Serializer 원본 보존 (비표준 새로 안 생산) | ✅ `render_lineseg_array_from_ir` |
| 사용자에게 UI 로 고지 | ✅ `ValidationModal` 모달 |
| 자동 보정이 기본 선택 | ✅ 포커스 + Enter |
| 기존 자동 reflow 동작 유지 (호환) | ✅ `reflow_zero_height_paragraphs` 변경 없음 |
| 대형 실문서 false positive ≤ 소수 | ✅ 측정 완료 (보고서 섹션 4) |
| 기술문서 공개 | ✅ `hwpx_lineseg_validation.md` |
| Discussion 공개 | ✅ #188 |

## 3. 단계별 산출물

| Stage | 주제 | 주요 산출물 | 일자 |
|---|---|---|---|
| 1 | 감지 인프라 | `validation.rs`, `validate_linesegs`, 단위 10개 | 2026-04-18 |
| 2 | Serializer 원본 보존 | `render_lineseg_array_from_ir`, 단위 4 + 통합 2 | 2026-04-18 |
| 3 | Reflow on-demand + WASM API + 모달 UI | `reflow_linesegs_on_demand`, 2 WASM API, `validation-modal.ts` | 2026-04-18 |
| 4 | 통합 검증 + R3 발견 | R3 규칙 추가, 측정, 기술문서 | 2026-04-18 |

## 4. 감지 규칙 3종 (최종)

### R1: `LinesegArrayEmpty`
```
텍스트 있음 + line_segs 비어있음
```

### R2: `LinesegUncomputed`
```
line_segs.len() == 1 && line_segs[0].line_height == 0
```

### R3: `LinesegTextRunReflow` (Stage 4 발견)
```
line_segs.len() == 1 && !text.contains('\n') && text.chars().count() > 40
```

**R3 은 가장 중요한 규칙**. hwpx-02 에서 104개 문단 모두 lineseg 1개로 선언된 패턴을 발견한 후 도입. Discussion #188 에서 작업지시자가 세운 "한컴은 textRun 기반으로 reflow" 가설의 직접 증거.

## 5. false positive 실측 (9개 샘플)

```
sample             total      empty  uncomputed   textRunRefl
-----------------------------------------------------------------
blank_hwpx             0          0           0             0
ref_empty              0          0           0             0
ref_text               0          0           0             0
ref_table              0          0           0             0
ref_mixed              0          0           0             0
hwpx-02               15          0           0            15
form-002              53          0           0            53
2025-q1                4          0           0             4
2025-q2                3          0           0             3
```

- 레퍼런스 5건 모두 0건 — 오탐 없음
- 문제 재현 파일 hwpx-02 에서 15건 포착 — 실제 문제 감지
- 대형 실문서 3~53건 — 긴 설명 문단 감지

## 6. 검증 결과

### 6.1 Rust 단위 테스트
- `document_core::validation::tests`: 4개
- `document_core::commands::document::validate_linesegs_tests`: 14개 (기존 6 + Stage 3의 4 + R3의 4)
- `wasm_api::tests`: 3개 (Stage 3)
- `serializer::hwpx::section::tests`: 9개 (Stage 2의 4 신규 포함)

**전체 라이브러리**: **875 passed, 0 failed, 1 ignored** — 기존 850 대비 +25, 회귀 0건.

### 6.2 통합 테스트
**14 passed, 0 failed**:
- Stage 0/1/5 하네스: 8개
- Stage 2 lineseg 보존: 2개
- Stage 4 회귀·히스토그램·측정: 3개

### 6.3 TypeScript 빌드
`npx tsc --noEmit` 에러 0.

### 6.4 WASM 빌드
Docker 성공 (1분 15초, rhwp_bg.wasm 3.72MB).

## 7. 주요 설계 결정

### 7.1 IR 순수성 유지 (작업지시자 선택 B)

경고를 `Paragraph` 자체 필드가 아닌 `DocumentCore::validation_report` 별도 구조에 저장. IR 의 `PartialEq`, `Clone`, 스냅샷 경로에 영향 없음.

### 7.2 기존 자동 reflow 동작 유지 (작업지시자 선택 B)

`reflow_zero_height_paragraphs` 는 그대로 두고 **경고 기록은 그 앞에서** 수행. 기존 기능 파급 최소화. 사용자 명시 reflow 는 `reflow_linesegs_on_demand` 로 분리.

### 7.3 Serializer 원본 보존

`render_lineseg_array_from_ir` 가 IR 9개 필드 (text_start, vertical_pos, line_height, text_height, baseline_distance, line_spacing, column_start, segment_width, tag) 전부 그대로 출력. IR 이 비어있을 때만 fallback 으로 정적값 생성.

### 7.4 Stage 4 에서 R3 발견 — 분할정복의 성과

Stage 3 까지 R1/R2 만 구현했다가, Stage 4 측정에서 "겹침 재현 파일에서도 경고 0건" 을 발견. 이는 감지 규칙의 false negative. 분석 → R3 추가 → 재측정으로 검증. 개발 과정에서 문제를 숨기지 않고 발견·공개·개선한 사례.

### 7.5 UI 모달 — 자동 보정 기본 선택

작업지시자 지시에 따라 `[자동 보정 (권장)]` 버튼에 기본 포커스 + Enter 키 매핑. 사용자 대부분이 원하는 동작을 기본값으로 하되, 선택 자체는 여전히 **명시적 UI로 요청**.

## 8. 알려진 제한

1. **R3 threshold 40자** 는 한국어 기반 — 영문·일문 문서에서 조정 필요 가능
2. **편집 시 stale lineseg** — IR 의 line_segs 가 텍스트 편집 후 어긋나는 문제는 별도 이슈 (#186 범위 포함 가능)
3. **모달 "다시 표시하지 않음" 선호 저장 없음** — 매번 경고 표시 (후속 UX 이슈)
4. **경고 50건 초과 시 상세 제한** — CSV/파일 내보내기는 후속

## 9. Discussion 연계

- **#184** ("LLM으로 HWPX 만들기" 성공의 실체) — 비표준이 조용히 누적되는 메커니즘 지적
- **#188** (HWPX lineseg 비표준 생산자 = 한컴 자신) — 본 타스크의 직접 동기. rhwp 대응 원칙 공개

## 10. 관련 문서

- 수행계획서: `mydocs/plans/task_m100_177.md`
- 구현계획서: `mydocs/plans/task_m100_177_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_177_stage{1..4}.md`
- 기술문서: `mydocs/tech/hwpx_lineseg_validation.md`

## 11. 커밋 이력

- Stage 1: `43e6147` — ValidationReport + 감지 로직
- Stage 2: `40a9247` — Serializer 원본 보존
- Stage 3: `3a7044a` — WASM API + 모달 UI
- Stage 4: (커밋 예정) — R3 규칙 + 문서화

## 12. 통계

| 항목 | 수치 |
|---|---|
| Stage 수 | 4 |
| 진행 기간 | 1일 (2026-04-18) |
| 신규 파일 | 3개 (validation.rs, validation-modal.ts, hwpx_lineseg_validation.md) |
| 수정 파일 | 7개 (section.rs, wasm_api.rs, document.rs, wasm-bridge.ts, main.ts, wasm_api/tests.rs, 통합테스트) |
| 새 단위 테스트 | 25개 |
| 새 통합 테스트 | 5개 |
| 라이브러리 전체 테스트 | 875 passed / 0 failed / 1 ignored |
| 감지 규칙 | 3종 (R1/R2/R3) |
| 측정된 샘플 | 9건 |
| Discussion 게시 | #188 (본 타스크 개시 전) |
| 파생 이슈 | 없음 (후속 UX 이슈는 필요 시 별도) |

## 13. 결론

### 달성한 것

1. **HWPX 비표준 lineseg 감지 · 고지 · 보정 인프라 완성**
2. **Serializer 원본 보존** — rhwp 가 비표준을 새로 생산하지 않음
3. **사용자 UI 고지** — 모달 + 자동 보정 기본 선택
4. **실문제 재현 파일(hwpx-02)에서 15건 정확 감지** + 레퍼런스 오탐 0
5. **공개 기술문서 + Discussion** — 기술부채를 외부에 공개하는 원칙 실행

### 개발 과정의 교훈

- **Stage 3 완료 후 측정에서 false negative 발견** → Stage 4 에서 R3 추가로 대응
- 이는 "자기 구현을 의심하고 실측하기" 의 성과
- 처음 세운 감지 규칙(R1/R2)만으로는 실제 문제를 포착하지 못했음을 발견 · 공개 · 개선

### Discussion #188 원칙의 실천

- 한컴의 "조용한 방어" 를 복제하지 않음
- 사용자가 선택하도록 모달로 명시 요청
- 기본값은 권장안이되 선택 자체는 명시적
- 부정확한 값을 rhwp 가 새로 만들지 않음
- 개발 과정의 false negative 도 공개

### 다음 단계

1. Stage 4 산출물 + 최종 보고서 커밋
2. `local/task177` → `local/devel` merge
3. `local/devel` → `devel` push
4. 이슈 #177 close

## 14. 승인 요청

본 최종 결과 보고서 검토 후 승인 시:
- Stage 4 + 보고서 커밋
- merge 경로 진행
- 이슈 close
- 오늘할일 갱신

피드백이 있으면 `mydocs/feedback/` 에 등록 부탁드립니다.
