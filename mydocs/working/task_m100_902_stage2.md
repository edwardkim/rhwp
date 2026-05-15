# Task #902 Stage 2 보고서 — WMF binary 정밀 reverse engineering

**Stage**: 2 / 5
**상태**: Task #896 Stage 9 분석 오류 정정 + 의심 fix 영역 식별

## 1. WMF binary 정밀 디코드 (sample16 bin_id=3)

```
Header: Standard WMF, fsize=4773574 bytes, num_objs=12

rec[0]  SETMAPMODE          mode=8 (MM_ANISOTROPIC)
rec[1]  SETWINDOWORG        Y=72,   X=56          ← 단 1회
rec[2]  SETWINDOWEXT        Y=4161, X=6333        ← 단 1회

SETVIEWPORTORG: 0회
SETVIEWPORTEXT: 0회
SCALEWINDOWEXT: 0회
SCALEVIEWPORTEXT: 0회

CREATEFONTINDIRECT: 437개 (lfHeight 분포: -117, -100, -83, -75, -67, -42, 3, 16, ...)
EXTTEXTOUT: 554개
Total: 20869 records
```

### 1.1 Task #896 Stage 9 분석 오류 정정

Task #896 Stage 9 (issue #902 본문) 의 기록:
```
rec[1] SETWINDOWEXT (y=72, x=56)    ← 단 1회 호출, 매우 작은 값
rec[2] SETWINDOWORG (y=4161, x=6333)
```

**오류**: rec[1] 과 rec[2] 의 record type 이 반대. 정확한 값:
- rec[1] = `SETWINDOWORG (X=56, Y=72)` — origin offset (작은 값)
- rec[2] = `SETWINDOWEXT (X=6333, Y=4161)` — full extent (큰 값, element bbox 와 거의 일치)

WMF spec 의 param order: Y first, X second. WindowExt 가 element 좌표 (0~6333, 0~4161) 와 일치하므로 Task #860 의 viewBox 자동 확장 동작이 정확.

### 1.2 따라서 issue #902 의 "비표준 비례" 가설 재검토

원본 가설: WindowExt(56, 72) 가 너무 작음 → element 좌표 (6333, 4212) 와 비례 부재.

**수정된 사실**: WindowExt(6333, 4161) 는 element 좌표 와 일치 ≈ Task #860 viewBox 6333×4212 와 일치 (4212-4161=51 차이는 element 가 window 약간 벗어남).

**ROOT CAUSE 재정의 필요** — viewBox 자체는 정확. 그렇다면 "텍스트 작음" 의 원인은 다른 영역.

## 2. 새 ROOT CAUSE 후보 — DX (character spacing) 처리

### 2.1 의심 코드 (`src/wmf/converter/svg/mod.rs:928~934`)

```rust
if dx != 0 {
    let excess_dx = (font.height.abs() / 2)
        * i16::try_from(s.width()).unwrap_or(0);
    let dx = core::cmp::max(dx - excess_dx, 0);
    tspan = tspan.set("dx", dx);
}
```

### 2.2 동작 분석

WMF EXTTEXTOUT 의 DX array = char 별 advance (next char's origin offset).

코드 의도: SVG tspan 의 dx 는 "natural width 외 추가 advance" 로 가정. excess_dx ≈ natural_width 추정값.

| s.width | 의미 | excess_dx | dx 조정 |
|---------|------|-----------|---------|
| 2 (Korean wide) | 전각 | font_height (= 117) | WMF_DX(117) - 117 = 0 |
| 1 (Latin narrow) | 반각 | font_height/2 (= 58) | WMF_DX(117) - 58 = 59 |

### 2.3 잠재 문제

Korean wide char + WMF_DX=117 → dx=0:
- SVG 가 natural width 만 사용
- 폰트 fallback (Apple SD Gothic Neo 등) 의 natural width ≈ font.height (em-width) 정상
- 그러나 일부 폰트 (특히 sans-serif fallback) 는 Korean 글리프 부재 시 narrow advance → "가까이 모임"

또는 한컴 viewer 는 다른 알고리즘 사용 — em-width 가산 후 explicit DX 적용 (한 단계 늘려 적용).

## 3. PDF 비교 한계 (Stage 1 재확인)

PDF 의 WMF 영역은 raster vector graphics 로 렌더 — pdftohtml/pdftotext 텍스트 추출 불가. 따라서 정밀 ratio 측정 불가, **시각 비교 의존 필요**.

## 4. 진단 데이터

- /tmp/task902_wmf/sample16_pic3.wmf (4.7 MB, 20869 records)
- /tmp/task902_svg/wmf_p394.svg (decoded SVG from rhwp)
- /tmp/task902_pdf/sample16_p18.xml (PDF text — WMF 영역 제외)

## 5. Stage 3 진행 전 결정 요청

### 5.1 시각 비교 요청

작업지시자께:
- rhwp-studio 의 sample16 page 18 의 WMF 영역 (paragraph 394 "주전산센터 목표시스템 구성(안)")
- pdf/hwp3-sample16-hwp5-2022.pdf page 18 의 WMF 영역
- 두 영역 의 텍스트 크기 + 간격 비교 + 어느 정도 차이 (예: rhwp 가 한컴 대비 ~70% 크기)

이 정보 없으면 정확한 fix ratio 도출 불가.

### 5.2 Fix 방향 후보 (시각 비교 결과에 따라)

- A: excess_dx 공식 조정 (예: font.height × s.width × 0.45 대신 0.5)
- B: excess_dx 제거 (raw WMF_DX 사용)
- C: font-size scale factor 추가 (전체 텍스트 비례 보정)
- D: viewBox 알고리즘 변경 (Task #860 영향 재검토)

각 후보의 효과는 시각 비교 후 측정 가능.

## 6. 산출물

- 본 보고서: `mydocs/working/task_m100_902_stage2.md`
- WMF binary 추출 도구: `examples/extract_wmf.rs`
- 디코드 결과: WMF binary 의 record breakdown
