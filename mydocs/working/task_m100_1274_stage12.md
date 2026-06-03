# Task 1274 Stage 12

## 대상

- Stage11 이후 남은 overflow:
  - `2024-09-between20`: 8줄
  - 18쪽 오른쪽 단 하단 `pi=937..939`
- 비교 대상:
  - `output/task1274/2024-09-between20/compare/compare_018.png`

## 관찰

- Stage11 이후 전체 sweep에서 여섯 대상 모두 PDF와 페이지 수가 일치한다.
- `2024-09-between20`만 18쪽 오른쪽 단 하단에 `문24）`(`pi=937`)와 뒤 문단이 과하게 들어가며 overflow가 남는다.
- PDF 18쪽 오른쪽 단은 `문23）` 시작과 첫 수식까지만 하단에 남고, `문24）`는 다음 쪽으로 넘어가는 흐름이다.
- `pi=937`에서 조판 누적 높이는 약 754px라서 충분히 들어가는 것으로 보이지만, 같은 단 첫 vpos 기준 렌더 예상 y는 하단에 매우 가깝고 실제 layout overflow는 1176px에서 발생한다.
- 단순히 vpos 예상 y만으로 넘기면 `pi=841` 같은 앞쪽 제목까지 과하게 넘어가므로, 현재 단이 75% 이상 찬 마지막 단에서만 이 보정을 적용해야 한다.

## 목적

- `2024-09-between20` 18쪽 오른쪽 단 하단의 `pi=937..939` overflow를 제거한다.
- Stage11의 p13/p14/p23 개선과 24쪽 페이지 수를 유지한다.
- 문항 번호/페이지 번호 조건 없이, 미주 제목이 단 하단에 들어갈 수 없는 구조를 공통 조건으로 처리한다.

## 검증 계획

- 진행 중 자동 테스트는 `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`만 사용한다.
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
- 필요 시 전체 sweep으로 여섯 대상 페이지 수와 overflow를 다시 확인한다.
- 전체 CI급 테스트는 전체 목표 마지막에만 수행한다.

## 수정

- `PageItem`에서 paragraph index를 추출하는 공통 helper를 추가했다.
- Stage11의 `split=1` 내부 rewind 해소 이후, 큰 미주 사이 문서의 마지막 단에서 새 미주 제목을 놓기 전에 현재 단 첫 항목의 vpos와 새 제목 vpos를 비교한다.
- vpos 예상 제목 위치가 하단 48px 여유선 안에 들어오고, 현재 단이 75% 이상 찼으면 다음 쪽으로 넘긴다.
- 이 조건으로 18쪽 오른쪽 단 하단의 `pi=937..939`가 보이지 않는 상태로 현재 쪽에 남지 않고, PDF처럼 19쪽 왼쪽 단에서 시작한다.
- `pi=841`처럼 current height가 아직 75% 미만인 앞쪽 제목은 기존 흐름을 유지한다.

## 검증 결과

- `cargo fmt`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 48개 통과
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - SVG/PDF/비교 PNG 24쪽 일치
  - `pi=937..939` overflow 제거
- `python3 scripts/task1274_visual_sweep.py`

전체 sweep 결과:

- `2022-09`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2023-09`: SVG/PDF/비교 PNG 20쪽, overflow 없음
- `2024-09-below20`: SVG/PDF/비교 PNG 23쪽, overflow 없음
- `2024-09-between20`: SVG/PDF/비교 PNG 24쪽, overflow 8줄
- `2022-10`: SVG/PDF/비교 PNG 18쪽, overflow 없음
- `2022-11-practice`: SVG/PDF/비교 PNG 21쪽, overflow 없음

## 시각 확인

- `output/task1274/2024-09-between20/compare/compare_018.png`
- `output/task1274/2024-09-between20/compare/compare_019.png`
- `output/task1274/2024-09-between20/compare/compare_023.png`
- `output/task1274/2024-09-between20/compare/compare_024.png`

18쪽/19쪽은 PDF처럼 `문24）`가 19쪽 왼쪽 단에서 시작한다. 23쪽/24쪽은 다음 후보로 남았다.

## 다음 후보

- `2024-09-between20` 21쪽 오른쪽 단 `pi=1080`의 2.8px overflow 로그
- `2024-09-between20` 23쪽 오른쪽 단 `pi=1175` partial overflow
