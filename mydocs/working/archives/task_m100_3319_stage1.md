# Task M100 #3319 Stage 1 — HMapsi OLE 선택 경로 확인

- Issue: [#3319](https://github.com/edwardkim/rhwp/issues/3319)
- Branch: `task/3319-hwpx-ole-selection`

## 원인

`SO-SUEOP.hwpx` 1쪽 OLE는 HMapsi CFB preview 경로를 탄다. 이 경로는 화면용 `RawSvg`를
만들면서 `RawSvgNode::new()`를 사용해 원본 `(section, paragraph, control)` 참조를 버렸다.
`getPageControlLayout()`은 OLE ref가 있는 RawSvg만 `type: "ole"`로 방출하므로, Studio
`findPictureAtClick()`은 화면에 보이는 preview를 선택 후보로 찾을 수 없었다.

다른 OLE preview(OOXML chart, EMF, native image, placeholder)는 이미 `RawSvgNode::ole()` 또는
`PlaceholderNode::ole()`로 같은 참조를 보존하고 있었다.

## 보정

`push_hwpx_hmapsi_preview_clip_node()`에 section·paragraph·control 인자를 전달하고
`RawSvgNode::ole()`로 생성하도록 통일했다. 이 변경은 렌더 SVG·bbox·fallback 정책을 바꾸지 않고
선택용 control metadata만 복구한다.

## 현재 검증

격리 target `target/task-3319-hwpx-ole-selection`, `CARGO_INCREMENTAL=0`에서
`issue_2069_ole_object_selection`을 실행했다.

- 결과: 10 passed, 0 failed
- 추가 회귀: `so_sueop_hwpx_hmapsi_ole_preview_is_exposed_as_selectable_control`
- 다음 단계: 최신 WASM으로 Studio headless E2E를 실행해 실제 클릭 후 `ole` 선택 ref와 화면 증적을 확인한다.
