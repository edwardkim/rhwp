# Task 1274 Stage 6

## 대상

- 전체 sweep 결과:
  - `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 2줄
  - `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
  - `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 2줄
  - `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 41줄
  - `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 12줄
  - `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

## 현상

Stage5 이후 6개 문서는 모두 PDF와 페이지 수가 1:1로 맞는다.
그러나 manifest에는 일부 문서의 compact 미주 하단 overflow 로그가 남아 있다.
가장 작은 공통 재현은 `2022-09`와 `2024-09-below20`에 동시에 나타나는 `pi=366` 12.3px overflow다.

## 목적

- `pi=366` overflow가 실제 시각 overflow인지, compact 미주 하단 content bleed 허용 범위인지 확인한다.
- 공통 레이아웃 로직으로 처리하고, 문서/페이지/문항 하드코딩은 하지 않는다.
- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## upstream 동기화

- 작업 전 Stage6 미커밋 변경을 stash로 분리했다.
- `git fetch upstream` 후 원격 `edwardkim/rhwp`의 `devel`은 `09f6b8d1`이었다.
- `git rebase upstream/devel`을 실행했고, task 1274 커밋들은 충돌 없이 `upstream/devel` 위로 재작성되었다.
- Stage6 변경을 stash에서 복원했으며 충돌은 없었다.

## 원인

- `pi=366`은 텍스트와 컨트롤이 없는 빈 spacer 문단이다.
- 문단 자체는 다음 내용을 밀어내기 위한 시각 여백이지만, 기존 overflow 집계는 빈 줄의 `line_height` 하단을 실제 콘텐츠 하단으로 기록했다.
- 그 결과 `2022-09`와 `2024-09-below20` 6쪽에서 실제 출력 내용은 페이지 안에 있는데도 `LAYOUT_OVERFLOW_DRAW`와 `LAYOUT_OVERFLOW`가 각각 1줄씩 남았다.

## 수정

- `src/renderer/layout/paragraph_layout.rs`에 빈 spacer 줄 판정 공통 helper를 추가했다.
- 조건은 다음 세 가지를 모두 만족할 때만 적용한다.
  - line run이 모두 공백이다.
  - 해당 줄에 TAC offset이 없다.
  - 원본 문단에 컨트롤이 없다.
- 빈 spacer 줄은 draw overflow 로그 대상에서 제외한다.
- 같은 줄의 `last_item_content_bottom`은 `y + line_height`가 아니라 현재 `y`로 기록해, 후행 빈 줄 높이가 실제 콘텐츠 overflow로 전파되지 않게 했다.
- 컨트롤을 가진 빈 문단이나 TAC 개체 줄은 기존 경로를 유지한다.

## 검증

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
  - 기존 `2022-10` `pi=588` 관련 overflow 로그는 남아 있음
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 39줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 12줄
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

Stage6에서 제거된 로그:

- `2022-09` `pi=366` draw/item overflow 2줄
- `2024-09-below20` `pi=366` draw/item overflow 2줄
- `2024-09-between20`의 동일 `pi=366` draw/item overflow 2줄

## 시각 확인

- `output/task1274/2022-09/contact_sheet.png`
- `output/task1274/2024-09-below20/contact_sheet.png`
- `output/task1274/2024-09-between20/contact_sheet.png`
- 6쪽 확대 비교:
  - `output/task1274/2022-09/compare/compare_006.png`
  - `output/task1274/2024-09-below20/compare/compare_006.png`
  - `output/task1274/2024-09-between20/compare/compare_006.png`
- 빈 spacer 줄 제거로 실제 텍스트나 도형이 사라진 흔적은 없고, 페이지 수는 PDF와 계속 1:1로 유지된다.

## 다음 후보

- `2024-09-between20`에는 `pi=714` 이후 여러 실제 문단 overflow가 남아 있다.
- `2022-10`에는 `pi=588`, `pi=627` 등 12줄 overflow가 남아 있다.
- 다음 stage에서는 남은 overflow 중 가장 작은 공통 원인을 다시 분리해 처리한다.
