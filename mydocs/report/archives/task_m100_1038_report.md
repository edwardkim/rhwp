# 최종 결과보고서 — Task M100 #1038

## 이슈
- edwardkim/rhwp#1038 — HWPX slash/backSlash type을 선 종류로 오파싱 → 대각선 없는 셀에 검은 대각선 렌더
- 마일스톤: v1.0.0 (M100) / 브랜치: `local/task1038` (기준 `stream/devel` @ d359c302)

## 문제
`samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx` 4페이지 헤딩
"Ⅰ 사업안내"(1×3 표, treat_as_char)의 세 셀에 검은 대각선(좌하→우상)이 그려졌다.
한컴 2022 PDF p4 에는 대각선이 없다.

## 루트 원인
HWPX borderFill 에서 셀 대각선은 **독립된 두 요소**로 표현된다:
- `<hh:slash>`/`<hh:backSlash>` `type` → 대각선 **방향/형태** enum (NONE/CENTER/…)
- `<hh:diagonal>` `type/width/color` → 실제 **선 종류·굵기·색** (HWP5 attr 비트 + DiagonalLine 과 1:1)

문제 셀(borderFill 341/342/343)은 `slash type="CENTER"` 만 있고 `<hh:diagonal>` 가 없다.
한컴은 두 요소가 모두 있어야 그리므로 미표시.

기존 파서는 slash/backSlash 의 `type`("CENTER")을 선 종류 파서(`parse_border_line_type_code`)에
넘겨 `Solid(1)`로 폴백 → 방향 비트 + `bf.diagonal.diagonal_type=1` 을 설정했다. 선 정의가 없는
셀에도 렌더 트리거(`border_rendering.rs:601`, `diagonal_type != 0`)가 켜져 기본 검정 실선이 그려졌다.

## 수정 (`src/parser/hwpx/header.rs` 단일 파일)
1. `parse_slash_shape_code(attr) -> u8` 신규 — slash 형태 enum → HWP5 attr 3비트 방향 코드
   (NONE→0, CENTER→0b010, CENTER_BELOW→0b011, CENTER_ABOVE→0b110, 기타→0b111).
2. `set_diagonal_attr_bits(bf, shift, code)` — 3비트 `code` 를 그대로 기록(기존: nonzero→0b010 축소),
   `code==0`이면 클리어.
3. `slash`/`backSlash` 핸들러 — `type` 을 방향 비트(attr)만 설정하도록 분리. `diagonal_type` 할당 제거,
   slash 의 width/color 분기 제거. 선 종류/굵기/색은 `<hh:diagonal>` 핸들러가 단독 책임.

렌더러/모델/HWP5·HWP3 경로 무수정.

## 검증
| 항목 | 결과 |
|------|------|
| `cargo test` 전체 | 통과 (47개 그룹, 실패 0) |
| 신규 단위/회귀 테스트 4건 | 통과 (slash 형태 파싱, diagonal_type 불변 가드 포함) |
| p4 헤딩 (문제 샘플) | 검정 대각선 3 → **0**, 박스 테두리만 잔존 → PDF p4 정합 |
| `tac-img-02.hwpx` 회귀 | `<hh:diagonal>` 보유 셀(slash CENTER + diagonal SOLID 포함) 대각선 4개 정상 유지 |
| rustfmt (header.rs) | 정합 (변경 없음) |

## 커밋
- `84039317` Stage 1: slash/backSlash 형태 enum 파싱 분리 (diagonal_type 미오염)
- `bdf7cd0f` Stage 2: slash 형태 파싱 + diagonal_type 불변 회귀 가드
- (Stage 3) 검증 + 최종 보고서

## 비고
- `closes #1038` 은 push/merge 전 실제 upstream 이슈 번호 재확인 후 반영.
- 오늘할일(`orders/`)은 작업지시자 관할로 미편집.
