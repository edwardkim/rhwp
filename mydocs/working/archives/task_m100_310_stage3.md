# Task #310 3단계 완료 보고서: 4개 샘플 vpos 패턴 분석

상위: 구현 계획서 `task_m100_310_impl.md`, Epic #309

## 산출물

- `mydocs/tech/line_seg_vpos_analysis.md` — 4개 샘플 vpos 패턴 비교 분석 보고서
- Epic #309 코멘트 게시 (분석 핵심 요약)

## 핵심 발견

| 샘플 | SVG 쪽 | vpos-reset | FullParagraph 내부 reset |
|------|--------|-----------|--------------------------|
| 21_언어 | 19 (+4) | 13 | **7** ← 어긋남 원인 |
| exam_math | 20 ✓ | 0 | 0 |
| exam_kor | 25 | 8 | 0 |
| exam_eng | 11 ✓ | 0 | 0 |

**결정적 단서**: 21_언어의 어긋남은 13개 vpos-reset 중 7개가 `FullParagraph` 내부에 있는 케이스 — 즉 HWP는 문단 중간에서 단/페이지를 끊을 의도였으나 우리 엔진은 통째로 한 단에 배치한 경우. 다른 3개 샘플은 이 패턴이 0건.

## 도출된 설계 원칙

1. vpos-reset 0개 문서는 현 엔진 동작 유지
2. vpos-reset이 PartialParagraph로 자연 분리된 경우는 현 엔진 동작 유지
3. **FullParagraph 내부 vpos-reset이 있는 경우만 분리 강제** → 21_언어만 영향, 다른 샘플 회귀 0 예상

## 권장 2단계 작업 (Epic #309 후속)

**Sub-issue #2 (제안)**: `페이지네이션에서 LINE_SEG vpos-reset을 단/페이지 경계로 강제`

페이지네이션 엔진에서 FullParagraph 처리 시 line_segs에 vpos-reset이 있으면 PartialParagraph 분리. 옵션 플래그로 단계 도입.

전면 재설계(원래 권장안)보다 훨씬 좁은 범위로 동일한 효과 가능 — 분석 결과의 최대 수확.

## 검증

- 분석 보고서 작성 완료
- 4개 샘플 dump-pages 출력 데이터 확보
- Epic #309 코멘트로 핵심 요약 게시 예정

## 다음 단계

- 본 타스크(#310) 종료 (최종 보고서 작성 후 close)
- Epic #309에 후속 sub-issue 등록 (작업지시자 승인 시)
