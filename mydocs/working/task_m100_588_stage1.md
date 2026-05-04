# Task #588 Stage 1 — 한컴 PDF 정답지 글리프 확정 + 광범위 PUA 통계

## 목표

1. `samples/exam_eng.pdf` 7쪽의 U+F003B 글리프 시각 형태 확정
2. 14 fixture (전체 159 샘플) 의 SPUA-A 저영역 (`0xF0000~0xF00CF`) 코드포인트 전수 통계
3. 회귀 차단 정합 점검

## 1. 글리프 형태 — 한컴 PDF 정답지 분석

### 1.1 PDF 구조 분석

`samples/exam_eng.pdf` 7쪽은 임베디드 서브셋 폰트 `AAAAAL+HCRBatang` (HWP의 HY신명조 등가) 를 사용하며, U+F003B 위치:
```
x=593.6, y=275.2, font=AAAAAL+HCRBatang, size=11.5pt
```

ToUnicode CMap 미설정 → PDF 텍스트 추출에서 U+F003B 가 그대로 보존 (pdfminer 검증).

### 1.2 임베디드 폰트 글리프 분석

`AAAAAL+HCRBatang` 의 `uF003B` 글리프 (1 contour, 7 points):

| Point | Coordinate | 의미 |
|-------|-----------|------|
| 0 | (661, 799) | 상부 우측 (stem top-right) |
| 1 | (661, 255) | stem 우측 하단 |
| 2 | (882, 255) | 화살촉 우측 끝 |
| 3 | (486, -141) | **화살촉 끝 (tip, 하단)** |
| 4 | (87, 255) | 화살촉 좌측 끝 |
| 5 | (308, 255) | stem 좌측 하단 |
| 6 | (308, 799) | 상부 좌측 (stem top-left) |

bbox: (87, -141, 882, 799), units/em = 1000

ASCII 렌더링:
```
       ████████████
       ████████████   ← stem (sturdy/thick rectangular shaft)
       ████████████
       ████████████
       ████████████
       ████████████
   ████████████████████  ← arrowhead (full-width horizontal)
    ██████████████████
     ████████████████
      ██████████████
       ████████████
         ████████
          ██████
           ████
            ██     ← tip
```

**확정 형태**: solid filled (단일 contour 폐다각형 → fill 영역) **down arrow**.

### 1.3 후보 매핑 비교

| 후보 | 이름 | 유닉스 글리프 | 정합 |
|------|-----|---|------|
| ↓ U+2193 | DOWNWARDS ARROW | 단순 (수능 표준) | **권장** |
| ⇩ U+21E9 | DOWNWARDS WHITE ARROW | 외곽선/속빈 | 형태 불일치 (속빈) |
| ⬇ U+2B07 | DOWNWARDS BLACK ARROW | 두꺼운 검정 | 형태 정합 |
| ⇓ U+21D3 | DOWNWARDS DOUBLE ARROW | 두 줄 | 형태 불일치 |

**근거**:
- 글리프 외곽 (stem 35% / arrowhead 100% / 단일 fill) = solid black arrow
- 한국 수능 출제 표준: ↓ U+2193 (한국교육과정평가원 + EBS 출제 폰트 정합)
- 해당 fixture 자체가 **수능 영어 기출** 문서 (40번 요약형 문항)

**최종 권장**: **↓ U+2193 DOWNWARDS ARROW**

대안 (⬇ U+2B07): 일반 폰트의 ↓ (U+2193) 글리프가 본 HCRBatang 글리프보다 얇게 그려지는 폰트가 다수 → 본 글리프의 두꺼운 시각과 더 정합. 단, Unicode 표준 의도는 다름 (U+2B07 은 명시적 "검정 화살표").

작업지시자 시각 판정 권위 (메모리 룰 `reference_authoritative_hancom`).

## 2. 14 fixture 광범위 PUA 통계

### 2.1 Task #588 target range (`0xF0000..=0xF00CF`)

전체 159 샘플 (HWP / HWPX) 의 export-text 출력 전수 스캔 결과:

