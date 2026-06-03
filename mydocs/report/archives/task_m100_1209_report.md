# Task M100 #1209 최종 보고서

## 개요

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- PR: [#1232](https://github.com/edwardkim/rhwp/pull/1232)
- 브랜치: `local/task_m100_1209`
- PR 원격 브랜치: `jangster77:task_m100_1209`
- 기준: `upstream/devel` `f83c43b5`
- 최종 코드 커밋: `c49597f9` (`task 1209: Stage8 단 바깥 TopAndBottom 흐름 보정`)
- PR 준비 커밋: `fcf1c937` (`task 1209: PR 준비 보고서 작성`)

Task #1139/#1189 후속으로 미주 간격, 문단/그림 배치, 어울림 그림 줄 흐름, 모달 드래그 동작을 보정했다. 마지막 PR 준비 중 `issue_1082_endnote_multicolumn_drift` 회귀를 발견해 Stage8에서 현재 단 바깥의 문단 기준 `TopAndBottom` 그림이 줄 위치 보정에 참여하지 않도록 공통 가로 교차 조건을 추가했다.

## 주요 변경

- 미주 모양의 `미주 사이`, `구분선 아래` 값을 공통 로직으로 반영해 파일별 예외 처리 의존을 줄였다.
- compact 미주와 internal rewind 수식/문단에서 한컴 기준 간격을 보존하도록 `HeightCursor`, `typeset`, `layout` 보정을 정리했다.
- 어울림 그림이 본문 흐름을 만드는 경우 줄 위치/후속 문단 배치가 한컴 기준에 가깝도록 보정했다.
- `test-image.hwp`/`test-image2.hwp` 기준으로 HWP5 개체 배치 진단기와 스펙 보완 주석을 정정했다.
- 글자 모양, 문단 모양, 수식/그림/표/검증 등 모달 대화상자의 드래그 처리를 공통 유틸로 정리했다.
- PR 준비 중 발견한 `3-11월_실전_통합_2022.hwp` page 1 회귀를 수정했다. 현재 단과 가로 범위가 교차하지 않는 문단 기준 `TopAndBottom` 그림/도형은 `LINE_SEG.vertical_pos` 보정 대상에서 제외한다.

## 수식 관련 닫힌 PR 회귀 점검

2026-06-02 기준 최근 닫힌 수식 관련 PR을 확인했다.

- #1208: HWPX 수식 스크립트 토큰 처리
- #1223: 수식 포함 줄 본문 한글 압축·겹침 해소
- #1225: HWP5 수식-only 셀 z-표 행 압축 수정

이에 대해 전체 테스트 외에 수식 단위/대표 통합 가드를 별도 실행했다.

## 검증

- `cargo fmt --all --check` 통과.
- `cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과.
- `cargo test --tests` 통과.
- `cargo test --lib renderer::equation -- --nocapture` 통과: 132 passed.
- `cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture` 통과.
- `cargo test --test issue_301 -- --nocapture` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과. `wasm-bindgen` prebuilt 미지원으로 cargo install fallback 경고가 있었으나 산출물 생성 완료.
- `npm run build` (`rhwp-studio`) 통과. Vite chunk size 경고는 기존 production build 경고 범주.

## PR 생성

제목:

```text
task 1209: task 1139 후속 미주·그림 흐름 보정
```

본문:

```markdown
## 요약

- 미주 모양의 미주 사이/구분선 아래 값을 공통 흐름 계산에 반영했습니다.
- compact 미주, internal rewind 수식, 어울림 그림의 문단 흐름 보정을 정리했습니다.
- HWP5 개체 배치 진단기와 스펙 보완 주석을 `test-image` 샘플 기준으로 정정했습니다.
- 글자 모양/문단 모양/그림/수식 등 주요 모달 대화상자의 드래그 처리를 공통화했습니다.
- PR 준비 중 발견된 `issue_1082_endnote_multicolumn_drift` 회귀를 현재 단 가로 교차 조건으로 보정했습니다.

## 검증

- `cargo fmt --all --check`
- `cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo test --tests`
- `cargo test --lib renderer::equation -- --nocapture`
- `cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture`
- `cargo test --test issue_301 -- --nocapture`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build` (`rhwp-studio`)

Closes #1209
```

## 남은 절차

- PR URL이 `https://github.com/edwardkim/rhwp/pull/1232` 형태인지 확인했다.
- CI 결과를 확인한다.
