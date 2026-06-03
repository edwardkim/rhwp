# Stage 4 단계별 완료보고서: 통합 검증 + 문서화

- **타스크**: [#177](https://github.com/edwardkim/rhwp/issues/177)
- **마일스톤**: M100 (v1.0.0)
- **브랜치**: `local/task177`
- **일자**: 2026-04-18
- **단계**: Stage 4 / 4 (최종)

## 1. 수행 범위

구현계획서 Stage 4 + Stage 3 에서 발견한 false negative 대응:

1. `hwpx-02.hwpx` 회귀 테스트 추가
2. 대형 실문서 4건 + 레퍼런스 5건 false positive 측정
3. **R3 감지 규칙 `LinesegTextRunReflow` 신설** — hwpx-02 에서 파생된 실측 기반
4. 기술문서 `mydocs/tech/hwpx_lineseg_validation.md` 작성
5. 최종 결과 보고서 `mydocs/report/task_m100_177_report.md`

## 2. 산출물

### 2.1 R3 규칙 신설 — 결정적 발견 기반

**Stage 3 완료 시점**에 false positive 측정을 돌려본 결과 **모든 샘플 0건** 이었다. 이는 **R1/R2 규칙이 너무 보수적**이었음을 뜻함. `hwpx-02.hwpx` (작업지시자가 겹침을 재현한 파일)의 XML 을 직접 분석하여 **문단당 lineseg 1개만 선언**된 비표준 패턴을 발견:

- hwpx-02: 104개 문단 모두 lineseg 정확히 1개씩 → 긴 문단조차 단일 lineseg
- 파서는 이를 그대로 파싱 → 렌더러가 1개 좌표에 모든 텍스트 그림 → 겹침

이를 감지하는 R3 규칙:

```
line_segs.len() == 1 && !text.contains('\n') && text.chars().count() > 40
```

**수정 파일**:
- `src/document_core/validation.rs` — `WarningKind::LinesegTextRunReflow` 추가
- `src/document_core/commands/document.rs` — R3 체크 + `needs_reflow_broadly` 확장
- `src/wasm_api.rs` — JSON kind 매핑
- `rhwp-studio/src/core/wasm-bridge.ts` — TypeScript union 타입 확장

### 2.2 회귀 테스트 2개

`tests/hwpx_roundtrip_integration.rs` 에 추가:

1. **`task177_hwpx_02_regression`** — 작업지시자 제공 샘플의 파싱·직렬화·재파싱 크래시 없음 + line_segs 길이 보존 확인
2. **`task177_hwpx_02_lineseg_histogram`** — lineseg 분포 관측 (eprintln 로 수치 표시)
3. **`task177_false_positive_measurement`** — 9개 샘플에 대한 경고 카운트 집계

### 2.3 R3 단위 테스트 4개

`src/document_core/commands/document.rs::validate_linesegs_tests`:

- `validate_detects_textrun_reflow_pattern` — R3 정확 감지
- `validate_skips_textrun_reflow_for_short_text` — 짧은 텍스트는 제외
- `validate_skips_textrun_reflow_when_has_newline` — `\n` 있으면 제외
- `needs_reflow_broadly_covers_textrun_reflow` — reflow 대상 포함

### 2.4 기술문서 신규

**`mydocs/tech/hwpx_lineseg_validation.md`**:
- 배경 · 관찰 패턴 · 원인 분석
- 감지 규칙 3종 (R1/R2/R3) 상세
- false positive 실측 수치
- 고지 플로우 (엔진 → WASM → UI)
- Serializer 원본 보존 정책
- Rust/TS API 요약

### 2.5 WASM 재빌드

R3 규칙 반영한 WASM:
```
docker compose run --rm wasm
# Finished in 1m 15s
# rhwp_bg.wasm 3.72MB
```

## 3. false positive 실측 수치

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

### 해석

- **레퍼런스 5건 모두 0** — 단순 문서에서 오탐 없음 (R3 휴리스틱 40자 threshold 의 방어적 효과)
- **hwpx-02 15건** — 겹침 재현 파일에서 정확히 감지 (15개 긴 문단 감지)
- **실문서 4건 소수 (3~53건)** — 보도자료·양식의 긴 설명 문단 포착

## 4. 검증 결과

### 4.1 단위 테스트

```
document_core::commands::document::validate_linesegs_tests: 14 passed
- validate_detects_* (3)
- validate_skips_* (5)
- validate_recurses_into_table_cells (1)
- validate_records_multiple_warnings (1)
- needs_reflow_broadly_* (4)
```

전체 라이브러리: **875 passed, 0 failed, 1 ignored** — Stage 3의 871 대비 +4 (R3 단위 테스트).

### 4.2 통합 테스트

```
14 passed
- Stage 0/1/5 (기존 8)
- task177_lineseg_preserved_on_roundtrip_ref_text/ref_mixed (2)
- task177_hwpx_02_regression (1)
- task177_hwpx_02_lineseg_histogram (1)
- task177_false_positive_measurement (1)
```

### 4.3 WASM 빌드

```
Finished `release` profile [optimized] target(s) in 24.33s
Done in 1m 15s
pkg/rhwp_bg.wasm 3.72MB
```

## 5. 완료 기준 대조

구현계획서 Stage 4 완료 기준:

| 기준 | 상태 | 근거 |
|---|---|---|
| `hwpx-02.hwpx` 회귀 테스트 통과 | ✅ | `task177_hwpx_02_regression` |
| 대형 샘플 4건 false positive 측정·문서화 | ✅ | 9건 측정 (레퍼런스 5 + 실문서 4) |
| 기술 문서 1건 (`hwpx_lineseg_validation.md`) | ✅ | 작성 완료 |
| 최종 결과 보고서 작성 | ⏸ | 다음 단계 |
| 전체 라이브러리 테스트 그린 | ✅ | 875/0/1 |

## 6. 주요 설계 결정

### 6.1 R3 threshold 40자

한글 한 줄 ~30자에 여유를 두어 40자. 이보다 낮추면 false positive 증가, 높이면 짧은 문단의 겹침을 놓침. 실측 결과 레퍼런스 5건 모두 0건이어서 현재 값이 적절.

### 6.2 false negative 를 스스로 발견

Stage 3 완료 시 측정에서 **hwpx-02 에서도 경고 0건**이 나와 "R1/R2만으로는 부족하다" 는 것이 드러났다. 이는 개발 과정에서 **감지 규칙이 실제 문제를 포착하지 못하는 상태**였다는 뜻. Stage 4에서 추가 분석하여 R3을 도입한 과정은 Discussion #188 원칙의 **실천 사례** — 조용히 넘어가지 않고 공개 · 개선.

### 6.3 `task177_hwpx_02_lineseg_histogram` 은 디버그 도구

향후 새 비표준 패턴을 발견하면 이 테스트를 참고하여 XML 직접 분석 → 규칙 추가 경로를 유지.

## 7. 알려진 제한

- **R3 휴리스틱**: threshold 40자는 한국어 기반. 영문·일문 문서에서는 조정 필요할 수 있음 (차기 이슈)
- **cell 내부 문단**: 셀 안의 긴 텍스트도 감지되지만, 셀 너비에 따라 판정 임계를 동적 조정하면 더 정확 (차기)
- **편집 시 stale lineseg**: 편집 후 lineseg 가 IR에 남아 부정확해지는 문제는 별도 이슈

## 8. 다음 단계

- **최종 결과 보고서** 작성 (`mydocs/report/task_m100_177_report.md`)
- Stage 4 커밋 + `local/devel` merge + `devel` push
- 이슈 #177 close

## 9. 승인 요청

본 Stage 4 단계별 완료보고서 검토 후 승인 시 최종 결과 보고서 작성 + merge 진행.
