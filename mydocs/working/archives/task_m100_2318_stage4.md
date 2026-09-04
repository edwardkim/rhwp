# #2318 4단계 완료보고 — 전 게이트 + Header/Footer 점검

- 계획서: `mydocs/plans/task_m100_2318.md`
- 브랜치: `local/task2318`

## 게이트 결과

| 게이트 | 결과 |
|--------|------|
| `cargo test --tests --profile release-test --no-fail-fast` | **전 통과** (실패 0) |
| `cargo test --lib` | 2280/2280 (2단계에서 확인) |
| `cargo fmt --all --check` | 통과 |
| `cargo clippy --all-targets` | warning/error 0 |
| studio 단위 + tsc | 307/307 + 통과 (3단계) |
| CDP e2e | 4/4 PASS (3단계) |

## Header/Footer 유사 증상 점검

`samples/` 전수(**380개 .hwp, 실패 0**) 대상으로 dump 출력의 머리말/꼬리말
블록 내 개체 attr 의 wrap 비트(21~23)를 디코드:

- InFrontOfText(글 앞으로) 개체: **0건**
- BehindText(글 뒤로) 개체: **0건**

머리말/꼬리말 경로(`render_layer_from_control` — 자체 wrap 그대로)는 구조상
바탕쪽과 같은 승격이 가능하지만, 코퍼스에 노출 사례가 없고 한컴의 머리말
개체 z-order 의미론(본문 대비 앞/뒤)을 실증할 대비 샘플이 없다. **추정 정정
금지 원칙에 따라 이번 범위에서 제외** — 실제 샘플이 확보되면 별도 이슈로
처리한다.

## SVG 불변 확인

이번 변경은 paint plane 분류층에 국한된다. SVG 는 `node_z_plane` 이
MasterPage node_type 을 layer 보다 먼저 검사(plane 1 직접 배치)하므로
`master_page` 필드를 읽지 않고, 바탕쪽 내부 정렬 키(`paper_node_sort_key`)도
원본 wrap 을 그대로 사용한다. SVG z-order 계약 테스트(issue_1167/1197,
svg_snapshot 등) 전 통과로 고정.

## 시각 판정 요청 자료

| 자료 | 경로 |
|------|------|
| skia PNG (정정 전) | `output/png/issue2318/shortcut.png` — "1" 이 Ctrl+Y 를 덮음 |
| skia PNG (정정 후) | `output/png/issue2318/after/shortcut.png` — 한컴 일치 |
| studio 스크린샷 (정정 후) | `output/png/issue2318/studio-after.png` — Ctrl+Y/Alt+Y 가 "1" 위 |
| studio 실기 | vite 7700 + 최신 pkg/ 빌드 — shortcut.hwp 열어 확인 가능 |

정답지: 한컴 실기 (작업지시자 2026-07-17 확인 — 바탕쪽은 글자 뒤) +
`pdf/basic/shortcut-2022.pdf` p1.
