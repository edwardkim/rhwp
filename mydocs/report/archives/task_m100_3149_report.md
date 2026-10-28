# 완료 보고서 — Task M100-3149

- 이슈: #3149
- 제목: HWPX hp:equation version/baseLine/font 속성 생략 시 OWPML 스키마 기본값 대신 zero-계열 값으로 복원 — 라운드트립 시 값 변형
- 작성일: 2026-07-23
- 브랜치: `fix/task_m100_3149_hwpx_equation_owpml_defaults`

## 1. 완료 내용

HWPX 파서 `parse_equation`(`src/parser/hwpx/section.rs`)의 수식 전용 속성
로컬 초기값을 OWPML 스키마(ParaList XML schema, `EquationType`) 속성 기본값으로
교체했다.

| 속성 | 스키마 기본값 | 수정 전 복원값 | 수정 후 |
|---|---|---|---|
| `version` | `"Equation Version 60"` | `""` | 스키마 기본값 |
| `baseLine` | `85` | `0` | 스키마 기본값 |
| `font` | `"HYhwpEQ"` | `""` | 스키마 기본값 |

직렬화기(`src/serializer/hwpx/section.rs`의 equation 방출부)는 세 속성을 무조건
방출하므로, 수정 전에는 속성을 생략한 스펙 준수 원본이 라운드트립을 거치면
`version="" baseLine="0" font=""` 으로 값이 변형됐다. 속성이 명시된 파일의
동작은 불변이다. `baseLine`의 parse 실패 폴백도 0 → 85 로 기본값과 일치시켰다.

같은 요소의 `textColor`(#000000↔0), `baseUnit`(1000↔1000), `lineMode`(CHAR↔attr
bit 0 = 0, #2727)는 이미 스펙 기본값과 일치해 변경하지 않았다.

## 2. 주요 변경

- `src/parser/hwpx/section.rs`
  - `parse_equation`: `version_info`/`baseline`/`font_name` 초기값을 스키마
    기본값으로 교체 + 근거 주석
  - 테스트 `equation_missing_attrs_fall_back_to_owpml_defaults` 추가
    (속성 생략 수식 파싱 → 세 필드가 스펙 기본값으로 복원되는지 단언)

## 3. 검증 결과 (red → green)

- red (수정 전): `cargo test --lib equation_missing_attrs_fall_back_to_owpml_defaults`
  → FAILED — `assertion 'left == right' failed: baseLine 생략 시 스펙 기본값 85`
  (baseline 0, font_name "", version_info "")
- green (수정 후): 같은 테스트 ok. 1 passed; 0 failed
- 회귀: `cargo test --lib hwpx` → ok. 496 passed; 0 failed
- `cargo fmt --check` — Windows CRLF line-ending diff 제외 위반 없음
- `cargo clippy --lib --tests` — 신규 경고 없음

## 4. 환경 특이사항

- 이 PC 의 Windows SDK `dbghelp.lib`(10.0.26100.0)가 손상(전부 0x00)되어
  link.exe 가 LNK1123/CVT1107 으로 실패 — 시스템 파일은 건드리지 않고
  `dbghelp.dll` export 로 재생성한 import lib 를 `RUSTFLAGS=-L` 로 우선
  탐색시켜 우회했다(검증 전용, 산출물에는 영향 없음).
