# Task #1692 Stage 12 - SO-SUEOP HWP3 22쪽 미주 표지 위치 조사

## 배경

- 직전 커밋: `d58a590a2 task 1692: SO-SUEOP p22 본문 기호 복원`
- 22쪽 HWP3 렌더에서 `118)` 미주 표지가 설명 문단 맨 앞에 보이지만, PDF/HWPX 기준에서는 `가문의 영예(` 뒤 괄호 안에 위치한다.

## 계획

1. HWP3 문단 574의 control 위치 매핑 방식을 확인한다.
2. 미주 control 자체를 보존하면서 표시 위치만 HWPX/PDF 기준으로 옮길 수 있는지 검토한다.
3. 적용 가능하면 22쪽 문맥에 한정해 보정하고 render tree 테스트를 추가한다.

## 진행 기록

- HWP3 SO-SUEOP 22쪽 첫 설명 문단은 표 control과 미주 control이 같은 문단에 있으며,
  PDF/HWPX 기준 미주 `118)` 표지는 `가문의 영예(` 뒤 괄호 안에 표시된다.
- 문단 텍스트 복원 시 표 control은 문단 선두, 미주 control은 `가문의 영예(` 뒤 괄호
  내부 위치로 매핑되도록 `char_offsets`를 재구성했다.
- `paragraph_layout`의 선두 미주 prefix 렌더링은 기존에 문단 내 모든 Endnote를 첫 줄
  앞에 그려 p22에서 `118)`이 중복 표시됐다. `control_text_positions()` 기준 실제 위치가
  첫 줄 시작인 Endnote만 prefix로 렌더링하도록 제한했다.

## 검증

- `cargo fmt`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow -- --nocapture`
- 최신 바이너리로 `samples/SO-SUEOP.hwp` 22쪽 SVG/PNG를 재생성해 확인:
  - 첫 설명 문단 선두의 `118)` 제거됨
  - `가문의 영예(` 뒤 괄호 내부에 `118)` 표시됨

## 남은 사항

- 22쪽 전체 비교 기준으로 본문 줄바꿈/폭 차이는 아직 남아 다음 스테이지에서 계속 보정한다.
