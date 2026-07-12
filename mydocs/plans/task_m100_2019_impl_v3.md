# 구현계획서 v3 — #2019 절대배치 부동 폼 페이지네이션 근본 정정

- 이슈: #2019
- 브랜치: `task/2019-absolute-extent-pagination`
- 기준: `upstream/devel` + 기존 #2019 부분 완화(`8c46ca2`) 포함 상태
- 관련 PR: #2035 (`8c46ca2` revert 제안)

## 1. 재분류

기존 #2019 보고서의 "한글 2022 오라클과 완전 일치"와 "#2019 해소" 결론은 정정한다.

- `8c46ca2` 는 rhwp 페이지 수를 81쪽에서 18쪽으로 줄여, 서식 조각이 81쪽으로 흩어지는 최악의 증상을 막은 부분 완화다.
- 그러나 페이지 수만 맞춘 상태는 한컴 2022 정합이 아니다. PI-page oracle 과 시각 대조에서 본문/서식 배치 불일치가 남는다.
- 따라서 현재 회귀 테스트의 `pages <= 20` 은 "81쪽 폭증 재발 방지" 가드로 유지하되, #2019 완료 판정 근거로 쓰지 않는다.
- PR #2035 의 전면 revert 는 81쪽 산란을 되살리므로 그대로 병합하지 않고, 부분 완화는 유지한 채 근본 정정을 별도 브랜치에서 수행한다.

## 2. 목표

부동 글상자/도형/표로 구성된 별지 서식의 stored `LINE_SEG.vpos` 를 단순히 버리거나 0으로 만들지 않고, 한글처럼 쪽나누기/자연 오버플로에서만 분할되도록 한다. Paper 기준 절대 위치 개체는 page-local 실제 세로 extent 를 함께 보아 페이지 귀속과 zone 진행을 결정한다.

완료 조건:

- 74312 fixture 가 rhwp 18쪽을 유지한다.
- `pages <= 20` 뿐 아니라 한컴 2020 PDF 기준 visual sweep 대표 페이지에서 별지 서식의 라벨·격자·본문 흐름이 맞는다.
- PI-page oracle 은 보조 진단으로만 사용한다. 최종 기준은 `pdf/issue2019/issue2019_floating_form_74312-2020.pdf` 의 페이지별 시각 결과다.
- #1994, #2015, #2032, #1858, #703 등 절대배치/RowBreak/부동개체 영향권 테스트가 회귀하지 않는다.

현 구현 결과(2026-07-08):

- 74312 fixture 는 rhwp 18쪽을 유지한다.
- PDF 기준 visual sweep `output/task2019_v3_visual_scaffold_skip2/` 에서 SVG/PDF/render-tree 18/18/18, `flagged=0/18` 이다.
- 대표 결함이던 5쪽 본문 `pi78` 과 Paper 기준 Square 표 상단 겹침은 좌표 기준으로 해소됐다(`pi78` bottom 279.5px, table top 281.8px).
- 남은 차이는 한컴 PDF와 rhwp 간 글꼴/래스터 굵기 차이로, 페이지 귀속·큰 표/본문 흐름 판정에는 영향이 없다.

## 3. 현재 결함 가설

v2 의 `para_is_floating_overlay_anchor` 는 부동 폼 앵커의 flow footprint 를 과도하게 줄여 81쪽 산란을 막았지만, 그 자체가 한글 모델은 아니다. `task/2019-repursue` 보존 브랜치의 paramap 분석에 따르면 stored `LINE_SEG.vpos` 는 섹션 누적 흐름 좌표이며, 일부 개체 높이는 정상 흐름 진행의 일부다. 문제는 렌더러가 다음 두 값을 구분하지 못하는 데 있다.

| 값 | 의미 | 현재 위험 |
|----|------|-----------|
| 텍스트 flow cursor | 일반 문단/표가 다음 줄·쪽으로 진행할 위치 | 부동 폼 앵커에 stored LINE_SEG vpos/line_height 가 섞이면 81쪽 산란 |
| 절대배치 object extent | Paper/Para 기준 비-TAC 개체가 실제로 차지하는 y 범위 | 페이지 귀속·zone 높이에 반영하지 않으면 18쪽 안에서도 PI-page/시각 오정렬 |

근본 수정은 flow cursor 를 무작정 0으로 압축하는 것이 아니라, 쪽나누기 17개와 자연 오버플로 1개(pi117)를 복원하도록 flow 누적과 Paper-앵커 object extent 를 함께 계산하는 방향이다.

## 4. 단계 계획

### Stage 1 — 정정 기록과 oracle inventory

- 기존 최종보고서와 테스트 주석에서 "완전 일치" 표현을 제거한다.
- `tests/issue_2019_floating_form_overpagination.rs` 를 부분 완화 가드로 명시한다.
- `verify_pi_page_vs_hangul` 절차와 기존 oracle 파일을 확인해 74312 의 현재 PI-page mismatch 를 재현 가능한 산출물로 남긴다.
- 한컴 MCP/PDF 기준 파일이 없거나 최신이 아니면 `pdf/issue2019/` 에 보존한다.

