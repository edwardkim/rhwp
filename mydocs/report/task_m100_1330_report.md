# Task M100-1330 최종 보고서

## 대상

- GitHub Issue: #1330 `rhwp-studio: 빈 글머리표 줄에 입력 시 marker와 caret 크기가 커짐`
- 브랜치: `issue-1330-bullet-marker-caret-size`
- 기준 브랜치: `upstream/devel`
- 관련 PR: #1331 `Task #1329: 빈 글머리표 줄 caret 위치 보정`

## 문제

rhwp-studio에서 글머리표/번호 문단 끝에서 Enter를 누르면 새 빈 list 문단이 생성된다.
이 상태에서 본문 TextRun이 아직 없으면 marker와 caret이 기본 글자 모양 기준으로 렌더링될 수 있었다.

이후 같은 줄에 텍스트를 입력하면 실제 본문 run이 생성되고 문단의 활성 글자 모양이 적용되어
marker와 caret/입력 글자 크기가 갑자기 커져 보였다.

## 근본 원인

`Paragraph::split_at()`은 새 빈 문단의 활성 `char_shape_id`를 `char_shapes[0]`에 보존하고 있었다.
따라서 모델 레벨의 스타일 전파는 이미 맞았다.

문제는 렌더러 fallback 경로였다.

- 빈 list 문단은 본문 run이 없어서 marker style 산정 시 첫 run을 참조할 수 없다.
- 이때 기존 fallback은 `resolved_to_text_style(styles, 0, 0)`을 사용했다.
- 빈 caret anchor용 empty TextRun도 같은 기본 style을 사용했다.
- 입력 후에는 본문 run이 생기면서 실제 active char shape 기준으로 다시 렌더링되어 크기가 바뀌었다.

## 수정

`src/renderer/layout/paragraph_layout.rs`에서 빈 문단 fallback의 style 기준을 문단 활성 글자 모양으로 맞췄다.

변경 사항:

- `paragraph_active_text_style()` helper 추가
  - 문단과 offset 기준의 활성 `char_shape_id`를 `TextStyle`로 변환
  - 빈 문단에서는 `char_shapes[0]` fallback 사용
- `numbering_marker_text_style()` helper 추가
  - 본문 run이 있으면 기존처럼 첫 run style 사용
  - 본문 run이 없으면 문단 활성 글자 모양 사용
- marker 폭 사전 계산 보정
- marker TextRun 생성 style 보정
- empty TextRun style 및 `char_shape_id` 보정

#1329 / PR #1331의 caret x 좌표 보정은 이 PR에 포함하지 않았다. 이 PR은 입력 전후의
marker/caret 크기 일관성만 다룬다.

## 회귀 테스트

신규 테스트:

- `tests/issue_1330_bullet_marker_caret_size.rs`

검증 흐름:

1. 빈 문서를 생성한다.
2. 큰 글자 크기(`fontSize=1800`)를 적용한 글머리표 문단을 만든다.
3. 문단 끝에서 `split_paragraph_native()`로 새 빈 글머리표 문단을 만든다.
4. 빈 문단의 marker font size, empty anchor font size, caret height를 측정한다.
5. 새 문단에 `"가"`를 입력한다.
6. 입력 후 marker font size, body font size, caret height가 입력 전과 같은지 확인한다.

## 검증

자동 검증:

| 항목 | 명령 | 결과 |
| --- | --- | --- |
| Rust format | `cargo fmt --all -- --check` | 통과 |
| #1330 회귀 테스트 | `cargo test --test issue_1330_bullet_marker_caret_size` | 통과 |
| Rust lib tests | `cargo test --lib` | 통과 |
| Rust full tests | `cargo test` | 통과 |
| Rust clippy | `cargo clippy -- -D warnings` | 통과 |
| WASM build | `wasm-pack build --target web` | 통과 |
| rhwp-studio build | `npm run build` (`rhwp-studio`) | 통과 |

수동 검증:

- `rhwp-studio` dev server: `http://127.0.0.1:7700/`
- Codex in-app Browser smoke check:
  - title `rhwp-studio`
  - blank page 아님
  - framework overlay 없음
  - console error/warn 0건
- 작업지시자 수동 확인:
  - 수정 반영 확인
  - 입력 전후 marker/caret 크기 튐 현상 해소 확인

참고:

- Docker daemon이 실행 중이 아니어서 Docker 기반 WASM 빌드는 수행하지 못했다.
- 로컬 `wasm-pack 0.15.0`과 `wasm32-unknown-unknown` 타깃으로 `pkg/`를 갱신했다.
- `pkg/`와 `rhwp-studio/dist/`는 git ignored 산출물이며 PR 포함 대상이 아니다.

## PR 방침

- base: `devel`
- head: `postmelee:issue-1330-bullet-marker-caret-size`
- #1331 변경은 포함하지 않는다.
- PR 본문에서는 #1331과의 관계를 명시하되, #1330 이슈 자동 close는 하지 않는다.

## 결론

빈 글머리표/번호 문단에서 입력 전 marker/caret과 입력 후 marker/body/caret이 같은 활성 글자 모양 기준을 사용하게 됐다.
따라서 Enter 직후 빈 list 줄에서 텍스트 입력 시 marker와 caret 크기가 갑자기 변하는 현상이 해소됐다.
