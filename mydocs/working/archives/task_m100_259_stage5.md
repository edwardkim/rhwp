---
타스크: #259 한글 폰트명 → 메트릭 DB 영문명 매핑 누락
단계: Stage 5 — 문서화 + 메모리 + 체크리스트
브랜치: local/task259
작성일: 2026-04-23
---

# Stage 5 완료 보고서

## 1. 목적

- 최종 결과 보고서 작성
- 재발 방지용 메모리 + 체크리스트 박제
- 오늘할일 및 font_fallback_strategy 매뉴얼 반영

## 2. 작업 결과

### 2.1 최종 보고서

- `mydocs/report/task_m100_259_report.md` 신규 작성
  - 배경 · 문제 본질 · 매핑 표 (HY 7 + 본한글 13 + 본명조 10) · 검증 결과 3종 (자동/시각/스모크) · 근사 한계 · 산출물 · 후속 메모리 체크리스트

### 2.2 메모리 엔트리 신규

- `feedback_font_alias_sync.md` — 한글 폰트 추가 시 Layer 1 + Layer 2 동기화 필수 체크리스트
- `MEMORY.md` 인덱스에 한 줄 추가

### 2.3 매뉴얼 갱신

- `mydocs/tech/font_fallback_strategy.md` 부록 A 신규
  - 2-계층 폰트 이름 해석 체계 다이어그램
  - HY 7건 매핑 표 (em_size, 비고 포함)
  - 본한글/본명조 근사 정책 표 (근거 컬럼)
  - 근사 한계 (Latin · weight 축) 명시
  - **유지보수 체크리스트** (신규 한글 폰트 추가 시 4단계)

### 2.4 오늘할일 갱신

- `mydocs/orders/20260423.md` 갱신:
  - 요약 섹션에 Task #259 완주 표기
  - 11번 섹션 (Task #259) 신규 — 문제 본질 · 수정 범위 · 검증 · 산출물 · 교훈
  - 이슈 활동 섹션 갱신 (#259 신규 등록 + 종료 예정 표기)
  - 통합된 PR 에 #256 추가 (Task #146)

## 3. 산출물 총정리

### 코드
- `src/renderer/font_metrics_data.rs` (resolve_metric_alias 17 arm + `mod tests` 신규)

### 문서 (`mydocs/`)
- `plans/task_m100_259.md`, `plans/task_m100_259_impl.md`
- `working/task_m100_259_stage{1,2,3,4,5}.md`
- `report/task_m100_259_report.md`
- `tech/font_fallback_strategy.md` (부록 A 추가)
- `orders/20260423.md` (11번 섹션 · 이슈 활동 · 요약 갱신)

### 메모리
- `feedback_font_alias_sync.md` 신규
- `MEMORY.md` 인덱스 1줄 추가

### 시각 증거
- `output/svg/text-align-fix-after/`
- `output/svg/text-align-259-compare.html` (3-way 비교)
- `output/svg/stage4-{exam-kor,biz-plan,hwpx-02}-{before,after}/` (37 페이지 × 2)

### 빌드
- WASM 재빌드 완료 (pkg/rhwp_bg.wasm 4,076,166 bytes, +1,061 bytes 증가)

## 4. 5-Stage 요약

| Stage | 내용 | 결과 |
|---|---|---|
| 1 | 매핑 테이블 실측 확정 | HY 7건 DB 존재 확인, Pretendard/Noto Serif KR target 존재 확인 |
| 2 | resolve_metric_alias 17 arm + 단위 테스트 6건 | cargo test 953 passed, clippy clean |
| 3 | text-align.hwp 회귀 + svg_snapshot | golden 재생성 불필요, 4번 문단 숫자 폭 7.67→9.04px 정상화 |
| 4 | 스모크 스위프 3 샘플 | 비-HY 단독 페이지 10건 바이트 동일, 회귀 0 |
| 5 | 문서 + 메모리 + 매뉴얼 체크리스트 | 재발 방지 박제 완료 |

## 5. 다음 단계

- 작업지시자 최종 승인
- 커밋 (타스크 브랜치 `local/task259` 에서 소스 + 보고서 일괄)
- `local/devel` 로 머지 제안

Task #259 Stage 5 완료. 최종 승인 요청.
