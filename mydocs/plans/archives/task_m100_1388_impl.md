# Task M100 #1388 구현계획서 — HWPX serializer secPr 페이지 여백 원본 보존

- 수행계획서: `mydocs/plans/task_m100_1388.md` (승인 완료)
- 브랜치: `local/task1388`
- 작성일: 2026-06-12
- 단계: 4단계

## 0. 사전 조사 확정 사항

### 0.1 serializer 경로 (serializer/hwpx/section.rs — 코드 확인 완료)

- `write_section()`이 섹션마다 `replace_page_pr(&out, &section.section_def.page_def)` 호출
  (`section.rs:81`) — **다중 섹션도 섹션별 page_def가 전달**되므로 치환 지점 확장만으로 충분.
- `replace_page_pr`(`section.rs:1249`)은 pagePr 여는 태그 고정 문자열
  (`landscape="WIDELY" width="59528" height="84186" gutterType="LEFT_ONLY"`)만 치환.
- 템플릿 margin 줄(고정):
  `<hp:margin header="4252" footer="4252" gutter="0" left="8504" right="8504" top="5668" bottom="4252"/>`
- 미스매치 시 원본 유지(silent no-op) 정책은 #1166과 동일하게 따른다.

### 0.2 parser 경로 (parser/hwpx/section.rs — 코드 확인 완료)

- `parse_page_margin`(`:338`) — 7필드 전부 `PageDef`로 적재. 수정 불요.
- `parse_page_pr`(`:295`) gutterType 매핑 확립:
  `LEFT_ONLY→SingleSided(0)` / `LEFT_RIGHT→DuplexSided(1)` / `TOP_BOTTOM→TopFlip(2)`
  (+ `page.attr` bit 1..2 동기). serializer는 이 역매핑을 그대로 쓴다 — 추정 불요.

### 0.3 게이트 경로 (serializer/hwpx/roundtrip.rs — 코드 확인 완료)

- `diff_documents`(`:236`) 섹션 루프(`:249`)에 `PageDef` 비교 없음.
- #1380 `ParagraphLinesegs` 패턴(variant 추가 + Display + 말미 비교 루프) 준용.

## 1단계 — 전수 측정 + height +2 유닛 조사

코드 수정 없음 (조사 전용).

### 1.1 원본 margin 분포 측정

- `samples/hwpx` 전수의 section0.xml에서 `<hp:margin>`·`gutterType` 추출(unzip+grep),
  템플릿 고정값과 다른 샘플 수·필드별 분포 정량화.
- RT 산출물(`hwpx-roundtrip --batch`)과 대조 — 변형 샘플 수 = 원본≠템플릿 샘플 수 일치 확인.

### 1.2 height +2 유닛 증상 조사 (#1380 4단계 비고)

- business_overview 계열에서 pagePr height 84186→84188(+2) 관찰됨. `replace_page_pr`는
  IR 값을 그대로 방출하므로 후보는 ① 원본 XML이 84188 (변형 아님) ② 파서 적재 변형.
  원본 XML 직접 대조로 귀속 확정. 여백 밖 원인이면 별도 이슈 분리 판정.

### 1.3 게이트 동승 영향 사전 판정

- 측정 결과로 3단계 동승 시 xfail 필요 샘플 유무를 구현 전에 판정.

### 1.4 보고 + 승인 요청

- `mydocs/working/task_m100_1388_stage1.md`

## 2단계 — serializer 수정 (`replace_page_pr` 확장)

### 2.1 margin 치환

- `TEMPLATE_PAGE_MARGIN` 고정 문자열 상수 추가 → IR `PageDef` 7필드
  (header/footer/gutter/left/right/top/bottom)로 `replacen(..., 1)` 치환.
- 치환 순서는 기존 pagePr 태그 치환과 동일 지점(`replace_page_pr` 내부)에서 수행.

### 2.2 gutterType 치환

- pagePr 여는 태그 포맷에 gutterType 동적화:
  `SingleSided→LEFT_ONLY` / `DuplexSided→LEFT_RIGHT` / `TopFlip→TOP_BOTTOM` (0.2 역매핑).

### 2.3 단위 테스트 (section.rs)

- margin 7필드 비기본값 PageDef → 방출 XML에 원본 값 검증
- gutterType 3종 매핑 검증
- 템플릿 미스매치 시 원본 유지(no-op) 검증
- 기존 #1166 landscape/width/height 테스트 회귀 없음

### 2.4 보고 + 승인 요청

- spot 배치 재실행으로 margin 변형 해소 수치 포함, `_stage2.md`

## 3단계 — 게이트 동승 (`diff_documents`)

### 3.1 variant 추가

- `IrDifference::SectionPageDef { section: usize, detail: String }` + Display arm.

### 3.2 비교 추가

- 섹션 대응 루프에서 `PageDef` 비교: width/height/landscape/binding + margin 7필드.
- `attr`(비트 원본)·`pagination_bottom_tolerance`(렌더 내부 필드)는 비교 제외 — 사유 주석 명기.

### 3.3 단위 테스트 (roundtrip.rs)

- margin 불일치 주입 → 게이트 검출 검증
- 실샘플(ta-pic-001-r 등) parse→serialize→재parse → diff 0 검증 (2단계 수정 후 기대치)

### 3.4 보고 + 승인 요청

- baseline 전수 결과(xfail 변동 포함), `_stage3.md`

## 4단계 — 전수 검증 + 문서 + 한컴 판정 요청

1. `hwpx-roundtrip --batch samples/hwpx` 전수 → `output/poc/task1388/` (inventory + rt)
2. `cargo test --test hwpx_roundtrip_baseline` — 신규 실패 0
3. 페이지 수 대조: ta-pic-001-r + business_overview/expense_report/form-002/보도자료
   (#1380 4단계 sed 패치 실험의 코드 구현판 — 패치 없이 일치해야 함)
4. SVG 비교: ta-pic-001-r 원본 vs RT — +56.7px 시프트 해소 확인
5. 매뉴얼 `mydocs/manual/hwpx_roundtrip_baseline.md` 갱신 (게이트 비교 항목 + known limitations에서 #1388 해소 처리)
6. CI급: `cargo test --tests` + `cargo fmt --check` + clippy
7. 최종 보고서 `mydocs/report/task_m100_1388_report.md` + 한컴 판정 요청 (rt 샘플 지정)

## 위험 관리 (수행계획서 5절 보강)

| 위험 | 단계 | 대응 |
|------|------|------|
| margin 치환 미스매치 silent no-op | 2 | 단위 테스트가 치환 성공을 직접 검증. 템플릿 변경 시 테스트가 즉시 실패 |
| 여백 동적화로 페이지 수 변동 | 2·4 | 변동은 원본 정합 방향 — 4단계 페이지 수 대조표로 전량 귀속 |
| 게이트 동승 시 잠재 차이 일괄 노출 | 1·3 | 1단계 측정으로 사전 정량화, xfail 판정을 구현 전 승인 |
| height +2 유닛이 별개 결함으로 판명 | 1 | 본 타스크 범위 밖이면 이슈 분리 (#1391~#1393 선례) |
