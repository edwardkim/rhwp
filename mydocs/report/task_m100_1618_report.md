# 최종 결과보고서 — Task #1618 (vpos-reset 신뢰도 분석 — 경로 A 1단계)

**제목**: 경로 A 1단계 — LINE_SEG vpos-reset 신뢰도 전수 분석
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1618 · **브랜치**: `local/task1618`

## 1. 결론

**순수 vpos-추종 페이지네이션(경로 A 원안)은 비타당**. vpos-reset 은 신뢰 가능한 페이지수
**하한**(과대예측 0.0%)이나, **표 내부 페이지 분할을 인코딩하지 않아 체계적 과소예측**.

| 측정 | 결과 |
|------|------|
| vpos-예측 vs 한글 (통제셋 92) | **43.5%** (현행 rhwp 78.3%보다 후퇴), delta 전부 음수 |
| vpos-예측 vs rhwp (코퍼스 16,600) | 일치 74.2%, 과소 25.8%, **과대 0.0%** |
| 과소 원인 (표 상관) | big_table 보유 63% 과소 vs 無 18% |

근본: 표가 페이지를 넘을 때(PartialTable) IR 단일 컨트롤이라 vpos 리셋 없음.

## 2. 잔여 12건 재조준 (핵심 함의)

하이브리드 Path A(vpos body + 측정 표분할)라도 잔여 12건 **직접 미해소**:
- **footer 8건**: Page+Bottom 앵커(vpos 무관). #1616 규칙 부재.
- **다페이지 4건**: 전부 big_table → 본질은 **표 row-split 측정 정확도**(vpos 무관).

→ 잔여 12건 실제 레버: ① 표 row-split 측정(4건, 추적 가능) ② footer 규칙(8건, 불가).
vpos-추종은 잔여-12 타깃이 아닌 **광역 body 충실도** 장기 과제.

## 3. 권고
- 경로 A 2단계(vpos 프로토타입)를 **잔여-12 목적으로 진행 안 함**(타깃 불일치).
- 잔여 추구 시 **다페이지 4건 표 row-split 측정 분석**으로 재조준(별도 태스크).
- vpos-추종 하이브리드는 광역 충실도 개선용으로 분리(대규모·장기).

## 4. 산출물
- 분석기: `examples/vpos_reset_analyze.rs` (재사용 가능 진단 도구)
- 데이터: `output/poc/task1618_controlset_vpos.tsv`, `task1618_corpus_vpos.tsv`
- 문서: `_stage1`, 본 보고서, `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` 갱신
- 코드 변경: example 추가만(본 파이프라인 무영향)
