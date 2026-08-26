# Stage 244: PR #4755 tokenizer 계약 리베이스

## 목표

PR #4755의 physical-frame reflow 경로와 #3820의 재조판 공백 metric 경로를 하나의 tokenizer 계약으로 연결하고, #4755의 동작이 회귀되지 않도록 한다.

## 원인

- PR #4755는 `tokenize_paragraph_with_split_cell_space_metric`을 일반 reflow와 frozen scalar projection에서 `false` 플래그로 호출한다.
- #3820은 같은 함수의 적용 범위를 stale split-cell에 한정하지 않고 한컴 재조판 공백 metric으로 일반화하면서 이름을 `tokenize_paragraph_with_regenerated_space_metric`으로 변경했다.
- 최신 리베이스에서 PR #4755가 추가한 호출부 2곳만 제거된 예전 이름을 유지해 라이브러리 컴파일이 실패했다.

## 변경

- PR #4755 호출부 2곳을 현재 권위 함수인 `tokenize_paragraph_with_regenerated_space_metric`에 연결한다.
- 두 호출의 `false` 플래그, inline-control 전달, frame carve 및 row commit 경로는 변경하지 않는다.
- 호환 wrapper나 중복 tokenizer를 추가하지 않는다.

## 검증

- `cargo fmt --all -- --check`: 종료 코드 `0`
- 전체 라이브러리 회귀: `3673 passed; 0 failed; 13 ignored`
- 전체 integration 회귀: 종료 코드 `0`, 실패 표식 `0`
- PR #4755 LayoutFrame 단위 테스트 9건: 모두 통과
- PR #4755 frame reflow, picture band, table owner-width 테스트: 모두 통과
- `cargo clippy -- -D warnings`: 종료 코드 `0`
