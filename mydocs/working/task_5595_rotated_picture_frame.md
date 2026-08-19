---
kind: working
status: active
issue: 5595
---

# 90° 회전 그림의 정사각형 조판 정정 (#5595)

작업 브랜치: `fix/5595-rotated-picture-frame`
대상: `src/renderer/layout/utils.rs` · `tests/cases/issue_5595_rotated_picture_frame.rs` ·
`samples/issue5595_rotated_picture_topbottom.hwpx`

## 한 줄

회전 그림의 표시 크기를 **축별 max** 로 합치던 helper 가 90°/270° 그림에서 긴 변을 가로·
세로 양쪽에 넣어 정사각형을 만들었다. 회전 그림은 `common`(= 회전 후 외접 프레임)을 그대로
쓰도록 고쳤다.

## 이슈가 요구한 것

- 선언 188.5×134.0mm(53420×37986 HWPUNIT)인 그림이 712.3×712.3px 로 조판되어 지면 밖으로
  나가는 것을 멈춘다(보고 문서 00493, 가로 용지).
- 쪽수·저장 왕복은 정상이므로 **순수 렌더 크기 계산**만 건드린다.

## 원인

`picture_display_size_hu`(`src/renderer/layout/utils.rs`)는 `common.width/height` 와
`shape_attr.current_width/height` 중 **축별로 큰 값**을 채택했다. 회전 그림에서 이 두 값은
같은 크기의 다른 표현이다.

| 값 | 뜻 |
|----|----|
| `common.width/height` | 한컴이 저장한 **회전 후** 외접 프레임 (53420×37986) |
| `current_width/height` | **회전 전** 원본 표시 크기 (37986×53420) |

90° 에서는 두 축이 뒤바뀌므로 축별 max = (53420, 53420) — 긴 변이 두 축 모두에 들어가
정사각형이 된다. 이 계약은 저장 경로가 이미 세우고 있다
(`DocumentCore::refresh_picture_rotation_layout_for_save` 가 `picture_rotated_bounds` 로
`common` 을 회전 후 bbox 로 갱신한다).

경로별로는 다음이 갈렸다.

| 경로 | 크기 출처 | 수정 전 |
|------|-----------|---------|
| 글자처럼(TAC) · 어울림 SQUARE | `layout_picture_full` 의 회전 프레임 분기 / `picture_flow_frame_size_hu` 의 `common` 분기 | 정상 (712.3×506.5) |
| TopAndBottom float (`layout_body_picture`) | `picture_flow_frame_size_hu` → `picture_display_size_hu` 폴백 | **정사각형 (712.3×712.3)** |

## 수정

`picture_display_size_hu` 를 회전 인지로 바꿨다. `rotation_angle % 360 != 0` 이고
`common`·`current` 네 값이 모두 유효하면 `common`(회전 후 프레임)을 반환한다. 프레임이
비어 있으면(0) 종전 축별 max 폴백을 그대로 유지한다 — #1122 문26(손상된 `common`) 회귀 방지.

`picture_flow_frame_size_hu` 는 이 helper 를 폴백으로 쓰므로 TopAndBottom float 도 함께 닫힌다.

## 만지지 않은 경로

- `layout_picture_full` 의 `uses_rotated_frame` 분기 (이미 정상)
- 도형(shape)·수식·표 경로, 저장/직렬화 경로
- 새 CLI 명령 없음, DocumentCore 편집 로직 없음

## 재현 픽스처

`samples/issue5595_rotated_picture_topbottom.hwpx` (8.3KB, 합성). 00493 과 같은 축:
가로 용지(1122.5×793.7px), 그림 1개 `angle=90`, `sz`=53420×37986, `curSz`=37986×53420,
`treatAsChar=0` + `TOP_AND_BOTTOM`.

## 검증 실측

```
rhwp export-render-tree samples/issue5595_rotated_picture_topbottom.hwpx
  수정 전: Image bbox 712.3×712.3 @(113.4,132.3)   ← 정사각형
  수정 후: Image bbox 712.3×506.5 @(113.4,132.3)   ← 선언 188.5×134.0mm

rhwp layout-anomaly <같은 파일> --json
  수정 전: hasSignal=true  offCanvasCount=1 overflowCount=1 (nodeType=Image)
  수정 후: hasSignal=false offCanvasCount=0 overflowCount=0

rhwp export-svg <같은 파일>
  수정 전: <image width="712.27" height="712.27"> rotate(90,469.52,488.40)
  수정 후: <image width="506.48" height="712.27"> rotate(90,469.52,385.51)
           → 회전 후 지면 자국 712.3×506.5
```

## 시험 명령

```
cargo test --profile release-test --test regression_suite_029 issue_5595   # 신규 가드
cargo test --profile release-test --tests                                  # 전체
node scripts/rust-unit-test-tiers.mjs --check                              # source-side 총량 불변
```

신규 가드는 수정 전 코드에서 실패(`got 712.3×712.3`), 수정 후 통과를 확인했다.

## fmt 게이트

```
cargo fmt --all -- --check
cargo clippy -- -D warnings
```

`tests/generated/` 가 없는 새 워크트리에서는 fmt 전에
`node scripts/rust-test-suite-manifest.mjs --generate` 로 파생 하니스를 만든다(커밋하지 않는다).

## 환경

Linux 6.17 · rhwp v0.8.4 · cargo 1.93.1 · 한컴 오라클 없음(리눅스 단독 검증).
원 보고 문서(admrul 00493)는 이 환경에 없어 같은 기하의 합성 픽스처로 재현했다 —
수정 전 수치가 이슈 보고와 일치(712.3×712.3 @113.4,132.3).

## PR 메모

`gh pr create --base devel --body-file ...`, 제목·본문 한국어, `closes #5595`.
