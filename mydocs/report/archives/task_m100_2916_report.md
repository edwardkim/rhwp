# 완료 보고서 — Task M100-2916

- 이슈: #2916
- 제목: hp:equation 의 hp:script 가 CDATA 로 인코딩된 경우 파서가 수식 스크립트를 소실함
- 작성일: 2026-07-22
- 브랜치: `task/m100-2916-equation-script-cdata-loss`

## 1. 완료 내용

`src/parser/hwpx/section.rs`의 `parse_equation()`이 `<hp:equation>`의 `<hp:script>`
자식 요소 텍스트를 수집할 때 `Event::Text`와 `Event::GeneralRef`(개체 참조)만
처리하고 `Event::CData`(`<![CDATA[...]]>`)는 처리하지 않아, 수식 스크립트가 CDATA로
저장된 HWPX 문서를 열면 `Equation.script`가 통째로 빈 문자열이 되는 문제를 수정했다.

같은 파일의 `header.rs::read_numbering_para_head_text()`가 이미 `Event::Text`와
`Event::CData`를 동시에 처리하는 패턴을 쓰고 있어, HWPX 문서에서 텍스트 콘텐츠가
CDATA로 인코딩되는 것이 실제로 관측되는 케이스임을 뒷받침했다.

## 2. 주요 변경

- `src/parser/hwpx/section.rs`
  - `parse_equation()`의 `<hp:script>` 파싱 루프에 `Event::CData(ref cdata)` 분기
    추가 — `in_script`일 때 CDATA 본문을 `script`에 그대로 누적(UTF-8 lossy 디코드).
  - 회귀 가드 단위 테스트
    `parser::hwpx::section::tests::task_m100_2916_equation_script_cdata_not_lost`
    추가: `<hp:script><![CDATA[a < b > c]]></hp:script>`를 `parse_equation()`에
    직접 통과시켜 `eq.script == "a < b > c"`를 검증(수정 전에는 `""`로 실패).

## 3. 검증 결과 (경량 검증 — 디스크 공간 제약으로 `cargo check --lib` 및 대상
테스트만 실행, 전체 빌드/clippy/release-test는 생략)

통과:

- `cargo check --lib`
- `cargo test --lib task_m100_2916_equation_script_cdata_not_lost`
  (수정 제거 시 `left: "" / right: "a < b > c"`로 실패 → red→green 확인)
- `rustfmt --edition 2021 src/parser/hwpx/section.rs`

## 4. 스코프 메모

이번 라운드 작업 도메인은 `<hp:equation>`의 `<hp:script>` 자식 요소와 형제 속성
(`version`/`baseUnit`/`baseLine`/`textColor`)으로 한정했다. `secPr`, 공용 도형
(common-shape), `lock`/`numberingType`은 다른 진행 중인 작업과 충돌 방지를 위해
건드리지 않았다.

사전 중복 확인 결과, 열린 PR #2850(`task/m100-2840-equation-lock-roundtrip`)은
수식 `lock` 속성만 다루고, PR #2899(`task/m100-2883-object-desc-eq-script-len-guard`)는
`rhwp-studio`의 TypeScript 입력 길이 가드만 다뤄 `.rs` 파일을 건드리지 않는다 —
둘 다 이번 변경과 겹치지 않는다.
