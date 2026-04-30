# `samples/2010-01-06.hwp` 페이지 1 각주 1) 줄 간격 좁음

| 항목 | 내용 |
|------|------|
| 발견일 | 2026-04-30 |
| 샘플 | `samples/2010-01-06.hwp` |
| 위치 | 페이지 1 하단 각주 1) "경상수입 / .조세수입 / .세외수입" |

## 증상

페이지 1 하단의 각주 1) 안 3줄("경상수입", ".조세수입", ".세외수입")의 줄 간격이 다른 각주(2)~5))에 비해 좁아 보임.

## SVG 실측

| 줄 | y baseline (px) | gap from prev |
|----|-----------------|---------------|
| 1) 경상수입 (p[0]) | 899.4 | — |
| 1) 조세수입 (p[1]) | 915.4 | **+16.0** ⚠️ |
| 1) 세외수입 (p[2]) | 936.2 | +20.8 ✓ |
| 2) 자본수입 | 964.5 | +28.3 |
| 3) 경상지출 | 988.1 | +23.6 |
| 4) 자본지출 | 1011.7 | +23.6 |
| 5) 순융자 | 1035.2 | +23.6 |

## HWP 원본 구조 (각주 1) — paragraph 4의 표 셀[15] " 경상수입"의 controls[0])

```
각주[0] number=1 paragraphs=3
  fn p[0] ps_id=7 text='  경상수입 '
    ls[0] vpos=0    lh=1200  ls=360
  fn p[1] ps_id=7 text='  ․조세수입 : 사회보장기여금 포함'
    ls[0] vpos=1560 lh=1200  ls=360       ← p[0]→p[1] vpos delta=1560 HU=20.8px
  fn p[2] ps_id=11 text='  ․세외수입 : ...'
    ls[0] vpos=3120 lh=1200  ls=360       ← p[1]→p[2] vpos delta=1560 HU=20.8px
```

각주 2)~5)는 모두 paragraph 1개씩 (ps_id=5, ls=720 HU).

## 근본 원인

**각주 1) p[0]→p[1] transition에서 4.8px (= line_spacing 360 HU) 누락**.

| transition | HWP vpos delta | SVG 실측 | 누락 |
|-----------|---------------|---------|------|
| p[0]→p[1] | 20.8px | 16.0px | **-4.8px** |
| p[1]→p[2] | 20.8px | 20.8px | 0 |

p[1]→p[2]는 일치하는데, p[2]의 ParaShape(ps_id=11)에 추가 spacing_before가 있어 누락된 line_spacing과 우연히 상쇄된 것으로 추정.

즉 **footnote multi-paragraph 렌더링에서 paragraph 간 line_spacing(=ParaShape의 ls 필드) 누적이 일관되지 않음**. paragraph transition 시 line_spacing이 더해지지 않거나 spacing_before로만 처리되는 분기 누락.

이는 #479의 layout drift와 유사한 패턴 (trailing/inter-paragraph spacing 처리). 동일 영역 의심.

## 핵심 코드 위치 (추적 필요)

| 파일 | 위치 |
|------|------|
| `src/renderer/layout/picture_footnote.rs` | footnote 렌더링 진입 |
| `src/renderer/layout/paragraph_layout.rs` | paragraph 렌더 (각주 paragraph도 여기 사용) |
| `src/renderer/typeset.rs` | spacing_before/after 누적 |

특히 paragraph_layout이 각주 paragraph 처리할 때 호출되는 분기에서 paragraph간 line_spacing 처리가 누락된 경우.

## 검증 방법

1. 한컴 한글 2010/2020에서 본 파일의 각주 1) 줄 간격이 다른 각주와 동일한지(=다른 각주는 paragraph가 1개라 동일하게 측정 어려움) 또는 vpos 의도(20.8px)와 동일한지 확인
2. 한컴 PDF 200dpi 비교 (보조 ref)
3. ParaShape 7, 11의 line_spacing/spacing_before 정확한 값 확인 (현재 dump에는 line_segs의 ls 값만 표시)

(메모리 가이드 [PDF 비교 결과는 절대 기준이 아님](feedback_pdf_not_authoritative.md): 한컴 환경 함께 점검 필수.)

## 관련 이슈

- #479 (페이지 12 layout drift, 누적 trailing line_spacing) — 유사한 spacing 누적 패턴

## 수정 방향 후보

| 옵션 | 변경 | 영향 |
|------|------|------|
| **A** | footnote paragraph_layout에서 paragraph 간 line_spacing 누적 적용 | 각주 줄 간격 정확화. 회귀 위험 검증 필요 |
| **B** | ParaShape 7과 11의 spacing_before 차이를 기반으로 분기 처리 | ParaShape 의존 — 다른 ParaShape에서도 같은 누락 가능성 검토 |

## 임시 회피책

없음 — 본질 수정 필요 (paragraph_layout 또는 footnote 처리).

## 별개 이슈로 등록 후보

본 트러블슈팅 분석을 GitHub Issue로 등록하여 추적 가능. #479와 묶어 한 task로 진행 가능 (둘 다 inter-paragraph spacing 누적 문제).
