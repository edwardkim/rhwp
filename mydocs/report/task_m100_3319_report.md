# Task M100 #3319 결과 보고 — SO-SUEOP HWPX OLE 선택

- Issue: [#3319](https://github.com/edwardkim/rhwp/issues/3319)
- Branch: `task/3319-hwpx-ole-selection`
- 상태: [PR #3321](https://github.com/edwardkim/rhwp/pull/3321) CI 성공, merge 승인 대기

## 결과

HWPX HMapsi OLE preview가 화면에는 렌더되지만 선택되지 않던 문제를 해결했다. HMapsi 경로도
다른 OLE preview 경로와 같이 `RawSvgNode::ole()`에 원본 control ref를 보존한다. 따라서
`getPageControlLayout()`이 이 preview를 `ole` control로 방출하고 Studio의 클릭 선택·속성 경로가
정확한 모델 control을 찾는다.

## 범위

- 수정: `src/renderer/layout/shape_layout.rs`
- Rust 회귀: `tests/issue_2069_ole_object_selection.rs`
- Studio E2E: `rhwp-studio/e2e/issue-2069-ole-object-selection.test.mjs`
- 증적: [`so_sueop_hwpx_ole_selected.png`](assets/task_m100_3319/so_sueop_hwpx_ole_selected.png)

HWP3/HWPX의 IR 타입 통일과 OLE 내부 편집 범위 확장은 이번 변경에 포함하지 않았다.

## 검증

`SO-SUEOP.hwpx` 1쪽의 실제 클릭은 `sec=0, para=0, control=2, type=ole` 선택으로 이어졌으며,
해당 OLE의 선택 테두리와 회전 핸들이 PNG 증적에 보인다. Rust OLE 선택 10개와 SO-SUEOP 렌더
11개 회귀도 모두 통과했다. 세부 명령과 결과는
[Stage 2 기록](../working/task_m100_3319_stage2.md)에 남겼다.
