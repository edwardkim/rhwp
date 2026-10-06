# Task M100 #2278 Stage 2 완료보고서 — 3D 원형 (rAngAx=0 회전+원근)

- 이슈: #2278 "C2b: 3D 입체·ofPie 보조플롯 렌더"
- 브랜치: `local/task2278`
- 계획: `task_m100_2278_v2.md` (수행 v2) / `task_m100_2278_impl_v2.md` (구현 v2)
- 선행: Stage 1R + v2 (시어 투영 3D 막대·방, 시각판정 통과)
- 작성일: 2026-07-19

## 1. 정답지 정밀 판독 (계획 (a))

`pdf/chart/원형/3차원원형-2022.pdf` 임베드 비트맵(2702×1577) 픽셀 실측 +
샘플 XML 카메라 실측:

- **카메라 (chart1.xml)**: `rAngAx=0, rotX=30, rotY=0, perspective=30,
  hPercent=100, depthPercent=100`
- **타원**: rx=846.5px, ry=406px → **ry/rx = 0.480**. 앞/뒤 반타원 407.5px
  **완전 대칭** — 원근 나눗셈(비대칭) 모델은 실측과 모순 → 기각, 대칭 타원 유지
- **유도식 채택**: `ry/rx = sin(rotX)·cos(perspective/2°)` = sin30°·cos15° =
  **0.483** — 실측과 0.5% 이내 (계획서의 "fov=perspective/2" 해석을 배율
  캘리브레이션으로 사용, 1점 캘리브레이션)
- **측벽 높이**: 175px → **rx × 0.207** (`PIE3D_WALL_RATIO`). 한컴 스펙 표 43
  Pie.ThicknessRatio(반지름의 백분율) 구조, hPercent로 스케일
- 벽 음영: top 원색 대비 어두움 — `shade(rgb, BAR3D_SIDE_SHADE)`(−0.25)가 실측
  (파랑 top (103,122,224) vs 벽 (72,95,150))과 근사. 슬라이스 경계 흰 테두리 유
- 시작각: rotY=0 → 12시 시작·시계방향 (파랑 대슬라이스 전면 하단 — 실측 일치)

## 2. 구현 (renderer.rs — 2D 무접촉)

- `render_pie_3d` 신설 (v1 계획의 측벽 알고리즘 + v2 타원비 유도):
  - 타원비 `sin(rotX)·cos(persp/2°)` — 정의역 방어: rotX 5..90 클램프(0° 퇴화
    차단), perspective 0..90 클램프(cos 음수 차단)
  - **반경 fit은 타원+벽 블록 기준** — 2D의 원 fit(`min(pw,ph)/2`)이 아니라
    `min(pw/2, ph/(2·타원비+벽비)) × 0.9`. 납작한 타원의 세로 여유를 사용해야
    한컴 크기(실측 rx=846.5 ≈ 플롯 절반폭×0.85)와 정합 — 1차 산출물 대조에서
    파이가 과소했던 원인으로 확인 후 보정
  - **1차 루프(벽 먼저)**: 하반부 θ∈(0,π) 클립. rotY 오프셋 대응으로
    (0,π)+(τ,τ+π) **2윈도우 클립** — 랩어라운드 산술 없음
  - **2차 루프(top)**: 2D `render_pie` 슬라이스 로직의 타원호 버전
    (`A rx,ry` + `hwp-pie3d-top`), 색·흰 테두리 규칙 동일
- 훅: `render_chart_svg` Pie 단독 경로에 `is_3d` 분기 — 2D `render_pie` 무변경
  (2D 원형 바이트 불변)

## 3. TDD 증적

신규 7종 선작성 → **RED 확인(6 FAIL + 2D 가드 1 트리비얼 통과)** → 구현 → GREEN.
초회 2 FAIL은 테스트 파서가 SVG `A` 명령의 반지름 쌍을 좌표로 오집계한 문제
(구현 기하는 정확: 실측치 0.690 = (ry+wall)/rx 로 역산 일치) — 인덱스 보정.

| 테스트 | 검증 |
|---|---|
| `test_pie3d_ellipse_ratio_follows_rotx` | rotX=30→0.483 / rotX=60→0.837 (유도식) |
| `test_pie3d_wall_height_measured` | 벽 높이/rx == 0.207 |
| `test_pie3d_wall_lower_half_only` | [25,25,50] → 벽 2·top 3 + 벽 색 = shade(팔레트) |
| `test_pie3d_wall_clipped_at_boundaries` | 첫 벽 시작 (cx+rx,cy)·마지막 벽 끝 (cx−rx,cy) |
| `test_pie3d_walls_before_tops` | 페인트 순서(벽→top) |
| `test_pie_2d_no_pie3d_vocab` | 2D 가드 (3D 어휘 부재) |
| `test_pie3d_degenerate_cameras` | rotX=0/90/−15·persp=240/0 — 무NaN·비율 (0,1] |

통합(`issue_2278_chart_3d_ofpie.rs` stage2 파트): 3차원원형 × {hwpx,hwp} —
`hwp-pie3d-top`==4·`hwp-pie3d-wall`≥1·placeholder 부재.

## 4. 검증 결과

- ooxml_chart 유닛: **120 passed / 0 failed** (113+신규 7)
- 차트 통합·앵커 8스위트(2278 stage2 파트 포함): **전부 통과**
- `cargo clippy --all-targets -- -D warnings`: 무경고
- `cargo test` 전체: **exit 0, 269 스위트 전부 ok, 3,244 passed / 0 failed**
  (로그: `$TMPDIR/c2b_stage2_full_test.log`)
- fmt: 수정 파일 한정 (renderer.rs, issue_2278 테스트)
- 시각판정 산출물: `output/poc/chart_c2b/{hwp,hwpx}/3차원원형.svg` +
  대조 합성 `output/poc/chart_c2b/compare/3차원원형_한컴_vs_rhwp.png`

## 5. 위험·잔여

- perspective 배율 cos(persp/2°)는 코퍼스 1점(30) 캘리브레이션 — 다른 perspective
  실샘플 확보 시 재검(잔차 지배 항은 sin(rotX)라 영향 작음)
- 상반부 벽(내면)은 미방출 — 한컴 실측에서 노출 없음(도넛 아님)
- 데이터 라벨·리더선: 코퍼스 3차원원형에 없음 — 범위 외 유지
- Stage 3(ofPie + 팔레트 #5)에서 원형 계열 마무리
