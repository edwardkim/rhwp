---
타스크: #290 cross-run 탭 감지가 inline_tabs 무시
단계: 4 / 4 — 최종 보고 + 이슈 close
브랜치: local/task290
작성일: 2026-04-24
---

# Stage 4 완료 보고서

## 1. 목표

구현계획서 Stage 4 의 4 항목 완료:

1. 최종 결과 보고서 작성 (`mydocs/report/task_m100_290_report.md`)
2. 오늘할일 (`mydocs/orders/20260424.md`) 에 #290 완료 항목 갱신 (신규 등록 → 종료)
3. 트러블슈팅 문서 (`mydocs/troubleshootings/tab_tac_overlap_142_159.md`) 에 #290 섹션 추가
4. 이슈 #290 close

## 2. 산출 문서

### 2.1 최종 결과 보고서

`mydocs/report/task_m100_290_report.md` — 다음 섹션 포함:
- 배경: 시각 증상 + PDF/SVG 대조
- 원인 분석: IR 관찰, 트레이스 메커니즘, ext[2] 포맷 실증
- 수정 내용: `resolve_last_tab_pending` 헬퍼 + cross-run 블록 2 곳 교체 + 테스트 6 건
- 검증 결과: 단위/통합/회귀/시각 4 축 모두 커버
- 범위 외 후속 과제: inline_tabs RIGHT/CENTER 렌더 버그 별도 이슈 후보
- 교훈 5 항목

### 2.2 오늘할일 갱신

`mydocs/orders/20260424.md`:
- 요약 부분에 오후 작업 (#290) 한 줄 추가
- "4. Task #290" 섹션 신설 (배경·원인·변경 범위·검증·교훈)
- 이슈 활동 표의 "신규 등록" → "종료" 로 이동

### 2.3 트러블슈팅 문서 보강

`mydocs/troubleshootings/tab_tac_overlap_142_159.md`:
- "후속 사건: #290" 섹션 신설
- #142 교훈 (`같은 데이터를 다른 경로로 계산하는 코드는 반드시 동기화`) 의 **범위가 `estimate_text_width` vs `compute_char_positions` 에서 "run 내부 탭 처리 vs cross-run 탭 감지" 로 확장됨**을 명시

## 3. 이슈 close

이슈 [#290](https://github.com/edwardkim/rhwp/issues/290) 에 close 코멘트 + close 처리. 코멘트 내용:
- merge 커밋 해시 (Stage 2 + 3)
- 수정 파일 목록
- 검증 지표 (184 페이지 회귀 1 페이지만 변경)
- before/after/PDF 3 면 비교 이미지 링크

## 4. 브랜치 정리

`local/task290` 커밋 로그:

```
Stage 1: 원인 트레이스 + ext[2] 매핑 실증 + 계획서 (7d3bbba)
Stage 2: resolve_last_tab_pending 헬퍼 + cross-run 블록 교체 + 단위 테스트 5건 (3c8bc4f)
Stage 3: 통합 테스트 + 184페이지 회귀 + before/after/PDF 3면 시각 비교 (0d2b747)
Stage 4: 최종 보고서 + 오늘할일/트러블슈팅 갱신 + 이슈 close (pending)
```

`git status` 클린 상태 확인 후 merge 준비.

## 5. 검증 지표 총괄

| 축 | 결과 |
|----|------|
| 단위 테스트 (신규 5) | 5/5 pass |
| 통합 테스트 (신규 1) | 1/1 pass |
| SVG snapshot | 3/3 pass |
| 전체 `cargo test --lib` | 955 pass (선존재 14 fail 무관) |
| clippy `--lib` | clean |
| 회귀 (4 문서 184 페이지) | 1 변경 (의도 100%) |
| RIGHT inline tab 회귀 | 0 (122 페이지 byte-identical) |
| 시각 비교 (3 면 PNG) | AFTER = PDF 일치 |

## 6. 종료

Task #290 모든 단계 완료:
- Stage 1 ✓ — 원인 트레이스 + ext[2] 매핑 실증
- Stage 2 ✓ — 구현 + 단위 테스트 5 건
- Stage 3 ✓ — 통합 테스트 + 184 페이지 회귀 + 시각 비교
- Stage 4 ✓ — 문서 + 이슈 close

작업지시자 최종 승인 후 `local/devel` → `devel` 머지 단계로 전환.
