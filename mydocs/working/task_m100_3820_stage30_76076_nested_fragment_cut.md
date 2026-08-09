---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 30 — 76076 p33–p34 중첩 표 fragment 실제 폭 정합

## 기준과 재현

- 입력: `samples/76076_regulatory_analysis.hwp`
- 독립 기준: `samples/issue1891/76076_regulatory_analysis-2024.pdf` (Hancom 2024)
- 범위: PDF physical p33–p34의 `근거설명` 안 1×1 비글자 중첩 표

기준 PDF는 p33의 `현황 추이(p.270)`를 표 안에 완전히 남기고, p34는 다음 문단
`자율안전확인신고한 분쇄기 등`으로 시작한다. 이전 rhwp 출력은 같은 source 줄을 p33
하단과 p34 상단 clip에 각각 일부 paint했다. 페이지 수 또는 render tree의 source node 존재만으로는
정상으로 판정하지 않고, glyph bbox와 조상 `TableCell` clip의 교차로 판정했다.

## 원인

외부 7×2 표의 unit cut은 자식 1×1 표의 `nested_table_mixed_fragment_heights`를 이용한다.
이 helper는 `aim=false` 기본 규칙으로 `inMargin=(0,0,141,141)`만 사용하여 510HU의 저장된 좌우
cellMargin을 버린 폭으로 재조판했다. 실제 비글자 중첩 표 배치는 그 저장 여백을 사용하므로,
unit 산출은 실제보다 넓은 줄폭에서 한 줄을 덜 계산했다.

그 결과 수정 전에는 p33 nested bbox가 `y=351.1, h=636.8`인데 마지막 `현황 추이` glyph bbox는
`y=981.2..997.2`여서 cell clip 하단 `987.9`를 9.3px 넘었다. p34에도 같은 source 줄의 bbox
`y=70.4..86.4`가 cell clip top `77.1`을 가로질렀다. 이는 문단 측정과 실제 조판이 서로 다른 폭을
쓴 것이며, 글꼴 raster 차이가 아니다.

## 수정

`src/renderer/layout/table_layout.rs`의 nested fragment unit 조판도 실제 배치와 같은
`resolve_cell_padding_for_context(..., !table.common.treat_as_char)`를 사용하도록 통일했다.
최상위 표의 기본 `aim=false` 규칙은 바꾸지 않고, 부모 셀 안 `Control::Table`인 비글자 중첩 표의
fragment 측정에만 범위를 한정했다.

수정 후 p33 nested bbox는 `h=649.3`이고 마지막 줄의 glyph 하단은 `997.2`로 clip 하단
`1000.4` 안에 완전히 들어간다. p34에서 이전 줄의 잔여 bbox는 `y=57.9..73.9`로 clip top `77.1`
위에 완전히 남고, 다음 문단은 `y=83.8`부터 paint된다. 기준 PDF와 같은 source-owner 경계다.

## 증적과 검증

- [PDF p33](../pr/assets/task_m100_3820_stage30_76076_nested_fragment_cut/reference_p033.png)와
  [rhwp p33](../pr/assets/task_m100_3820_stage30_76076_nested_fragment_cut/rhwp_p033_after.png)
- [PDF p34](../pr/assets/task_m100_3820_stage30_76076_nested_fragment_cut/reference_p034.png)와
  [rhwp p34](../pr/assets/task_m100_3820_stage30_76076_nested_fragment_cut/rhwp_p034_after.png)
- `cargo fmt --all -- --check` 통과
- `CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2308_render_normalized_derived_state -- --nocapture` — 3 passed
- 같은 target에서 `issue_2007_nested_cell_pagination` 9 passed 및
  `issue_3595_nested_split_row_identity` 2 passed

새 회귀는 p33의 마지막 줄이 clip 안에 완전히 paint되고, p34에 같은 줄의 반쪽 잔여가 없으며,
다음 문단이 온전히 paint되는지를 고정한다. 전체 integration·clippy는 이 변경을 커밋하고 최신
`upstream/devel`에 rebase한 뒤 PR 준비 단계에서 다시 실행한다. WASM build는 사용자 수동 검증 범위라
이 Stage에서 실행하지 않았다.
