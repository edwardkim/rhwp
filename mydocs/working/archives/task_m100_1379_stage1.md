# Task M100 #1379 1단계 완료 보고서 — 게이트 강화 + 전수 측정 + 미보존 항목 분류

- 구현계획서: `mydocs/plans/task_m100_1379_impl.md` 1단계
- 브랜치: `local/task1379`

## 1. 수행 내역

### 1.1 게이트 강화 — `diff_documents` 컨트롤 비교 (`src/serializer/hwpx/roundtrip.rs`)

- `IrDifference::ParagraphControls { section, paragraph, path, expected, actual }` variant 추가
  — 문단별 **인라인 슬롯 컨트롤 타입 시퀀스** 비교 (예: `expected=[pic] actual=[]`)
- 비교 대상은 `is_hwpx_inline_slot()` 기준 (section.rs 가시성 `pub(crate)` 조정).
  Bookmark 등 비슬롯 컨트롤은 serializer 가 문단 선두로 재배치하므로 비교에서 제외 —
  파서 적재 순서와 serializer 방출 순서의 정합 기준을 슬롯 컨트롤로 한정
- #1378 재귀 비교(`diff_paragraph_char_shapes` — 셀·글상자 Group·각주/미주)에 동승:
  중첩 subList 의 컨트롤 소실도 path(`/ctrl[0]tbl.cell[0].p[0]` 형식)로 검출
- `control_type_name()` 16종 매핑 (tbl/pic/shape/eq/fn/en/field/form/header/footer/
  autoNum/pageHide/pageNumPos/newNum/charOverlap/ruby)
- 단위 테스트 5건 추가: 본문 소실 / 셀 소실 / 타입 변경 / 비슬롯(Bookmark) 무시 /
  동일 시퀀스 통과 — `cargo test --lib serializer::hwpx::roundtrip` 17 passed

### 1.2 전수 사전 측정 (강화 게이트 baseline)

`cargo test --test hwpx_roundtrip_baseline` 사전 실행 결과 — **신규 실패 16 샘플 / 224 diff,
전건 `expected=[…] actual=[]` 형태의 셀·글상자 subList 소실. 본문 경로 0건.**

| 샘플 | diff | 소실 내역 |
|------|-----:|----------|
| 2024년 1분기 보도자료 ff | 3 | 셀 pic·pageNumPos |
| 2024년 연간 보도자료 _ ff | 3 | 셀 pic·pageNumPos |
| 2025년 1분기 보도자료f | 3 | 셀 pic·pageNumPos + 글상자 pic |
| [2027] 온새미로 | 1 | 셀 shape |
| exam_social-p1 | 5 | 셀 shape×2 + 중첩 tbl×3 |
| footnote-01 | 1 | 셀 fn(각주) |
| footnote-tbox-01 | 1 | 글상자 fn(각주) |
| form-002 | 190 | 셀 form 다수 + 중첩 tbl |
| hwpx-h-01 | 3 | 셀 pic·pageNumPos |
| hwpx-h-03 | 3 | 셀 pic·pageNumPos + 글상자 pic |
| hy-001 | 2 | 셀 pic + 글상자 pic |
| hy-002 | 2 | 셀 pic + 글상자 pic |
| issue_1133 | 1 | 셀 중첩 tbl |
| mel-001 | 3 | 셀 charOverlap×3 |
| ta-pic-001-r | 2 | 셀 pic (이슈 대표 — hp:pic 4개 소실) |
| tb-img-03 | 1 | 셀 pic |

- 검출 타입 인벤토리: pic / form / tbl(중첩 표) / shape / fn / pageNumPos / charOverlap
  — 전건 본 타스크 2~3단계 수정 범위. 범위 밖 신규 실패 없음
- `XFAIL_1378_RECURSIVE` 12건(#1379 기인분)은 char_shapes 경계 시프트와 컨트롤 diff 가
  동일 원인(소실)으로 교차 검출됨을 확인 — 2단계 해소 시 일괄 승격
- **발견**: `render_control_slot()`(section.rs:595)에 CharOverlap·Ruby arm 부재(`_ => {}`)
  — mel-001 셀 charOverlap 3건이 해당. 2단계 전환 시 arm 보강 필요 (전환만으로는 미해소)

### 1.3 임시 xfail 등록 (`tests/hwpx_roundtrip_baseline.rs`)

- `XFAIL_1379_CONTROLS` 16건 등록 — 각 사유와 해소 단계(2단계/3단계) 명기
- skip 블록은 `eligible += 1` 뒤 배치 (LARGE "대상 없음" 가드 보호, #1378 패턴 동일)
- `xfail_1379_controls_entries_still_fail` still-fail 가드 추가 — 해소 시 승격 강제
- 결과: `cargo test --test hwpx_roundtrip_baseline` 6 passed

### 1.4 미보존 항목 분류 (승인 요청 대상)

원본 XML 전수 대조 결과 (parser/hwpx/section.rs · model/shape.rs 확인):

| 항목 | 전수 사용 현황 | IR 보존 | 분류 제안 |
|------|---------------|---------|----------|
| drawText `name`/`editable` | **사용 0건** (전 샘플 기본값) | 미보존 | **후속 분리** — 한컴 열기와 무관, 실측 수요 없음 |
| rect `numberingType` | **광범위** (exam_kor 22건 등) | 미보존 (`numbering_type_picture` bool 만) | **본 타스크 포함** — 3단계에서 모델 필드 추가 + 파서 적재 + 방출 |
| drawText `textDirection` VERTICAL/VERTICALALL 구분 | k-water-rfp 3건·tbox-v-flow-01 1건이 VERTICALALL | 부분 보존 (`list_attr` bit 0~2 에 code 1 로 합쳐짐) | **본 타스크 포함** — tbox-v-flow-01 한컴 열기 판정 대상이므로 `TextBox` 보존 필드 추가 |
| rect pt0~pt3 / lineShape / fillBrush / shadow / instid / offset·orgSz·curSz·flip·rotationInfo·renderingInfo | — | **전부 보존** | serializer 방출만 (3단계, 계획대로) |

요소 순서는 원본 샘플 실측으로 확정: rect 자식은
`offset→orgSz→curSz→flip→rotationInfo→renderingInfo→lineShape→fillBrush→shadow→
drawText→pt0~pt3→sz→pos→outMargin` (구현계획서 3.3 순서와 일치).

## 2. 검증

- `cargo test --lib` 1690 passed
- `cargo test --test hwpx_roundtrip_baseline` 6 passed (XFAIL_1379_CONTROLS 등록 상태)
- `cargo fmt -- --check` (수정 파일 3건) / `cargo clippy --lib --tests -- -D warnings` 통과

## 3. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/serializer/hwpx/roundtrip.rs` | ParagraphControls variant + 비교 로직 + 테스트 5건 |
| `src/serializer/hwpx/section.rs` | `is_hwpx_inline_slot` 가시성 `pub(crate)` |
| `tests/hwpx_roundtrip_baseline.rs` | XFAIL_1379_CONTROLS 16건 + skip + still-fail 가드 |

## 4. 승인 요청 사항

1. 1.4 분류안 — name/editable **후속 분리**, numberingType·VERTICALALL 구분 **본 타스크 포함**
2. 2단계(셀 경로 전환) 착수 — 단, render_control_slot 의 charOverlap arm 보강을
   2단계 범위에 추가 (mel-001 해소에 필수, 전환만으로는 미해소)
