# Task M100 #3930 Stage 7 - p30 머리말 및 p144 표 이월 직접 회귀 고정

- 이슈: [#3930](https://github.com/edwardkim/rhwp/issues/3930)
- 브랜치: `fix/issue-3930-save-layout-input`
- 시작 기준: `ae3a2ca26a191f324bc089f90a23212fef1c5fc9`
- 기록일: 2026-08-05 KST
- 상태: 진행 중

## 배경

기존 #3930 회귀는 원본 HWPX와 저장 HWP의 전체 쪽수 387, SectionDef 바탕쪽 슬롯과 raw
저장 계약을 확인한다. 그러나 실제 수용 조건인 p30 책 제목 머리말과 p144/p145의 붙임 안내
블록 위치를 페이지 조판 출력으로 직접 고정하지는 않았다.

## 시작 증적

- `cargo test --profile release-test --tests -- --nocapture`는 exit code `0`으로 완료됐다.
- 현재 소스 HWPX를 `rhwp convert ... --verify --verify-pages`로 저장하면 HWP도 387쪽이며
  IR 검증이 통과한다.
- 원본과 저장 HWP의 p30(인덱스 29) render tree SHA-256은 동일하다. 두 tree의 바탕쪽에는
  `2025 행정업무운영 편람`이 있다.
- p144/p145(인덱스 143/144)는 이미지 payload를 제외한 SVG 및 페이지별 텍스트가 동일하다.
  `기안문에 작성한 붙임 문서를 첨부` 블록은 양쪽 모두 p145에 있고 p144에는 없다.

## 구현 계획

1. 기존 `issue_3930_hwpx_hwp_save_layout` 회귀에서 원본 HWPX의 p30/p144/p145 render tree를
   저장 전에 확보한다.
2. 저장 HWP 재열기 후 같은 세 페이지 tree가 원본과 byte-identical인지 검증한다.
3. p30에는 책 제목 머리말이 있고 장 제목 머리말은 없어야 함을, p144/p145에는 붙임 안내
   블록이 각각 없어야/있어야 함을 명시적으로 검사한다.
4. focused test, 관련 저장 검증 및 formatter/lint를 실행하고 결과를 이 문서에 기록한다.

## 성공 기준

- 한 쪽 수가 같더라도 p30 바탕쪽 선택 또는 p144 표 이월이 바뀌면 회귀 테스트가 실패한다.
- 검사 대상은 원본 HWPX와 실제 HWP 저장 후 재열기 결과이며 외부 PDF 또는 GUI에 의존하지 않는다.

## 구현

- `tests/issue_3930_hwpx_hwp_save_layout.rs`가 저장 전 원본 HWPX의 p30, p144, p145
  render tree를 확보한 뒤 저장 HWP 재열기 결과와 각각 완전 일치하는지 검사하도록 확장했다.
- p30은 바탕쪽의 `2025 행정업무운영 편람` 텍스트를 반드시 포함하고 `제2장. 공문서 관리`를
  포함하지 않아야 한다.
- p144에는 `기안문에 작성한 붙임 문서를 첨부`가 없어야 하며 p145에는 반드시 있어야 한다.
  전체 쪽수 387만 같아도 블록 위치 또는 머리말 선택이 바뀌면 실패한다.

## 테스트 결과

| 검증 | 결과 |
| --- | --- |
| 기존 전체 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests -- --nocapture` | exit code `0` |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout -- --nocapture` | 1 passed |
| `rhwp convert ... --verify --verify-pages` | HWP 387쪽, IR 검증 통과 |
| 원본/저장본 p30 render tree | SHA-256 동일, 책 제목 바탕쪽 확인 |
| 원본/저장본 p144·p145 | image payload 제외 SVG 및 페이지 텍스트 동일, 붙임 안내는 양쪽 p145 |

전체 스위트는 직접 회귀를 추가하기 전에 이미 완료됐다. 추가된 조건은 focused test로 별도 통과했으며,
사용자 요청에 따라 중복 전체 재실행은 수행하지 않는다.

## Stage 종료

#3930의 본연 수용 조건인 p144 블록 이월과 p30 머리말 선택은 현재 저장 경로에서 직접 확인됐고,
향후에는 페이지 수와 raw 저장 metadata만으로 통과 처리되지 않도록 페이지 조판 tree 회귀로 고정했다.
