# Task #314 1단계 완료 보고서: LINE_SEG 변환 전후 비교

상위: 구현 계획서 `task_m100_314_impl.md`

## 진단 도구

`tests/task314_diag.rs` 임시 진단 테스트 작성. 작업 완료 후 제거 예정.

## 핵심 발견

### A. LINE_SEG는 100% 보존됨

vpos / line_height / line_spacing / text_height 모두 direct (HWPX) 와 reloaded (어댑터 + 직렬화 + 파싱) 사이 **0건 차이**. 어댑터 코멘트의 가정 ("vpos가 보존됨")이 정확.

### B. paragraph 필드 차이 (origin 후보)

| 필드 | 차이 건수 | 설명 |
|------|----------|------|
| `raw_header_extra` | 130 (전체) | paragraph header 끝 extra 바이트. HWPX 파서 비채움 |
| `char_shapes_len` | 59 | empty → `[(0,0)]` (HWP 직렬화기 default 강제) |
| `control_mask` | 27 | bit 11(0x800) 차이. HWPX 파서가 controls와 동기화 누락 |
| `raw_break_type` | 4 | direct=0x00, reloaded=0x04 (column_type=Page 기반 재계산) |
| `controls_len` | 2 | 어댑터가 SectionDef 삽입 (의도된 차이) |
| `char_count` | 3 | 미세한 char count 차이 |
| `char_count_msb` | 2 | 마지막 paragraph 표시 |

### C. 페이지 수 차이 발생 위치

페이지별 paragraph index 비교:
- 페이지 1, 2: direct/reloaded 동일
- **페이지 3에서 차이 시작**: direct는 pi=33..51 (Table pi=51 + PartialParagraph 포함), reloaded는 pi=33..50까지
- 이후 페이지가 1씩 밀려서 마지막에 +1쪽

페이지 3 단 0:
- direct: items=20, used=1268.5px (column h=933.6 초과 — 어울림 표 처리)
- reloaded: items=18, used=595.5px

direct는 같은 paragraph를 더 작은 누적 height로 처리. reloaded는 더 큰 누적 → 일찍 페이지 break.

## 식별된 차이의 typeset 영향 분석

| 필드 | typeset 영향 |
|------|--------------|
| LINE_SEG | 미사용 (이미 동일) |
| `control_mask` | renderer 미사용 (검증됨) |
| `raw_break_type` | typeset은 column_type만 사용. 미영향 |
| `char_shapes` | composer에서 first().unwrap_or(0). [] 와 [(0,0)] 모두 0 사용 → 미영향 |
| `raw_header_extra` | 미사용 (paginate 영향 없음) |

이론적으로는 typeset 결과가 같아야 하는데 페이지 수 다름. **추가 변수 존재**.

## 가설 (2단계에서 검증)

이론적으로 영향 없는 차이가 실제로는 paginate 결과를 바꾸는 어떤 경로가 있다. 후보:
1. composer가 char_shapes 길이로 분기하는 케이스가 있을 수 있음
2. height_measurer가 paragraph 속성을 다르게 해석
3. ctrl_data_records 직렬화/파싱 라운드트립 손실
4. 표/Shape 의 다른 필드 차이

가장 빠른 검증: **HWPX 로드 후 normalize** (char_shapes 빈 것을 [(0,0)] 채우기, control_mask 재계산 등)를 적용하여 direct IR을 reloaded IR과 동일하게 만든 뒤 page count 비교.

## 다음 단계

2단계: HWPX 로드 시점에 normalize 적용 → direct page count 변화 측정 → reloaded와 일치하는지 확인. 일치하면 origin 확정. 불일치하면 더 깊은 조사.

## 산출

- `tests/task314_diag.rs` (임시 진단 테스트, 4단계에서 제거)
- 본 보고서
