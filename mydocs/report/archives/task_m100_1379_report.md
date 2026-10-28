# Task M100 #1379 최종 결과보고서 — HWPX serializer: subList 내부 컨트롤 보존

- 이슈: #1379 "HWPX serializer: subList 내부 컨트롤 보존 — 표 셀·글상자 내 이미지 직렬화" (M100, #1315 하위)
- 브랜치: `local/task1379`
- 계획서: `mydocs/plans/task_m100_1379.md`, `task_m100_1379_impl.md`
- 단계별 보고서: `mydocs/working/task_m100_1379_stage{1..3}.md`

## 1. 결과 요약

표 셀·글상자 subList 문단의 컨트롤(그림/중첩 표/각주/글자겹침/다단 정의 등)이
직렬화에서 통째로 빠지던 결함을 해소했다. 셀·글상자 문단을 본문과 동일한
`render_paragraph_parts()` 공유 경로로 전환하여 컨트롤 슬롯·run 분할·char_shapes
경계·lineseg 보존이 세 경로(본문/셀/글상자)에서 일관되게 동작한다.

**baseline 게이트: xfail 25건 승격** — `samples/hwpx/` 54건 중 A등급 48건
(이전 A=52는 컨트롤 비교 미포함 기준, #1379 1단계 게이트 강화 후 기준으로는 23건).
잔존 xfail 5건은 전부 본 타스크 범위 밖 별도 이슈 귀속 (#1382 1건, #1384 4건).

## 2. 단계별 수행 내역

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 게이트 컨트롤 비교 강화 (문단별 인라인 슬롯 컨트롤 타입 시퀀스, 셀·글상자·각주/미주 재귀) + 전수 측정 + xfail 16건 등록 | e75667f2 |
| 2 | 셀 문단 직렬화 공유 경로 전환 + charOverlap(hp:compose) 방출 + 테스트 7건 | 89f51121 |
| 3 | 글상자(drawText) 공유 경로 전환 + rect 하위 요소 전면 보존 + 셀·글상자 한정 colPr 인라인 방출 + 테스트 7건 | 50476888 |
| 4 | xfail 25건 일괄 승격 + 귀속 갱신(#1382/#1384) + --batch 전수 + SVG 비교 + 매뉴얼 갱신 | (본 보고서와 함께 커밋) |

### 3단계 주요 보존 항목 (원본 XML 실측 기반)

- drawText: `textDirection` VERTICAL/**VERTICALALL** 구분(`TextBox.vertical_all` 신설),
  vertAlign, textMargin은 subList 뒤 (footnote-tbox-01 실측 순서)
- rect: 자식 순서 `offset→orgSz→curSz→flip→rotationInfo→renderingInfo→lineShape→
  fillBrush→shadow→drawText→pt0~3→sz→pos→outMargin→shapeComment` (tbox-v-flow-01 실측),
  renderingInfo 행렬 f32 정밀도 보존("1.579917")
- pos `flowWithText`/`allowOverlap` 하드코딩 제거 — IR 값 방출 (원본이 하드코딩과 반대값)
- colPr: `sub_list_depth` 가드로 셀·글상자 한정 인라인 방출, **본문 경로 불변**

## 3. 검증

### 3.1 게이트·테스트

- `cargo test --lib` 1704 passed / `cargo test --test hwpx_roundtrip_baseline` 4 passed
  (승격 25건 포함 전수 그린) / clippy 경고 0 / fmt 통과
- 신규 테스트 14건 (2단계 7 + 3단계 7) — 슬롯 위치·경계 복원·재귀 채번·역매핑 문자열·
  대표 샘플 roundtrip

### 3.2 --batch 전수 (`output/poc/task1379/inventory.tsv`)

54건: **PASS 48** / IR_DIFF 1(#1382) / SERIALIZE_FAIL 4(#1384) / PARSE_FAIL 1(EXCLUDED
hwpx-01) — xfail·EXCLUDED 목록과 1:1 정합.

### 3.3 SVG 비교 (`output/poc/task1379/svg/`)

| 샘플 | 결과 |
|------|------|
| tbox-v-flow-01 (글상자 대표) | 원본·RT SVG **바이트 동일** |
| ta-pic-001-r (셀 그림 대표) | 셀 내 그림 2개(image 요소) 보존. 차이 2건은 아래 신규 발견 — 본 타스크 수정 범위 밖 |

## 4. 신규 발견 (후속 이슈 후보 2건)

ta-pic-001-r SVG 비교에서 검출 — 둘 다 #1379(컨트롤 보존)와 별개 축이며 baseline
게이트의 사각(`diff_documents` 미비교 항목)이다.

### ① 표 캡션(`hp:caption`) 미직렬화

- 원본 표의 캡션 subList(문단 "&lt;그림 1&gt; 의정활동 모니터링 시스템 예시",
  autoNum 컨트롤 포함)가 RT에서 소실 — 파서는 `table.caption` 적재, serializer
  `write_table` 미방출, `diff_documents` 미비교
- 제안: 별도 이슈 등록 (caption 직렬화 + 게이트 caption 비교 동승)

### ② secPr 페이지 여백 템플릿 하드코딩

- `hp:margin` left/right=8504 고정 방출 — 원본 4252(15mm)가 30mm로 변형, 본문
  콘텐츠 전체 +56.7px 균일 시프트. #1166은 landscape/width/height만 동적화
- 제안: 별도 이슈 등록 (pagePr margin IR 동적화)

## 5. 정정 사항

- 이슈 #1379 본문 "ta-pic-001-r hp:pic 4개 소실"은 부정확 — 원본 section0.xml 실측
  **2개**(전부 셀 내)이며 전부 보존 확인 (2단계 보고서, 테스트 주석 명기)

## 6. 한컴 시각 판정 요청 (작업지시자)

`output/poc/task1379/`의 RT 파일 2건을 한컴 편집기에서 열어 판정 부탁드립니다:

| 파일 | 판정 포인트 |
|------|------------|
| `tbox-v-flow-01.rt.hwpx` | 세로쓰기 글상자(VERTICALALL) 방향·내용·테두리/채움 |
| `ta-pic-001-r.rt.hwpx` | 셀 내 그림 2개 표시 (단, 캡션 소실·여백 30mm 변형은 신규 발견 ①②로 기지) |

## 7. 변경 파일 (전체)

| 파일 | 변경 |
|------|------|
| `src/serializer/hwpx/table.rs` | write_sub_list 공유 경로 + depth 가드 + 테스트 9건 |
| `src/serializer/hwpx/shape.rs` | write_rect 전면 보강 + write_draw_text 공유 경로 + 역매핑 헬퍼 + 테스트 5건 |
| `src/serializer/hwpx/section.rs` | render_compose/render_col_pr_ctrl + 가시성 조정 + render_text_runs 제거 |
| `src/serializer/hwpx/context.rs` | `sub_list_depth` |
| `src/serializer/hwpx/roundtrip.rs` | 컨트롤 타입 시퀀스 비교 (1단계) |
| `src/model/shape.rs` | ObjectNumberingType + numbering_type + vertical_all |
| `src/parser/hwpx/section.rs` | numberingType/instid/ratio + VERTICALALL 적재 |
| `src/document_core/...` 2건 | 신규 필드 초기화 |
| `tests/hwpx_roundtrip_baseline.rs` | 게이트 강화(1단계) + xfail 승격·귀속 갱신(4단계) |
| `mydocs/manual/hwpx_roundtrip_baseline.md` | 현황·비교 항목·Known limitations 갱신 |

## 8. 승인 요청 사항

1. 최종 보고서 승인 + 이슈 #1379 클로즈 (devel 머지 후)
2. 신규 발견 ①(표 캡션 미직렬화) ②(secPr 여백 하드코딩) 별도 이슈 등록
3. 한컴 시각 판정 (6절 — 판정 결과에 따라 후속 조치)
