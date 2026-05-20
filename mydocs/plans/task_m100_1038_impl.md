# 구현계획서 — Task M100 #1038

## 이슈
- edwardkim/rhwp#1038 — HWPX slash/backSlash type을 선 종류로 오파싱 → 대각선 없는 셀에 검은 대각선 렌더
- 브랜치: `local/task1038` (기준 `stream/devel` @ d359c302)

## 설계 근거
HWP5 바이너리(`src/parser/doc_info.rs:295~335`)에서 셀 대각선은 **독립된 두 필드**다:
- `attr`(u16) 속성 비트: slash 방향 = bits 2~4, backSlash 방향 = bits 5~7 (3비트 코드)
- `diagonal`(DiagonalLine): 대각선 **선 종류**(0=없음/1=실선/…)·굵기·색

렌더러(`src/renderer/layout/border_rendering.rs:591~603`)는
`slash_bits=(attr>>2)&7`, `backslash_bits=(attr>>5)&7` **그리고** `diagonal.diagonal_type != 0`
두 조건이 모두 참일 때만 대각선을 그린다.

HWPX 파서는 이 구조를 그대로 미러링해야 한다:
- `<hh:diagonal type width color>` → `bf.diagonal` (선 종류/굵기/색) — 현행 `b"diagonal"` 핸들러 유지
- `<hh:slash type>` / `<hh:backSlash type>` → `bf.attr` 방향 비트 **only**

현행 버그: slash/backSlash 핸들러가 `type`("CENTER")을 `parse_border_line_type_code()`로 넘겨
`Solid(1)`로 폴백 → `set_diagonal_attr_bits`로 방향 비트(0b010) + **`bf.diagonal.diagonal_type=1`** 설정.
선 정의(`<hh:diagonal>`)가 없는 셀(341/342/343)에도 diagonal_type가 켜져 검정 실선이 그려진다.

### slash 형태 enum → 3비트 방향 코드 (HWP5 정합)
| HWPX type | 3비트 | 렌더러 대응 |
|-----------|-------|------------|
| `NONE` | 0 | 미표시 |
| `CENTER` | 0b010 (2) | 단순 슬래시 |
| `CENTER_BELOW` | 0b011 (3) | 가운데+아래 |
| `CENTER_ABOVE` | 0b110 (6) | 가운데+위 |
| 기타/`ALL` | 0b111 (7) | 3방향 |

## 단계별 계획 (3단계)

### Stage 1 — 파서 수정 (`src/parser/hwpx/header.rs`)
1. `parse_slash_shape_code(attr) -> u8` 신규 함수: slash 형태 enum 문자열 → 3비트 방향 코드(위 표).
2. `set_diagonal_attr_bits(bf, shift, code)` 변경: 인자로 받은 3비트 `code`를 해당 shift에 그대로 기록
   (현행은 nonzero를 무조건 0b010으로 축소). `code==0`이면 비트 클리어.
3. `b"slash"` / `b"backSlash"` `type` 분기:
   - `parse_border_line_type_code` 호출 제거 → `parse_slash_shape_code` 호출.
   - `bf.diagonal.diagonal_type = …` 할당 **제거** (방향 비트만 설정).
   - slash/backSlash의 `width`/`color` 분기 제거 (OWPML에서 slash는 선 스타일을 갖지 않음; 선은 `<hh:diagonal>` 전담).
- 커밋: `Task #1038: HWPX slash/backSlash 형태 enum 파싱 분리 (diagonal_type 미오염)`
- 보고서: `mydocs/working/task_m100_1038_stage1.md`

### Stage 2 — 단위 테스트 (`src/parser/hwpx/header.rs` #[cfg(test)])
1. `parse_slash_shape_code`: NONE→0, CENTER→2, CENTER_BELOW→3, CENTER_ABOVE→6, 미지값→7 검증.
2. borderFill 파싱 통합 테스트:
   - `slash type="CENTER"` + `<hh:diagonal>` 없음 → `diagonal.diagonal_type==0`, `attr` slash 비트==0b010.
   - `slash type="NONE"` + `<hh:diagonal type="SOLID">` → `diagonal_type==1`, slash 비트==0.
- 커밋: `Task #1038: slash 형태 파싱 + diagonal_type 불변 회귀 가드`
- 보고서: `mydocs/working/task_m100_1038_stage2.md`

### Stage 3 — 검증 및 최종 정리
1. `cargo build` + `cargo test` 전체 통과.
2. `export-svg -p 3` 재생성 → 헤딩 3개 셀 검정 대각선(`stroke="#000000"`) 제거 확인
   (`samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx`, PDF p4 정합).
3. 실제 `<hh:diagonal>` 보유 표 샘플 회귀: 대각선 정상 렌더 유지 확인.
4. 수정·신규 파일 한정 `rustfmt` 정리 (전체 fmt 금지).
- 최종 보고서: `mydocs/report/task_m100_1038_report.md`, 오늘할일은 작업지시자 관할이므로 미편집.
- 커밋: `Task #1038: 검증 + 최종 보고서 (closes #1038)` — closes 번호는 push 전 재확인.

## 영향 범위 / 리스크
- 수정: `src/parser/hwpx/header.rs` 단일 파일.
- 렌더러/모델/HWP5·HWP3 경로 무수정.
- 리스크: `set_diagonal_attr_bits`가 이제 실제 3비트 코드를 기록 → 기존엔 모든 방향이 0b010으로 축소됐던 동작이 정밀화됨. CENTER는 0b010 동일이라 기존 정상 렌더 표는 영향 없음. CENTER_BELOW/ABOVE 보유 표는 더 정확해짐(회귀 아님).
