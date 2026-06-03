# Task #528 Stage 1 — 폰트 후보 검증 + 본질 발견

**작성일**: 2026-05-02
**이슈**: [#528](https://github.com/edwardkim/rhwp/issues/528)
**브랜치**: `local/task528`

## 1. 결론 (Executive Summary)

> 본 task 의 **초기 가설 (Hangul Jamo Extended-A/B 폰트 미지원) 이 부정확**함이 측정으로 확정됐다.
>
> 실제 본질: `samples/exam_kor.hwp` p17 의 옛한글 음절은 **HanCom Private Use Area (PUA) 인코딩** 으로 저장. 폰트 fallback 만으로는 해결 불가. **HncPUAConverter 정합 — PUA → KS X 1026-1:2007 유니코드 자모 시퀀스 변환** 이 필수 선행.

이 본질은 [Issue #512](https://github.com/edwardkim/rhwp/issues/512) 에 이미 등록되어 있다.

## 2. 측정 데이터

### 2-1. 추출 명령

```bash
target/release/rhwp export-text samples/exam_kor.hwp -p 16 -o /tmp/exam_kor_text
# → /tmp/exam_kor_text/exam_kor_017.txt (2,524 chars)
```

### 2-2. 코드포인트 분포

```python
# 분석 스크립트 (/tmp/exam_kor_text/exam_kor_017.txt)
```

| 영역 | 발견 | 본 task 가설 정합 |
|------|------|-------------------|
| Hangul Syllables (U+AC00-D7AF) | 1362 회, 268 unique | (정상 렌더) |
| Compatibility Jamo (U+3130-318F) | 38 회, 11 unique | 단일 자모 → 정상 렌더 |
| **Hangul Jamo (U+1100-11FF)** | **0** | ❌ 본 task 가설 영역, **부재** |
| **Hangul Jamo Extended-A (U+A960-A97F)** | **0** | ❌ 본 task 가설 영역, **부재** |
| **Hangul Jamo Extended-B (U+D7B0-D7FF)** | **0** | ❌ 본 task 가설 영역, **부재** |
| **Basic PUA (U+E000-F8FF)** | **50 회, 22 unique** | ★ 옛한글 음절 영역 |
| **Supplementary PUA-A (U+F00DA, F0854, F0855)** | **68 회, 3 unique** | ★ 옛한글 음절 영역 |

### 2-3. PUA 코드포인트 상세

**Basic PUA (U+E000-F8FF) — 22 unique**:
```
U+E17A x1, U+E1A7 x2, U+E1C2 x1, U+E288 x1, U+E38A x5, U+E40A x1,
U+E474 x1, U+E560 x1, U+E566 x1, U+E79C x1, U+E8A7 x2, U+E8B2 x3,
U+E95B x3, U+EB66 x1, U+EB68 x2, U+EBD4 x1, U+ECF0 x4, U+ECFB x2,
U+ED41 x1, U+ED98 x1, U+ED9A x1, U+F152 x1, U+F154 x1, U+F1C4 x1,
U+F537 x5
```

**Supplementary PUA-A — 3 unique**:
```
U+F00DA x2, U+F0854 x33, U+F0855 x33
```

→ p17 만 약 25 종 PUA 옛한글 음절. 광범위 옛한글 사용 HWP 문서 (예: 중세국어 교재) 는 더 많을 것으로 추정.

## 3. 본질 정정

### 3-1. 한컴 매뉴얼 정합 ([hncpuaconverter.htm](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/hwp/Help/extracted/hwpbase/hncpuaconverter.htm))

> 이전 판 한/글의 옛한글은 유니코드 PUA 사용자 영역 (E000~F8FF) 에 정의된 옛한글 완성자를 사용하거나 PUA 영역의 옛한글을 조합하여 표현했습니다.
>
> 한/글 2010 의 옛한글은 산업자원부 기술표준원에서 제정된 "정보교환용 한글 처리지침 (KS X 1026-1:2007)" 을 기준으로 유니코드의 자모 영역 조합하여 옛한글을 입력하도록 개발되었습니다.

→ 한컴 자체적으로 PUA → KS X 1026-1:2007 자모 시퀀스 변환 도구 (`HncPUAConverter.exe`) 를 별도 제공.

### 3-2. exam_kor 의 PUA 영역

`samples/exam_kor.hwp` 는 한컴 2010+ 환경에서 작성됐으나 옛한글 입력 영역은 **이전 판 호환 영역 (PUA 인코딩)** 으로 보존된 것으로 추정. 한컴 PDF 출력 시 한컴 자체 폰트 (함초롬바탕 LVT) 가 PUA 글리프를 직접 보유하여 정상 렌더.

rhwp 는:
- 함초롬바탕 LVT 라이선스 미지원 (한컴 전용)
- Source Han Serif K / Noto Serif KR 등 OFL 폰트는 **PUA 영역에 옛한글 글리프 없음** (KS X 1026-1 자모 영역만 지원)

### 3-3. 폰트 fallback 단독으로 해결 불가

본 task 의 초기 가설:
> 시스템 폰트 + Noto Serif KR 가 Hangul Jamo + Extended-A/B 합자 미지원

→ 가설 자체는 **사실** (이 영역은 실제로 fallback 미흡) 이지만 **exam_kor 의 결함 원인은 다른 영역** (PUA 인코딩).

→ 본 task 가 폰트 fallback 만 처리하면:
- Hangul Jamo Extended-A/B 영역의 잠재 결함은 보강
- **exam_kor p17 의 실제 증상은 해결 안 됨** (PUA 코드포인트가 변환 안 됨)

## 4. 해결안 정정

### 4-1. PUA 변환 우선

```
[PUA 코드포인트] → [HncPUAConverter 정합 매핑] → [KS X 1026-1:2007 자모 시퀀스] → [폰트 합자 렌더]
       ↑                       ↑                          ↑                          ↑
       원본 IR                Stage 2-3                  변환 후 IR                Stage 4-5
                              (#528 = #512 흡수)         (Jamo 영역)            (font fallback)
```

### 4-2. 매핑 표 확보 영역

후보 자료원:
1. **KS X 1026-1:2007 부속서** — 정식 표준 (한국산업표준 ks 사이트, 유료)
2. **HncPUAConverter.exe 역공학** — Windows 환경에서 PUA 입력 → 변환 출력 비교 (라이선스 제약)
3. **함초롬바탕 LVT 폰트 cmap** — 폰트 자체에 PUA 코드포인트 → glyph 매핑 존재. 이 글리프를 KS X 1026-1 자모 시퀀스로 역추적 (라이선스 제약)
4. **오픈소스 변환 도구** — `nuhwp`, `hwp.js`, `pyhwp` 등의 PUA 매핑 추출 가능성
5. **rhwp `gen-pua` + 한컴 출력 PDF 비교** — Task #509 패턴 정합. PUA 코드포인트 입력 HWP → 한컴 편집기 → PDF 캡처 → 시각 매핑

### 4-3. 변환 적용 위치

| 옵션 | 장점 | 단점 |
|------|------|------|
| **파싱 단계** (`src/parser/hwp5/`) | IR 자체에 정규화된 자모 시퀀스 → 모든 렌더러 (SVG/Canvas/PDF) 일괄 적용 | 파싱 IR 변경 → 다른 영역 영향 검증 필요 |
| **렌더링 단계** (`src/renderer/composer.rs`) | 파싱 영역 영향 없음. 기존 `convert_pua_enclosed_numbers` 패턴 정합 | 렌더러별 별도 처리 필요 (svg / web_canvas / paint::json) |
| **Composition 단계** (Composer) | 렌더 직전 단일 진실 원천. 폭 계산도 합자 후 폭으로 정합 | 자모 결합 알고리즘 + LINE_SEG 영향 |

→ **Composer 단계 추천** — Task #122 에서 이미 자모 클러스터 폭 계산 인프라 구축됨.

## 5. 본 task 의 Stage 재정의

### 변경 전 (가설 기반)

```
Stage 1: 폰트 후보 검증     ← 본 단계에서 가설 부정확 발견
Stage 2: subset 추출
Stage 3: fallback 체인
Stage 4: 문서 + 회귀
Stage 5: 시각 판정
```

### 변경 후 (본질 기반)

```
Stage 1: PUA 매핑 표 확보 + 본 발견 (현재 단계 — 본 보고서)
Stage 2: PUA → KS X 1026-1 자모 변환 함수 + 매핑 표 코드 영역
Stage 3: Composer 단계 변환 적용 + 자모 결합 (Task #122 인프라 활용)
Stage 4: 폰트 fallback 보강 (변환 후 자모 영역 — 원래 본 task 가설 영역)
Stage 5: 광범위 회귀 + 작업지시자 시각 판정
```

## 6. 영향

### 6-1. Issue 영향

- **#528** (본 task) — 가설 영역 폰트 fallback 단독 → PUA 변환 + 폰트 fallback 통합으로 피벗
- **#512** — 본 task 흡수, close 후 #528 referenced

### 6-2. 작업량 영향

| 항목 | 변경 전 | 변경 후 |
|------|--------|---------|
| 작업 영역 | 폰트 fallback (정적 자산 + style_resolver) | **PUA 매핑 + 변환 함수 + Composer 통합 + 폰트 fallback** |
| 매핑 표 영역 | 없음 | **신규 — 핵심 작업** |
| 회귀 위험 | 중 (style_resolver) | **중-고** (Composer 단계 변환 → 모든 렌더러 영향) |
| 단계 수 | 5 | 5 (Stage 영역 재정의) |

### 6-3. 메모리 정합

- `feedback_pdf_not_authoritative` — 한컴 PDF 는 절대 기준 아님 → PDF 비교 + 작업지시자 시각 판정
- `feedback_essential_fix_regression_risk` — 본질 정정 회귀 위험 → 광범위 샘플 + 한컴 환경 검증
- `feedback_rule_not_heuristic` — KS X 1026-1:2007 은 **표준 룰** (한컴 자체 매뉴얼 정합) → 휴리스틱 분기 추가 금지

## 7. 다음 단계

본 보고서 작성 + 작업지시자 승인 후:

1. **수행계획서 (`task_m100_528.md`) 재작성** — 새 본질 영역 정합
2. **구현계획서 (`task_m100_528_impl.md`) 재작성** — 새 Stage 정의
3. **이슈 #528 본문 갱신** — 가설 정정 + 새 영역 명시
4. **이슈 #512 close** — #528 흡수 referenced
5. **승인 후 Stage 2 (매핑 표 확보) 진행**

## 8. 산출물 (본 단계)

| 산출물 | 내용 |
|--------|------|
| 본 보고서 (`mydocs/working/task_m100_528_stage1.md`) | 본질 발견 + 해결안 정정 |
| 측정 데이터 (`/tmp/exam_kor_text/exam_kor_017.txt`) | exam_kor p17 텍스트 |
| 코드포인트 분포 분석 | 본 보고서 §2-2, §2-3 |
| 코드 변경 | **0** (본 단계는 조사만) |
