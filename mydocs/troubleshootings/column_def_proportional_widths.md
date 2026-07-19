---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# HWP 5.0 단 정의(cold) 너비/간격 비례값 인코딩

## 증상

KTX.hwp 파일에서 2단(same_width=false) 렌더링 시 오른쪽 단의 내용이 단의 왼쪽에서 패딩 처리된 것처럼 오른쪽으로 밀려서 렌더링됨. HWP 프로그램에서는 오른쪽 컨텐츠가 단의 시작 왼쪽 위치에서 바로 시작됨.

## 원인

### 1차 원인: 파서 바이트 순서 오류

HWP 5.0 스펙(표 140)과 hwplib Java 라이브러리의 바이너리 포맷 설명이 서로 다름:

**HWP 5.0 스펙 (표 140)**:
```
[attr(2)] [spacing(2)] [widths... 2×cnt] [attr2(2)] [separator(6)]
```
- spacing이 항상 존재
- 너비만 있고 간격 없음

**hwplib Java 라이브러리** (ForControlColumnDefine.java):
```
same_width=false: [attr(2)] [attr2(2)] [col0_width(2) col0_gap(2)] [col1_width(2) col1_gap(2)] ... [separator(6)]
same_width=true:  [attr(2)] [gap(2)] [attr2(2)] [separator(6)]
```
- same_width 여부에 따라 attr2/spacing 위치가 다름
- width + gap 쌍으로 저장

**실제 바이너리 검증 결과**: hwplib 포맷이 정확함.

### 2차 원인 (핵심): 너비/간격 값이 비례값

raw 바이트 분석:
```
08 00 89 05 9a 35 4e 02 18 48 00 00 00 00 00 00 00 00
attr  attr2 w0    g0    w1    g1    sep...
0x08  1417  13722 590   18456 0
```

width/gap 값 합계: 13722 + 590 + 18456 + 0 = **32768 (= 2^15)**

이 값들은 절대 HWPUNIT이 아니라 **body_width에 대한 비례값**:
- 13722 = 48.4mm → 실제로는 117.7mm
- 590 = 2.1mm → 실제로는 5.1mm
- 18456 = 65.1mm → 실제로는 158.3mm

비례 변환 공식:
```
실제_값 = 비례_값 / 합계(32768) × body_width
```

검증:
- col0_width = 13722/32768 × 79652 HU = 33363 HU = **117.7mm** ✓ (HWP 대화상자와 일치)
- col0_gap = 590/32768 × 79652 HU = 1434 HU = **5.1mm** ✓
- col1_width = 18456/32768 × 79652 HU = 44856 HU = **158.3mm** ✓

## 수정 내용

### 파일: `src/parser/body_text.rs`
- `parse_column_def_ctrl`: hwplib 바이트 순서로 읽기 (attr2 → width+gap 쌍)
- `cd.proportional_widths = true` 설정

### 파일: `src/model/page.rs`
- `ColumnDef`에 `gaps: Vec<HwpUnit16>` 필드 추가
- `proportional_widths: bool` 플래그 추가

### 파일: `src/renderer/page_layout.rs`
- `calculate_column_areas`: proportional_widths=true일 때 body_area.width 기준 비례 변환

### 파일: `src/serializer/control.rs`
- `serialize_column_def`: same_width 여부에 따른 올바른 바이트 순서로 직렬화

## 참고

- HWPML 3.0 스펙(표 101 COLUMN 엘리먼트)에서는 Width, Gap이 `[hwpunit]` 절대값으로 명시됨
- HWP 5.0 바이너리 포맷과 HWPX/HWPML 포맷은 인코딩 방식이 다름
- hwplib Java 코드는 바이트 읽기 순서는 정확하지만, 비례값→절대값 변환 로직은 렌더링 단계에서 처리하는 것으로 추정

## 일시

2026-02-16
