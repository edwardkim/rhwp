# 해결 — #1658 한글 행높이 COM 추출 (블로커 해소) + 3 PDF 질문 정리

- 일자: 2026-06-30 / 대상: 한글 표 행높이 레퍼런스 차단 해소.

## 1. COM 셀 높이 — 해결 ✅
이전 실패(CellShape=None, TableLowerCell False)의 원인: **캐럿이 표 밖**이라 셀 진입 실패.
해법(작동):
```
ctrl = hwp.HeadCtrl; while ctrl: if ctrl.CtrlID=="tbl": break; ctrl=ctrl.Next   # 표 찾기
hwp.SetPosBySet(ctrl.GetAnchorPos(0)); hwp.FindCtrl()                          # anchor 진입
hwp.ShapeObjTableSelCell()  # True
n = hwp.get_row_num()       # 행수
hwp.Cancel()
for _ in range(n): h = hwp.get_row_height(); hwp.TableLowerCell()              # 행별 높이(mm)
```
도구: `tools/hangul_row_heights.py` (한글 행높이 mm→px vs rhwp cut_row_h(RHWP_TABLE_DRIFT)).

## 2. 진단 결과 — 행높이 fidelity 정상 (PDF 가 못 준 답)
| 문서 | 한글 행수 | rhwp 행수 | 총높이 diff(rhwp−한글) |
|------|----------|----------|----------------------|
| byeolpyo1(별표1) | 15 | 15 | **+6.4px** (~0.1%) |
| byeolpyo4(별표4) | 79 | 80 | **+14.4px** (~0.07%) |
- 행 0~9 완전 일치(27.3/99.0/19.8…). 거대 셀도 총합상 일치.
- byeolpyo4 per-row 큰 차이는 **행 수 79 vs 80**(rowspan/병합 파싱 off-by-one) 정렬 아티팩트일 뿐,
  **총 표 높이는 한글과 ~0.1% 일치**.
- ∴ **행높이 측정 fidelity 정상**(byeolpyo1·4 모두). byeolpyo4 Δ+3 over 는 **측정이 아니라 cut 로직**
  (거대 셀 페이지 분할 배치) 확정. (앞선 PDF·통합진단의 "줄높이 drift" 가설 최종 기각.)

## 3. PDF 질문 3종 정리
- **PDF 전용 파라미터셋**: 액션 `PrintToPDF` + pset `HPrint`(PrinterName='Hancom PDF',
  **PrintMethod=4**=모아찍기/배율 → 25→13쪽 압축 원인). 단 **출력 경로 항목 없음**(프린터 경유, 자동화
  파일 미생성) → PrintToPDF 자동 export 미해결.
- **save_as PDF 배율 100%**: save_as 는 **doc 저장 인쇄설정(PrintMethod=4)** 을 honor, **세션 HPrint
  override 무효**(method0 설정 후에도 13쪽). doc 인쇄설정 자체를 바꿔 재저장해야 하나 미해결.
- **결론**: PDF 경로는 (i) 미해결 + (ii) **redundant**. COM 이 **편집기** 행높이를 직접 주며, 한글 PDF
  는 편집기와 다른 렌더(별표4 25↔13)라 오히려 부정확. **COM 이 올바른 레퍼런스.**

## 4. 의의
- 차단됐던 한글 행높이 레퍼런스를 **COM 으로 해소** → fidelity 정량 비교 가능.
- byeolpyo4 Δ+3 의 성격을 **cut 로직(거대 셀 분할)** 으로 확정(측정 아님). 행 수 79↔80 파싱 차이도 발견.
- 후속: 거대 셀 페이지 분할 cut 로직을 한글 행위치 기준으로 정합(COM 행높이/행-페이지 매핑 활용).