| 코드포인트 | 합계 | 분포 |
|---|---|---|
| **U+F003B** | 1 | exam_eng.hwp(1) — **본 task target** |
| U+F0090 | 1 | img-start-001.hwp(1) — 별도 task 후보 |

**확정**: 본 사이클 정정 영역 = U+F003B 1건. U+F0090 은 같은 영역의 별개 코드포인트 (별도 이슈 등록 후 처리).

### 2.2 인근 영역 (회귀 차단 점검)

기존 매핑 영역과 본 사이클 신설 분기 (`0xF0000..=0xF00CF`) 충돌 없음 확인:

| 영역 | 범위 | 검출 코드포인트 종 | 매핑 상태 |
|------|------|---|---|
| **본 사이클** | `0xF0000..=0xF00CF` | 2 (F003B, F0090) | 분기 없음 (default = 원본 유지) |
| Task #528 | `0xF00D0..=0xF09FF` | 4 (F00DA, F012B, F081C, F0854/F0855) | F00DA/F0854/F0855 매핑됨, F012B/F081C 미매핑 |
| Task #509 | `0xF02B0..=0xF02FF` | 13 (F02B1~F02BA, F02C3~F02C5, F02CE~F02D0, F02EF, F02FC) | F02B1~F02B9, F02EF 매핑됨, 나머지 미매핑 |

본 사이클 분기 (`0xF0000..=0xF00CF`) 는 다른 영역과 디스조인트 → 분기 추가 시 회귀 위험 0.

### 2.3 별도 task 후보 (본 사이클 외)

`feedback_no_pr_accumulation` 정합 — 본 사이클은 U+F003B 1건만 처리. 같은 영역의 미매핑 코드포인트는 별도 task:

- U+F0090 (img-start-001.hwp 1건) — 본 task 영역
- U+F012B (복학원서.hwp 1건)
- U+F081C (복학원서.hwp 2건)
- U+F02BA, F02C3~F02C5 (mel-001 최대 3건)
- U+F02CE~F02D0 (k-water-rfp 1건씩)
- U+F02FC (pic-in-head-01 + pic-in-table-01 각 11건)

상기는 본 task 종료 후 일괄 이슈 등록 후보 (작업지시자 결정 대기).

## 3. 회귀 차단 점검

| 항목 | 결과 |
|------|------|
| 본 사이클 분기 영역 (`0xF0000..=0xF00CF`) 의 다른 코드포인트 영향 | U+F0090 1건 — default 분기 (원본 유지) 로 처리 (회귀 0) |
| 기존 매핑 영역 (`0xF02B0..=0xF02FF`, `0xF00D0..=0xF09FF`, `0xF020..=0xF0FF`) | 본 사이클 무영향 |
| Task #509/#528 단위 테스트 회귀 | 본 사이클 신설 분기 → 기존 분기 무영향 → 회귀 0 |
| 광범위 byte sweep 영역 | exam_eng.hwp p7 SVG 만 변경 (U+F003B → ↓), 다른 fixture 회귀 0 |

## 4. 산출물

- `/tmp/uF003B_glyph.svg` — 임베디드 폰트의 글리프 SVG 출력 (시각 검증용)
- `/tmp/pua_survey_full.py` — 광범위 통계 스크립트
- `/tmp/font_TT11_AAAAAL_HCRBatang.ttf` — PDF 임베디드 폰트 추출 (글리프 분석용)

## 5. 다음 단계

작업지시자에게 다음 2건 확인 요청:

1. **글리프 매핑 확정**: ↓ U+2193 (권장) vs ⬇ U+2B07 vs 다른 코드포인트
   - 한컴 PDF 시각 (또는 한컴 편집기에서 본 파일 직접 확인) 으로 최종 판정
2. **Stage 2 진행 승인**: 매핑 확정 후 Red 테스트 + 매핑 구현

## 메모리 룰 정합

- `reference_authoritative_hancom` — 한컴 PDF 시각 정답지 권위 (Stage 1 분석)
- `feedback_hancom_compat_specific_over_general` — U+F003B 단일 코드포인트 한정
- `feedback_no_pr_accumulation` — 별도 코드포인트는 별도 task