현 상태(2026-07-08): 정정 기록, MCP 2020 PDF(`pdf/issue2019/issue2019_floating_form_74312-2020.pdf`), visual sweep 기준선(`output/task2019_v3_visual/`)까지 완료했다. 이 PDF 를 기준으로 대표 페이지 시각 차이를 줄인다. Windows 한컴 paramap 재생성은 원인 추적용 보조 자료로만 둔다.

### Stage 2 — PDF visual 기준 절대배치 extent 계측

- `dump-pages`/render tree 를 이용해 74312 의 부동 앵커별 문단 index, wrap, `tac`, `vert_rel_to`, offset, rendered bbox 를 캡처한다.
- PDF 기준 page 6/7/8 을 대표 페이지로 삼아 rhwp SVG/render tree 와 PDF raster 를 대조한다. `hwp_paramap.py` 계열 PI map 은 페이지 경계 원인 추적용 보조 자료다.
- `format_paragraph`, 단나누기 핸들러, `process_multicolumn_break` 에서 flow cursor 와 object extent 가 섞이는 지점을 계측한다.
- 정상 다단 문서(`shortcut.hwp`, #702 계열)와 비교해 다단 zone spacing 로직은 건드리지 않을 기준선을 잡는다.

### Stage 3 — 페이지/zone 진행 규칙 구현

- 부동 폼 앵커의 stored vpos/line_height 를 모두 제거하지 않는다. 개체가 vpos 를 진행시키는 Para-앵커/흐름 개체와, vpos 를 거의 진행시키지 않는 Paper-앵커 절대 개체를 분리한다.
- Paper/Page 기준 비-TAC object 는 앵커가 속한 page-local object extent 로 집계한다.
- Para 기준 비-TAC object 는 host paragraph 의 page-local 기준점과 offset 을 합산하되, 일반 flow advance 로 중복 가산하지 않는다.
- 다단 zone 전환에서는 `max_vpos_end` 같은 섹션 절대값을 zone height 로 쓰지 않고, flow height 와 object extent 를 분리해 page-local 값으로 클램프한다.

현 상태(2026-07-08): Paper/Page 기준 글상자 vpos origin 정규화, Paper overlay tail page-local overflow 판정, 부동 overlay 사이 ColumnDef-only 구분자 억제, 빈 앵커 Paper/Page Square 표의 절대 배치와 스캐폴드 빈 줄 숨김을 구현했다.

### Stage 4 — 검증

- 표적 테스트:
  - `cargo test --test issue_2019_floating_form_overpagination -- --nocapture`
  - #1994/#2015/#2032/#1858/#703 영향권 테스트
- 레이아웃 검증:
  - `rhwp dump-pages samples/hwpx/issue2019_floating_form_74312.hwpx`
  - 한컴 PDF visual sweep 대표 페이지 대조
  - PI-page oracle 대조는 보조 진단으로 기록
- 최종 PR 전 검증은 별도 승인 후 CI급 테스트와 WASM 빌드를 수행한다.

현 검증(2026-07-08):

- `cargo build --bin rhwp`
- `CARGO_INCREMENTAL=0 cargo test --test issue_2019_floating_form_overpagination -- --nocapture`
- `target/debug/rhwp dump-pages samples/hwpx/issue2019_floating_form_74312.hwpx` → 18페이지
- `scripts/task1274_visual_sweep.py --key issue2019 ... --out output/task2019_v3_visual_scaffold_skip2` → `flagged=0/18`

## 5. PR #2035 처리 방향

메인터너 판단으로는 revert 대신 유지+정정이 맞다.

- revert 는 18쪽 오정렬보다 더 나쁜 81쪽 산란을 복원한다.
- 다만 기존 테스트와 보고서가 완전 정합처럼 보이게 만든 점은 즉시 정정한다.
- #2035 에는 "부분 완화 전면 revert 대신 v3 보정으로 PDF 기준 visual sweep 을 통과시킨다"는 방향이 맞다. GitHub 코멘트/리뷰는 초안 승인 후 게시한다.

## 6. 위험과 완화

| 위험 | 완화 |
|------|------|
| object extent 를 flow advance 에 중복 반영 | flow cursor 와 absolute extent 를 별도 구조/변수로 분리 |
| 다단 zone spacing 회귀 | #702/shortcut 계열 dump-pages 와 테스트를 Stage 2 기준선에 포함 |
| 페이지 수만 맞고 시각이 틀어지는 반복 | `pages <= 20` 를 smoke guard 로 낮추고 PI-page/visual sweep 을 완료 조건으로 승격 |
| MCP/PDF oracle 비공개 정보 노출 | token/IP 는 공개 문서에 쓰지 않고, 산출 PDF 만 `pdf/issue2019/` 에 보존 |
