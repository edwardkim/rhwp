# Task #836 Stage 1 (사전조사 + RED) 보고서

**브랜치**: `local/task836`
**선행**: 수행계획서 + 구현계획서 승인 완료
**목표**: 본질 결함 위치/카테고리 식별 + 회귀 테스트 작성
**진행 상태**: **본질 부분 확인 — 단일 결함 아닌 다층 가능성**. Stage 2 진입 전 추가 조사 필요.

## 산출물

### 1. fixture 추가
- `samples/3-09월_교육_통합_2022.hwp` (5.5 MB)
- `samples/3-09월_교육_통합_2023.hwp` (1.4 MB)
- `samples/3-10월_교육_통합_2022.hwp` (2.8 MB)
- `samples/3-11월_실전_통합_2022.hwp` (5.3 MB)
- `pdf/3-09월_교육_통합_2022-2022.pdf` (1.3 MB)
- `pdf/3-09월_교육_통합_2023-2022.pdf` (0.8 MB)
- `pdf/3-10월_교육_통합_2022-2022.pdf` (0.9 MB)
- `pdf/3-11월_실전_통합_2022-2022.pdf` (1.3 MB)

### 2. 진단 helper
`examples/diag_836.rs` — paragraph 별 measured vs IR_h 비교 + 카테고리 분류 (TEXT / EMPTY / SHAPE / TABLE)

## 본질 조사 결과

### 정합 확인 (정상 동작)

| 항목 | 결과 | 검증 방법 |
|---|---|---|
| ColumnDef 파싱 (2단) | ✅ | 직접 IR 추출 — `count=2 same_width spacing=850` |
| body_area + column_areas | ✅ | `PageLayoutInfo::from_page_def` — col[0]/col[1] 위치 정확 |
| per-paragraph height 측정 | ✅ DIFF=0 | `diag_836` 출력 — 468 paras 합산 정확 (TEXT/EMPTY/SHAPE/TABLE 모두 +0.0) |
| 머리말 표 구조 파싱 | ✅ | 2x4 table, "수능 모의고사" + DCT 로고 + "22-09 교육" 셀 정상 |
| `dump-pages` diff | +6.0px | 이전 -253px 보고는 측정 오류, 실제는 +6.0px (정합 범위) |
| ParaShape 파싱 | ✅ | spec (표 43) 준수: spacing_before/spacing_after/line_spacing/line_spacing_v2/line_spacing_type |
| LINE_SEG 파싱 | ✅ | spec (표 62) 준수: vpos/lh/th/bl/ls 정확 |

### 부정 가설

| 가설 | 검증 | 결과 |
|---|---|---|
| line_spacing 누락 (paragraph_layout) | per-para 측정 비교 | ❌ DIFF=0, 누락 부재 |
| `height_for_fit` 다단 trailing_ls 누락 (Task #359) | 패치 실험 (4 호출 모두 `total_height` 변경) | ❌ 페이지 수 9→11 (기대 23, 효과 미미) |

### 미확정 (추가 조사 필요)

**핵심 발견 — 페이지 수 큰 차이**:
| Sample | rhwp pages | 한컴 PDF pages | 차이 |
|---|---:|---:|---:|
| 3-09월_2022 | 9 | **23** | -14 (60% 적음) |
| 3-09월_2023 | 10 | 20 | -10 |
| 3-10월_2022 | 9 | 18 | -9 |
| 3-11월_2022 | 10 | 21 | -11 |

**시각 비교** (rsvg-convert + pdftoppm):
- Page 1 시각 거의 동일 (9 problems × 2 column)
- Page 2 부터 rhwp 가 한 페이지에 ~2x 더 많은 problem packing
- 차이의 본질은 **페이지/단별 content packing 밀도**

### 가능 원인 (더 깊은 조사 필요)

1. **PDF 측 추가 페이지 분할 로직** — 한컴오피스 viewer 가 "다음 문단과 함께" / "외톨이줄 보호" / "문단 보호" 등 ParaShape attr1 bit 16~18 적용
2. **글꼴 metrics 차이** — rhwp 의 폰트 fallback (한컴 함초롬 → Pretendard 등) 으로 라인 높이/측정 미세 차이 누적
3. **AutoNumber (문제 번호) 미렌더** — PDF 의 "문1)", "문2)" 등 번호 prefix 가 rhwp 에서 부재 (별도 결함 가능성, 페이지 수에 영향 없음)
4. **빈 paragraph height** — HWP intent vs rhwp default 의 차이 (현재 default 400 HU = 5.3px)
5. **페이지 분할 vpos-reset 처리** — page break / column break 시점 차이

## HWP5 spec 정합 검증 (`mydocs/tech/한글문서파일형식_5.0_revision1.3.md`)

| spec 표 | 항목 | rhwp 정합 |
|---|---|---|
| 표 43 (ParaShape) | spacing_before / spacing_after / line_spacing | ✅ 파싱 정확 |
| 표 44 (속성1) | bit 16 외톨이줄 / bit 17 함께 / bit 18 보호 / bit 19 쪽나눔 | ⚠️ 파싱 여부 미확인 — Stage 2 조사 필요 |
| 표 46 (줄 간격 종류) | bit 0~4: 0/1/2/3 | ✅ |
| 표 62 (LINE_SEG) | vpos / line_height / text_height / baseline / line_spacing | ✅ 파싱 정확 |

**주요 미점검 영역**: ParaShape 속성1 의 페이지 분할 관련 비트 (bit 16~19) — 한컴 viewer 가 이 비트들을 페이지 분할에 활용 가능, rhwp 측 적용 여부 조사 필요.

## RED 테스트

본 Stage 에서 회귀 테스트 작성 미완 — 본질 미확정으로 assertion 정의 불가. Stage 2 가설 확정 후 작성 (예: "page count = PDF page count" 단순 assertion).

## 다음 단계 진행 방향 제안

작업지시자 결정에 따라:
- **A**: Stage 1 확장 — ParaShape attr1 bit 16~19 처리 조사 + 빈 paragraph height + 폰트 metrics 정합 확인
- **B**: Stage 2 임시 정정 시도 — height_for_fit 외 다른 후보 단위 패치 + 페이지 수 효과 확인 (반복 실험)
- **C**: 외부 권위 자료 비교 강화 — `ir-diff` 명령으로 HWP↔HWPX (한컴 변환본) IR 비교 → 한컴이 어떻게 다르게 처리하는지 단서

## 산출물

- `examples/diag_836.rs` — 진단 helper
- 4 sample + 4 PDF fixture
- 본 보고서

본질 부분 확정 + Stage 2 진입 조건 미충족. 작업지시자 방향 결정 후 진행.
