# M100 #974 최종 결과 보고서

## 1. 작업 개요

- 이슈: #974
- 브랜치: `local/issue-974-textbox-picture-render`
- 대상 샘플:
  - `samples/hwpx/hy-001.hwpx`
  - `samples/hwpx/hancom-hwp/hy-001.hwp`
  - `pdf-large/hwpx/hy-001.pdf`
- 목표: 글상자 안에 배치된 TAC 그림을 HWP/HWPX 양쪽에서 렌더링하고, 한컴 기준의 그림 사이 공백 배치를 맞춘다.

## 2. 처리 내용

첫 번째 수정에서는 글상자 내부에 있는 TAC 그림 컨트롤이 inline position을 얻지 못해 렌더링되지 않는 문제를 수정했다.

두 번째 수정에서는 글상자 내부 TAC 그림 사이의 space 문자열이 inline advance에 반영되지 않아 그림들이 붙어 보이는 문제를 수정했다.

세 번째 수정에서는 직접 run `charPr`의 음수 자간으로 space 폭을 계산하던 경로를 보정했다. 해당 케이스는 `TAC 그림 + 순수 space 문자열 + TAC 그림` 형태이며, 문단 정렬은 왼쪽이고 문단 스타일은 `바탕글`이다. 이 경우 space 폭은 직접 run 글자 모양만으로 계산하지 않고, `바탕글` 스타일이 반영된 한컴 조판 결과인 `lineSeg.horzsize` 기준으로 역산해 적용한다.

## 3. 주요 변경 파일

- `src/renderer/layout.rs`
  - 빈 문단 뒤의 글상자 TAC Shape가 inline position을 등록할 수 있도록 처리했다.
- `src/renderer/layout/shape_layout.rs`
  - 글상자 내부 TAC 컨트롤 사이 텍스트 advance를 계산했다.
  - 순수 space + TAC 컨트롤 라인에서는 `lineSeg.horzsize` 기반 space advance 보정을 적용했다.
- `src/wasm_api/tests.rs`
  - `hy-001` HWP/HWPX 렌더링 회귀 테스트를 추가/강화했다.
- 샘플 추가:
  - `samples/hwpx/hy-001.hwpx`
  - `samples/hwpx/hancom-hwp/hy-001.hwp`
  - `pdf-large/hwpx/hy-001.pdf`

## 4. 커밋

- `c3e32151 Fix textbox picture rendering`
- `26c6d78f Fix textbox TAC picture spacing`
- `15b137d9 Fix textbox TAC space advance`

## 5. 검증

수행한 검증:

```text
cargo test --quiet test_hy001_textbox_inline_pictures_render_for_hwp_and_hwpx
git diff --check
docker compose --env-file .env.docker run --rm wasm
```

결과:

- `hy-001` HWP/HWPX 글상자 내부 이미지 렌더링 테스트 통과
- 글상자 내부 두 TAC 그림 사이 간격 회귀 테스트 통과
- diff whitespace 검사 통과
- wasm 빌드 성공
- 작업지시자 시각 판정 통과

참고: `cargo test` 실행 중 기존 경고가 출력되었지만, 이번 변경과 무관한 기존 경고이며 테스트는 통과했다.

## 6. 판정

#974의 목표였던 글상자 내부 그림 렌더링과 TAC 그림 사이 공백 배치 문제는 처리 완료로 판정한다.

현재 브랜치 작업 트리는 깨끗하다.

```text
git status --short --branch
## local/issue-974-textbox-picture-render
```

## 7. 승인 요청

위 결과로 #974 완료 처리를 승인 요청한다.
