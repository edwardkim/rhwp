---
kind: working
status: active
issue: 6378
---

# HWPX 표 outMargin 위치를 HWP 경로와 맞춘다 (#6378)

작업 브랜치: `fix/6378-hwpx-outmargin-position`
대상:
- `src/renderer/layout/table_layout.rs` (`compute_table_x_position` 단 기준 x)
- `src/renderer/layout.rs` (빈 host 자리차지 상단)
- `tests/cases/issue_6378_hwpx_outmargin_position.rs`

만지지 않은 것: 새 CLI, gym, DocumentCore 편집, `native_hwp5_layout` 게이트를 원본 HWPX 전체에 푸는 일.

## 한 줄

같은 문서 `tac-img-02` 의 HWP 경로는 표 바깥 여백 1mm(283HU=3.8px)를 위치에 싣고,
HWPX 직파스는 단 원점·빈 host 상단에 그 값을 빼먹는다.

## 판별

`LayoutCompatibilityProfile::hwp5_stored_pagination_layout()` 이 원본 HWPX 에서
false 라, HWP5 전용 empty-host helper 가 여백을 안 준다. HWP5 경로에 여백을
여기서 또 더하면 이중 가산이므로 **원본 HWPX 만** 단 기준 x 와 빈 host 상단에
`outer_margin_*` 를 더한다.

## 실측 (이 패치 후)

1쪽 첫 Table bbox 가 HWP·HWPX 모두 `x=79.4 y=119.6 w=631.2 h=898.2`.
쪽수는 HWP 66 / HWPX 67 — 이슈가 적은 used 누산은 이 좌표 축과 별개로 남긴다.

## 기록

이슈 번호 `#6378`, 픽스처 `samples/tac-img-02.hwp` / `.hwpx`.
사용자 이름은 주석에 반복하지 않는다.
