# [#5907] 문단 간 저장 vpos 되감김을 쪽 경계로 읽는다 — p122 1쪽 → 3쪽

- 일자: 2026-08-22
- 이슈: [#5907](https://github.com/edwardkim/rhwp/issues/5907)
- 기준 커밋: `72674c5653f09cb78b994dc4cd2dfd0a97ae6c8a` (`origin/devel`)
- 대상 문서: `samples/p122.hwp` / 정본 `pdf/p122-2022.pdf` (Hwp 2022 12.0.0.4426, Hancom PDF 1.3.0.550)

## 1. 증상

한/글 정본은 3쪽인데 rhwp 는 1쪽만 만들었다 (`render_page_samples.tsv` delta = −2).

| | 1쪽 | 2쪽 | 3쪽 |
|---|---|---|---|
| 한/글 2022 정본 | 공백 (문단 0) | 그림 (문단 1) | 공백 (문단 2) |
| rhwp devel | 문단 0 + 그림 + 문단 2 | — | — |
| rhwp 이 PR | 공백 (문단 0) | 그림 (문단 1) | 공백 (문단 2) |

문서에 저장된 미리보기(`PrvImage` — 한/글이 저장 시점에 직접 그린 1쪽 썸네일, 177×250 GIF)도
**공백**이다. 저장 시점에도 1쪽에 그림이 없었다는 독립 증거다.

부수 결과로 그림 위치도 어긋났다. 정본은 그림 좌상단을 본문 영역 원점(좌 30mm, 상 20mm + 머리말
15mm = 35mm)에 두는데, devel 은 문단 0 의 줄(1000 HU) + 줄간격(600 HU) = 1600 HU 만큼 아래로 밀린
자리에 그렸다 (96dpi 기준 y = 153.6px, 정본 132.3px).

## 2. 원인

`PARA_LINE_SEG` 의 `vertical_pos` 는 "단 안에서의 세로 위치"다. 이 문서의 본문 문단 3개는 **전부
vpos = 0** 이다.

```
문단 0.0  ls[0]: vpos=0, lh=1000,  th=1000,  cs=0, sw=42520, tag=0x00060000   (구역정의+단정의, 빈 문단)
문단 0.1  ls[0]: vpos=0, lh=22238, th=22238, cs=0, sw=42520, tag=0x00060000   (글자처럼 취급 그림 150.0×78.5mm)
문단 0.2  ls[0]: vpos=0, lh=1000,  th=1000,  cs=0, sw=42520, tag=0x00060000   (빈 문단)
```

문단 0 이 vpos 0 에서 시작해 1000 에서 끝났는데 문단 1 이 다시 vpos 0 을 주장한다. 같은 단 안에서는
불가능하므로 한/글이 그 사이에서 쪽을 넘긴 것이다. 문단 1 → 문단 2 도 같다.

`TypesetEngine` 에는 이미 문단 간 vpos 되감김 트리거(Task #321)가 있으나 단일 단 조건이

```rust
cv == 0 && pv > 5000 && ...
```

라서 "직전 문단이 쪽 하단부에 있었다"를 전제한다. 한/글이 짧은 문단 하나만 올리고 쪽을 넘긴
이 문서에서는 `pv = 0` 이라 트리거가 침묵했다.

`rhwp dump-pages` 는 불일치를 이미 관측하고 있었다.

```
단 0 (items=4, used=347.2px, hwp_used≈21.3px, diff=+325.8px)
```

## 3. 수정

`src/renderer/typeset.rs`

- `stored_vpos_top_collision(prev, curr)` 추가 — 앞뒤 문단이 **둘 다** 단 맨 위(stored vpos 0)를
  주장하는 충돌을 판정한다.
- 단일 단 트리거에 `stored_top_collision_reset` 을 disjunct 로 추가.
- `discard_terminal_blank_only_page` 에서 이 경계로 열린 끝 쪽은 보존한다 — 넘침 잔재(#3637)가
  아니라 한/글이 저장한 쪽 경계 그 자체이므로 한/글도 그 빈 쪽을 인쇄한다.

`src/renderer/pagination/engine.rs` — `RHWP_USE_PAGINATOR=1` 폴백 경로에도 같은 규칙을 동형으로 둔다.

### 오탐 가드

| 가드 | 이유 |
|---|---|
| `st.col_count == 1` | 다단은 단 경계에서 vpos 가 정상적으로 0 으로 되감긴다 — 기존 다단 경로가 담당 |
| `para.column_type == None` | 문단 자체에 나누기가 있으면 위에서 이미 처리 |
| `st.wrap_around_cs < 0` | 어울림 밴드 활성 중에는 vpos 재사용이 정상 |
| 양쪽 문단에 어울림(비 `treat_as_char`) 개체 없음 | 어울림 개체 옆 줄은 앞 문단과 같은 vpos 를 정당하게 쓴다 |
| 앞 문단이 가시 텍스트나 컨트롤을 가짐 | 문서 말미 빈 문단 연속(#1663 자리차지 표 뒤)은 저장 vpos 0 이 남아 있을 뿐 쪽 경계가 아니다 |
| 같은 단 기하 (`column_start`, `segment_width` 일치) | 다른 wrap zone 끼리 비교 방지 |
| 실제 파싱된 LINE_SEG (`TAG_FIRST_SEGMENT` + `segment_width > 0` + `line_height > 0`, 합성 tag bit31 제외) | 프로그램으로 만든 합성 IR(`LineSeg::default()`)은 vpos·tag·sw 가 모두 0 이라 "전부 단 맨 위"로 보인다 |

`tests/fixtures/render_page_samples.tsv` — `samples/p122.hwp` 행만 `1 / −2` → `3 / 0`.

## 4. 검증

전/후 바이너리를 각각 따로 빌드해 대조했다 (`release-test`, 전용 `CARGO_TARGET_DIR`).

| 게이트 | 수정 전 | 수정 후 | 판정 |
|---|---|---|---|
| 259문서 쪽수 게이트 정합 | 245 / 259 (94.6%) | **246 / 259 (95.0%)** | +1 |
| 회귀 (정합 → 불일치) | — | **0건** | ✓ |
| 쪽수 바뀐 문서 | — | `samples/p122.hwp` 1쪽 → 3쪽 (delta −2 → 0) **1건뿐** | ✓ |
| 코퍼스 SVG self-diff (259문서 × 앞 2쪽 = 518 렌더) | — | 차이 **p122 뿐** (p0 해시 변경, p1 신규 생성) | ✓ |
| `cargo test --profile release-test --lib -p rhwp` | — | **3893 passed / 0 failed** | ✓ |
| `cargo test --test regression_suite_011` | — | 통과 | ✓ |
| `rustfmt --edition 2021 --check` (변경 3파일) | — | 차이 0 | ✓ |
| `cargo clippy --all-targets -- -D warnings` | — | exit 0 | ✓ |
| `rust-unit-test-tiers.mjs --check` | — | 4225 (래칫 유지, src 테스트 추가 없음) | ✓ |

### red → green

새 테스트 `tests/cases/p122_stored_vpos_page_break.rs` (3건) 를 두 바이너리로 각각 실행했다.

| 바이너리 | 결과 |
|---|---|
| `origin/devel` | **3 failed** — `left: 1, right: 3` (쪽수), 2쪽 없음, `<image>` 없음 |
| 이 PR | **3 passed** |

### 초기 시도에서 잡은 오탐 2건 (본문 가드로 반영)

1. `samples/issue1663_coanchored_float_orphan.hwpx` 2쪽 → 4쪽. 문서 말미 빈 문단 3개가 전부
   vpos 0 이라 연쇄 발동. → "앞 문단이 가시 텍스트나 컨트롤을 가짐" 가드로 차단.
2. `renderer::typeset::tests` 2건 + `wasm_api::tests` 1건. 합성 IR(`LineSeg::default()`)의
   vpos·tag·sw 가 전부 0. → "실제 파싱된 LINE_SEG" 가드로 차단.

## 5. 남은 차이 (이 PR 범위 밖)

정본 2쪽의 **그림 크기**는 여전히 다르다. 문서가 저장한 크기는 150.0 × 78.5mm(42520 × 22238 HWPUNIT,
`SHAPE_COMPONENT` 의 초기·현재 크기, 변환행렬 항등, crop 없음)인데, 한/글 2022 의 PDF 출력은 같은
그림을 892.6 × 660.0mm(가로 ×5.95, 세로 ×8.41, 비균등)로 확대해 쪽 밖으로 잘라 내보낸다.

```
% pdf/p122-2022.pdf 2쪽 content stream
2529.8 0 0 1870.91 85.034 -1129.05 cm
/Im1 Do
```

이 확대는 문서가 저장한 어떤 값으로도 설명되지 않고, 한/글 자신이 저장한 LINE_SEG(`lh = 22238`
= 78.5mm)와도 모순된다. 재현 대상으로 보지 않고 이 PR 범위에서 제외한다. 이 PR 이 정합시킨 것은
**저장된 조판이 말하는 쪽 구조**(3쪽, 그림은 2쪽 단독, 좌상단 = 본문 원점)이며, 그림 원점은 정본과
일치한다(85.03pt, 99.13pt).

## 6. 시각 증빙

`mydocs/report/edit_demo_5907/`

| 파일 | 내용 |
|---|---|
| `p122_page_structure.png` | 쪽 구조 한 장 요약 (행 = 전/후/정본, 열 = 1~3쪽) |
| `p122_compare_p1.png` | 1쪽 전/후/정본 |
| `p122_compare_p2.png` | 2쪽 전/후/정본 |
| `p122_compare_p3.png` | 3쪽 전/후/정본 |

## 7. 다른 열린 PR 과의 충돌

`git merge-tree --write-tree <이 브랜치> <PR head>` 로 확인.

| PR | 결과 |
|---|---|
| #5900 | 충돌 없음 |
| #5903 | 충돌 없음 |
| #5904 | 충돌 없음 |
| #5905 | 충돌 없음 |
| #5909 | 충돌 없음 |
