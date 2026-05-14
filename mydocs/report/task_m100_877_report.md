# Task #877 최종 결과 보고서 — hwp3-sample16.hwp WASM 로드 실패 + 시각 정합

**이슈**: [edwardkim/rhwp#877](https://github.com/edwardkim/rhwp/issues/877)
**브랜치**: `local/task877_v2` (분기: `local/task873`, 19 commits)
**마일스톤**: v1.0.0 (M100)
**기간**: 2026-05-13 ~ 2026-05-14

## 개요

`samples/hwp3-sample16.hwp` (2.9 MB, 64쪽 RFP 문서, 한국수자원공사 2004.11) 를 rhwp-studio 에서 열면 panic 으로 로드 실패:

```
panicked at library/alloc/src/raw_vec/mod.rs:28:5: capacity overflow
[main] 파일 로드 실패: RuntimeError: unreachable
```

본 task 는 panic 차단부터 시작하여 시각 정합까지 단계적으로 해결.

## 최종 결과

### sample16.hwp 의 변화

| 항목 | 초기 (panic) | Stage 1 후 | Stage 2 후 | Stage 4 최종 | 한컴 viewer |
|------|----------|---------|---------|---------|---------|
| Panic | ❌ 발생 | ✅ 차단 | ✅ | ✅ | (n/a) |
| 문단 수 | 0 | 77 | 1058 | 1058 | 1058 ✓ |
| 페이지 수 | (실패) | 28737 | 65 | 64 | 64 ✓ |
| 페이지 2 | (실패) | 빈 | 빈 | **목차** | 목차 ✓ |
| 표지 RFP 박스 | (실패) | ❌ | ❌ | **✅** | ✓ |
| 16쪽 본문 외곽선 | (실패) | ❌ | ❌ | **✅** | ✓ |
| Ⅰ~Ⅹ 로마숫자 | (실패) | ❌ | ❌ | **✅** | ✓ |
| 1단계 글머리 ○ | (실패) | ❌ | ❌ | **✅** | ✓ |
| 2단계 글머리 ◦ | (실패) | ❌ | ❌ | **✅** | ✓ |
| 16쪽 다이어그램 | (실패) | ❌ | ❌ | **✅** | ✓ |

## Stage 별 작업 요약

### Stage 1 (b4a4c6b) — Allocation sanity check

**문제**: HWP3 length-prefixed `vec![0u8; length]` 가 garbage length (0xDC000000 = 3.69 GB) 로 호출되어 32-bit WASM 의 `RawVec` capacity overflow panic.

**수정**:
- `HWP3_MAX_RECORD_SIZE = 256 MB` cap
- `alloc_record_buf` / `check_record_count` 공통 helper
- 8개 위치 가드 (`Hwp3AdditionalInfoBlock`, picture `ext_buf`, drawing object 등)

**결과**: WASM panic 차단, native 도 graceful Err 반환.

### Stage 2 (ce04375) — Special char alignment (ch=5/6/7/8)

**문제**: HWP3 paragraph 70 의 `ch=6 책갈피` record 가 spec 의 42 byte 가 아닌 8 byte 로 처리 → 후속 paragraph stream 34 byte 어긋남 → paragraph 71 부터 garbage cc/lc → 28737 페이지 오인식.

**진단**: `mydocs/tech/한글문서파일구조3.0.md` §10.2~§10.4 spec 대조:
- ch=5 (필드코드): 가변 (8+n byte) — n byte 누락
- ch=6 (책갈피): 42 byte — 34 byte 누락
- ch=7 (날짜형식): 84 byte — 76 byte 누락
- ch=8 (날짜코드): 96 byte — 88 byte 누락

**결과**: **77 → 1058 문단 / 28737 → 65 페이지** (한컴 HWP5 변환본 정확 일치).

### Stage 3 (7f35fa3, b0bf58f, 2d737be, 9fbb798) — 시각 차이 4건 중 3건

| Commit | 내용 |
|--------|------|
| 7f35fa3 | HWP3 사적 인코딩 0x3590~0x3599 → Ⅰ~Ⅹ (U+2160~U+2169) 매핑 |
| b0bf58f | drawing line_style=0 + width>0 → Solid LineType 보강 |
| 2d737be | drawing object 모든 variant 의 treat_as_char 검사 (빈 페이지 2 해소: 65→64 페이지) |
| 9fbb798 | HWP3 0x3366 → PUA U+F03C5 (1단계 글머리) 매핑 |

### Stage 4 (b647227, 5b70dfc, ab1fd83, acf3b09, 202cef9, 8008501, **00d6bba**, c8ba53b, 648c2cb) — 잔여 처리 + PDF 정합

| Commit | 내용 |
|--------|------|
| b647227 | HWP3 paragraph margins 패턴 기반 ◦ 글머리 자동 prefix 휴리스틱 (sample16 25개) |
| 5b70dfc | image magic detection 에 **WMF / EMF 추가** — 16쪽 다이어그램 표시 |
| ab1fd83 | 점선 (LineType 2~7) 가시성 최소 1.0 px 보강 |
| acf3b09 | PUA U+F03C5 → ○ (U+25CB) 변경 + picture ref_pos=0 위치 정합 |
| 202cef9 → 8008501 | fill_color high flag (0x10000000) 처리 — RGB=0+flag → 흰색 |
| **00d6bba** | **★ 근본 원인 ★** HWP3 drawing Fill.alpha = 0 (한컴 convention) — alpha=255 → opacity=0 완전 투명 회귀 해소 |
| **c8ba53b** | **HWP3 doc_info 페이지 외곽선 IR 변환** — PDF 정합 (모든 페이지 외곽 box) |
| 648c2cb | ◦ 글머리 휴리스틱 ls=145 확장 (paragraph 396/397/398/399 등 16쪽 본문) |

#### Stage 4 의 핵심 발견 (00d6bba)

SVG export 결과 `<rect ... opacity="0.000"/>` — Rectangle 자체가 완전 투명. renderer 의 opacity 계산 ([renderer/layout/utils.rs:199](../../src/renderer/layout/utils.rs#L199)):

```rust
opacity = 1.0 - alpha/255   // alpha=0 → opacity=1 (불투명), alpha=255 → 0 (투명)
```

이는 **한컴 convention** (`0=불투명`) 인데, HWP3 drawing 변환 시 표준 HWP5 convention (`255=불투명`) 으로 잘못 설정 → 부호 반대로 적용되어 완전 투명. Fill.alpha=0 으로 수정.

**이전 모든 fix (외곽선 / 글머리 / WMF) 가 IR 단에서 정확했음에도 화면에 안 보였던 진짜 원인** — opacity=0 으로 모든 drawing object 가 invisible.

## 검증

### cargo test
```
total: passed: 1381, failed: 0
```

### HWP3 sample 6종 회귀 없음
| 샘플 | 문단 수 | 페이지 수 |
|------|--------|----------|
| hwp3-sample.hwp | 195 | 16 |
| hwp3-sample10.hwp | 26767 | 763 |
| hwp3-sample13.hwp | 71 | 3 |
| hwp3-sample14.hwp | 256 | 11 |
| hwp3-sample4.hwp | 1273 | 36 |
| hwp3-sample5.hwp | 1931 | 64 |

### 신규 단위 테스트
- `test_alloc_record_buf_overflow_returns_err` — overflow Err 반환
- `test_alloc_record_buf_within_cap_ok` — 정상 범위 Ok
- `test_check_record_count_overflow_returns_err` — count 가드
- `test_hwp3_sample16_load_alignment` — paragraph >= 1000 검증

## 변경 파일

### 소스
- `src/parser/hwp3/mod.rs` — 가드 helper, ch=5/6/7/8 alignment, picture ref_pos, outline bullets 휴리스틱, WMF magic
- `src/parser/hwp3/records.rs` — InfoBlock / AdditionalInfoBlock 가드
- `src/parser/hwp3/drawing.rs` — 가드, line_style 보강, fill_color flag, alpha convention
- `src/parser/hwp3/ole.rs` — OleInfo 가드
- `src/parser/hwp3/johab.rs` — 로마숫자 + PUA → ○ 매핑
- `src/renderer/layout/utils.rs` — 점선 가시성 보강

### 문서
- `mydocs/plans/task_m100_877.md` — 수행 계획서
- `mydocs/plans/task_m100_877_impl.md` — 구현 계획서
- `mydocs/working/task_m100_877_stage{1,2,3,4}.md` — 단계별 보고서
- `mydocs/tech/hwp3_paragraph_border_fill_analysis.md` — border_fill 분석
- `mydocs/report/task_m100_877_report.md` — 본 최종 보고서

### 신규 sample / pdf
- `samples/hwp3-sample16.hwp` (2.9 MB)
- `samples/hwp3-sample16-hwp5.hwp` (3.0 MB)
- `samples/hwp3-sample16-hwp5.hwpx` (3.1 MB)
- `pdf/hwp3-sample16-hwp5-2022.pdf` (2.2 MB)

## 잔여 / 후속 task

### 본 task 범위 외 (별도 task)

| 항목 | 비고 |
|------|------|
| **HWP3 alpha convention 통일** | drawing.rs 의 alpha=0 convention 이 한컴 사적. HWP5/HWPX 파서 + 모델 / renderer 통일 검토 별도 task. |
| **HWP3 PUA chars 전반적 매핑** | 0x3366 외 한컴 사적 PUA chars (다양한 글머리/장식 char) 의 표준 unicode 또는 PUA → glyph fallback 매핑. 별도 task 에서 cross-ref 도구로 진행. |
| **paragraph border_fill 자동 부여** | 분석 결과 본 task 에서 불필요 (`mydocs/tech/hwp3_paragraph_border_fill_analysis.md`). 향후 시각 회귀 발견 시 별도 task. |

## 결론

본 task 의 원래 목표 (WASM panic 차단 + paragraph alignment 정합) 를 넘어 **sample16 의 한컴 viewer 정합 시각 표시까지 완전 달성**:
- 문단 / 페이지 / 표지 박스 / 16쪽 본문 박스 / 글머리 / 다이어그램 모두 한컴 정합

특히 Stage 4 의 마지막 발견 (Fill.alpha convention 부호 반대) 이 이전 모든 fix 들의 효과를 가시화한 진짜 근본 원인.
