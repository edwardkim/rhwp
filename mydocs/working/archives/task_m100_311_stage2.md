# Task #311 2단계 완료 보고서: vpos-reset 강제 분리 (실험 플래그)

상위: 구현 계획서 `task_m100_311_impl.md`, Epic #309

## 변경 요약

LINE_SEG `vpos-reset` 위치에서 PartialParagraph 강제 분리 로직 + `--respect-vpos-reset` CLI 플래그(기본 off) 구현. **검증 결과 가설 부정** — default on 전환은 보류하고 실험 플래그로 유지.

## 변경 파일

- `src/renderer/pagination/engine.rs` — `paginate_text_lines` 옵션 인자 추가, 신규 메서드 `paginate_with_forced_breaks` 신설
- `src/renderer/pagination.rs` — `PaginationOpts.respect_vpos_reset` 1단계에서 정의됨 (활용)
- `src/document_core/mod.rs`, `src/document_core/commands/document.rs` — `DocumentCore.respect_vpos_reset` 필드 추가
- `src/document_core/queries/rendering.rs` — `paginate()` 에서 옵션 전달
- `src/wasm_api.rs` — `set_respect_vpos_reset(enabled: bool)` 셋터 (변경 시 dirty 마킹 + 즉시 재페이지네이션)
- `src/main.rs` — `export-svg`/`dump-pages` `--respect-vpos-reset` 플래그 + 도움말

## 4개 샘플 검증 (옵션 ON)

| 샘플 | OFF (기존) | ON (실험) | 변화 | 평가 |
|------|------------|-----------|------|------|
| 21_언어 | 19쪽 | **20쪽** | **+1** | ❌ 가설 부정 |
| exam_math | 20쪽 | 20쪽 | 0 | ✓ 무변화 |
| exam_kor | 25쪽 | 25쪽 | 0 | ✓ 무변화 |
| exam_eng | 11쪽 | 11쪽 | 0 | ✓ 무변화 |

## 부정적 발견 — 근본 원인 재해석

### 페이지 7 비교 (21_언어, ON 모드)

```
OFF 단 0: pi=115(3..8) + pi=116 + pi=117(전체) ... + pi=127  (13개 항목)
ON  단 0: pi=115(3..8) + pi=116 + pi=117(0..1)              (3개 항목)
ON  단 1: pi=117(1..8) + pi=118 ~ pi=133                    (17개)
→ pi=134 가 다음 페이지로 overflow
```

`pi=117 line 1`의 vpos-reset에서 강제 분리 → 단 0이 짧아짐 → 단 1로 밀린 내용 누적 → pi=134 overflow → 페이지 1쪽 증가.

### 진짜 원인 (재추정)

우리 엔진의 column 가용 공간 계산이 HWP의 실제 column 사용 공간보다 **관대함** (15px 정도 더 채움). 그 결과:
- HWP는 pi=117 line 0 까지만 단에 넣고 line 1을 다음 단으로
- 우리는 pi=117 전체(line 0+1, 14.7 + 14.7 = 29.4px)를 한 단에 채움

vpos-reset 강제 분리만으로는 이 차이를 해결하지 못함. 오히려 분리만 되고 단 수용량이 보충되지 않아 후속 내용이 모두 다음 단/페이지로 밀려 페이지 수 증가.

### Task #310 분석 보고서의 가설 검증 결과

원 분석에서 권장한 "vpos-reset 강제 분리"는 가설로는 그럴듯했으나 **실측 결과 단일 해결책이 아님**. 다음 두 조건이 동시 충족되어야 PDF와 일치:
1. ✓ vpos-reset 위치에서 분리 (본 단계 구현)
2. ✗ column 가용 공간 계산 정확도 향상 (별도 작업 필요)

조건 2 미해결 시 조건 1만으로는 페이지 수 감소 효과 없음.

## 코드 상태

- `--respect-vpos-reset` 플래그는 **실험 플래그로 유지** (기본 off)
- 가설이 틀렸음을 입증하기 위한 도구로 보존 가치 있음
- 향후 column 가용 공간 정확도 개선 작업 시 결합 검증용

## 회귀 검증

- `cargo build` 성공
- `cargo test`: **992 passed; 0 failed**
- 옵션 OFF (기본): 4개 샘플 페이지 수 무변화

## 다음 단계 (3단계 변경)

당초 계획(default on 전환)은 보류. 3단계는 다음으로 변경:
- 부정적 발견을 분석 보고서(`mydocs/tech/line_seg_vpos_analysis.md`)에 추가 (가설 검증 결과 섹션)
- Epic #309 코멘트로 결과 게시 + 다음 sub-issue 후보 제안 ("column 가용 공간 정확도 조사")
- 본 sub-issue #311 클로즈
