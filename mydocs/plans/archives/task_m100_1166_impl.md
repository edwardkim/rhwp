# 구현계획서 — Task M100-1166: HWPX 가로 편집 용지 (3단계)

## 설계 요약

- **권위 매핑**: `NARROWLY`=가로(landscape=true), `WIDELY`/기타=세로(false). hwplib ForSecPr.java 확정.
- **파싱**: `parse_page_pr` (section.rs:234) `b"landscape"` 분기에서 값 매핑.
- **PageDef**: `landscape: bool` 필드 이미 존재 (page.rs:32). 렌더 swap 도 page.rs:177 존재 — 파싱만 채우면 렌더 자동 정합.
- **직렬화**: `write_section` 이 `EMPTY_SECTION_XML` 템플릿 치환. 템플릿 pagePr 의 하드코딩값(`landscape="WIDELY" width="59528" height="84186"`)을 IR page_def 로 치환.
- **섹션별**: 파싱은 secPr→section_def.page_def 로 이미 섹션별. 직렬화도 섹션별 page_def 사용.

## Stage 1 — HWPX 파싱 landscape 매핑 + RED 테스트

**목표**: HWPX 가로 용지 → page_def.landscape=true (RED 박제).

- `parser/hwpx/section.rs` `parse_page_pr`: `b"landscape"` 분기 정정
  ```rust
  b"landscape" => {
      // [#1166] OWPML: NARROWLY=가로(landscape), WIDELY=세로(portrait).
      // hwplib ForSecPr: Portrait→WIDELY, Landscape→NARROWLY.
      page.landscape = attr_str(&attr).eq_ignore_ascii_case("NARROWLY");
  }
  ```
- `tests/issue_1166_landscape.rs` (신규):
  - landscape-001.hwpx 로드 → `getPageDef(0)` landscape=true 단언 (현재 RED)
  - para-001.hwpx(세로) → landscape=false (회귀 가드)
- 검증: landscape-001 RED→GREEN, para-001 무영향. `cargo test`.
- 보고서: `working/task_m100_1166_stage1.md`

## Stage 2 — HWPX 직렬화 IR page_def 반영

**목표**: HWPX 저장 시 IR landscape/width/height 출력 (round-trip).

- `serializer/hwpx/section.rs`: 템플릿 pagePr 하드코딩값을 IR page_def 로 치환.
  - `landscape="WIDELY"` → `landscape="{NARROWLY|WIDELY}"` (page_def.landscape 기준)
  - `width="59528" height="84186"` → IR page_def.width/height
  - (margin 도 IR 반영 검토 — 기존 동작 회귀 주의)
- 치환 방식: 템플릿 고정 pagePr 문자열을 IR 기반 동적 문자열로 replacen, 또는 pagePr 전체 재구성.
- 검증: landscape-001.hwpx → IR → HWPX 저장 → 재파싱 landscape=true round-trip. 세로 문서 회귀 없음.
- 보고서: `working/task_m100_1166_stage2.md`

## Stage 3 — 통합 검증 + 시각/e2e

**목표**: 전 경로 회귀 가드 + 시각 게이트.

- 검증: `cargo test --tests` 전수 + native-skia + fmt + clippy --lib.
- **e2e (호스트 CDP)**: landscape-001.hwpx 로드 → getPageDef landscape=true + renderPageSvg 가로 비율(width>height swap 후).
- **작업지시자 시각 판정**: 한컴 정답(297×210 가로) 대조.
- 회귀: para-001 등 세로 HWPX 무영향, HWPX round-trip 테스트 유지.
- 최종 보고서: `report/task_m100_1166_report.md`.

## 단계별 커밋 규칙

각 stage 소스 + `_stage{N}.md` 함께 커밋. fixture(landscape-001) 포함.

## 리스크 점검

- 직렬화 템플릿 치환 시 pagePr 의 다른 속성(gutterType/margin) 회귀 — HWPX round-trip 기존 테스트로 보호.
- 세로 문서(WIDELY)가 landscape=false 유지 확인 (eq_ignore_ascii_case NARROWLY 만 true).
