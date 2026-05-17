# Task #959 Stage 5 — 시각 검증 + 최종 작업 정리

## 1. 한컴 PDF 정합 검증

`pdf/3-11월_실전_통합_2022.pdf` page 1 vs rhwp page 1:
- 좌측 단 문1~문5: 정합 ✓
- 우측 단 문6~문9: 정합 ✓
- 문9 위치: 한컴 ~y 810 vs rhwp y=787~805 ✓
- 우측 단 picture: 한컴 invisible, rhwp 도 column flow 에서 제외 (column 외부 emit) ✓

## 2. 최종 변경 요약

### 2.1 코드 변경
`src/renderer/layout.rs` 라인 3500~3556:
- **Fix C**: horz_rel_to=Column picture 가 col_area 우측 외부 emit 시 cursor advance skip
- `RHWP_DEBUG_TAC_CURSOR` 환경변수 영구화 (paragraph item 별 y_offset 추적 도구)

### 2.2 문서 추가
- `mydocs/plans/task_m100_959.md`
- `mydocs/plans/task_m100_959_impl.md`
- `mydocs/plans/task_m100_959_impl_v2.md`
- `mydocs/working/task_m100_959_stage1.md`
- `mydocs/working/task_m100_959_stage4.md`
- `mydocs/working/task_m100_959_stage5.md`
- `mydocs/report/task_m100_959_report.md`
- `mydocs/orders/20260517.md` 갱신

### 2.3 신규 fixture
- `samples/3-09월_교육_통합_2022.{hwp,hwpx}`, `3-09월_교육_통합_2023.{hwp,hwpx}`
- `samples/3-10월_교육_통합_2022.{hwp,hwpx}`, `3-11월_실전_통합_2022.{hwp,hwpx}`
- `pdf/3-09월_교육_통합_2022.pdf` 외 3개 — 한컴 2022 권위 PDF

(샘플 + PDF 는 PR #956 에 포함 예정이나 #956 머지 전이므로 본 PR 에도 포함)

## 3. PR 준비

- base = upstream/devel
- head = jangster77:local/task959
- title: fix: horz_rel_to=Column picture 의 column 외부 emit 시 cursor advance skip — 시험지 page 1 문9 정합 (closes #959)
