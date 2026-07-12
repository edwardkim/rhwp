# 최종 결과보고서 — #2019 부동 폼 다수 별지 서식 과분할·조각 렌더

- 이슈: #2019 [Rendering/Layout] / 브랜치: `fix/2019-through-wrap-overlay-vpos`
- 검출: hwpdocs 8차 10k 서베이. 재현: `74312 벤처투자 시행규칙(안).hwpx` (rhwp 81p vs 한글 18p, 4.5×)

## 정정 — 2026-07-08

아래 최초 보고서의 "한글 2022 오라클과 완전 일치", "서식 렌더 정상화", "#2019 해소" 결론은 취소한다.

후속 PR #2035 검토와 PI-page/시각 대조 기준으로 재분류한 결과, `8c46ca2` 는 81쪽 산란을 18쪽으로 줄인 **부분 완화**다. 현재 `tests/issue_2019_floating_form_overpagination.rs` 의 `pages <= 20` 가드는 81쪽 폭증 재발을 막는 smoke test 로만 유효하며, 한컴 2022 시각 정합을 보장하지 않는다.

메인터너 판단으로는 이 부분 완화가 전면 revert 보다 낫다. 이 정정 시점에는 #2019 를 완료가 아니라 미해결 상태로 되돌려 보고, `task_m100_2019_impl_v3.md` 에서 Paper/Para 기준 절대배치 개체 extent 를 page-local pagination 에 반영하는 방향으로 다시 진행했다. 아래 v3 진행 결과가 그 후속 보정이다.

## v3 진행 결과 — 2026-07-08

MCP/Hancom 2020 PDF 를 기준으로 `task/2019-absolute-extent-pagination` 에서 v3 보정을 구현했다.
최종 판단 기준은 `pdf/issue2019/issue2019_floating_form_74312-2020.pdf` 의 페이지별 시각 결과이며, PI-page map 은 보조 진단으로만 사용한다.
원본 fixture 는 #2035 에서 삭제 대상으로 제시된 `samples/hwpx/issue2019_floating_form_74312.hwpx` 이며, `devel` 보존본 sha256 은 `092651a9cd41b9ece41abf0b4431a0e5c0a07d811ebb88375d208a1c92ba3d54` 이다.

구현 요약:

- Paper/Page 기준 비-TAC 글상자 내부 stored `LINE_SEG.vertical_pos` 에서 object origin 을 빼 이중 vertical offset 을 제거했다.
- 단나누기 뒤 Paper overlay tail 이 page-local body 하단을 넘는 경우에만 새 page 로 넘기고, 부동 overlay 사이 ColumnDef-only 구분자는 컬럼 진행에서 제외했다.
- 빈 앵커에 매달린 Paper/Page 기준 Square 표는 선언 y 에 절대 배치하고 flow cursor 를 전진시키지 않는다.
- 해당 절대 Square 표 앞뒤의 쪽나누기/ColumnDef/Bookmark, ColumnDef-only 스캐폴드 문단은 PDF 기준 별도 빈 줄로 렌더하지 않는다.

검증:

- `cargo build --bin rhwp`
- `CARGO_INCREMENTAL=0 cargo test --test issue_2019_floating_form_overpagination -- --nocapture`
- `target/debug/rhwp dump-pages samples/hwpx/issue2019_floating_form_74312.hwpx` → 18페이지
- `cargo test --profile release-test --test visual_roundtrip_baseline visual_baseline_all_samples -- --nocapture`
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `target/debug/rhwp render-diff samples/hwpx/issue2019_floating_form_74312.hwpx --via hwpx` → PASS, 18→18, 구조 불일치 0, 최대 변위 0px
- visual sweep: `output/task2019_v3_visual_scaffold_skip2/summary.json`
- SVG/PDF/render-tree pages: 18/18/18
- visual metrics: `flagged=0/18`
- 대표 육안 확인: `output/task2019_v3_visual_scaffold_skip2/issue2019/compare/compare_005.png`
- p5 좌표: `pi78` bottom 279.5px, table top 281.8px → 본문과 Paper 기준 Square 표 상단 겹침 없음

잔여 차이는 한컴 PDF와 rhwp 간 글꼴/래스터 굵기 차이다. 이번 v3 판정에서는 페이지 귀속·큰 표/본문 흐름 정합을 기준으로 통과 처리한다.

## 이전 결과 요약(취소됨)

**74312: 81페이지 → 18페이지 (한글 2022 오라클과 완전 일치) + 서식 렌더 정상화(표 격자·라벨 복원).**
덤으로 서베이 최대 과분할 문서 **3690000(노인복지관 타당성): 310 → 176페이지**(한글 177) 동반 해소.
**무회귀 0건** (300 랜덤 + 80 baseline + 전 테스트 스위트).

## 근본원인 (3층, 계측 확정)

부동 폼(별지 서식)의 stored LINE_SEG vpos 는 텍스트 흐름 좌표가 아니라 **개체의 섹션 절대
위치·높이(≈17p 캔버스)**를 인코딩한다. rhwp 는 이 앵커 문단을 흐름 콘텐츠로 취급해 3경로 과분할:

| 층 | 지점 | 현상 |
|----|------|------|
| ① 높이 | `format_paragraph` (typeset.rs) | stored line_height(=개체높이 51.3mm)를 흐름 예약→오버플로 |
| ② 단나누기 | 단나누기 핸들러 (typeset.rs) | 폼 구분자(빈문단+단나누기+같은 1단 ColumnDef)를 단일단서 페이지로 변환 |
| ③ zone 오프셋 | `process_multicolumn_break` vpos_zone_height (typeset.rs) | 섹션 절대 vpos(2204px)를 zone 오프셋→candidate_offset>page→1단↔2단 전환(71회)마다 새 페이지 |

배제: vpos-reset 트리거(0회), 명시적 쪽나누기(17개=한글 정합), 스타일 브레이크(false).

## 수정 (통합 게이트)

공통 술어 **`para_is_floating_overlay_anchor`**(layout.rs 신설): 빈 텍스트 + 전 컨트롤이 부동
비-TAC (Shape/Picture: 통과·글앞·글뒤 / Table: +어울림). 이 술어로 3경로에서 흐름 footprint 를 0 처리:
- ① 게이트 시 line_heights 를 빈문단 fallback 으로 대체.
- ② 게이트(또는 빈 텍스트 + ColumnDef 단독) 시 단일 단 단나누기 억제.
- ③ 이전 zone 마지막 문단이 게이트 대상이거나 **max_vpos_end 가 본문 높이 초과(누적 vpos 신호)**면
  zone 높이를 `st.current_height`(page-상대 흐름 누적값)로 대체.

수정 파일: `src/renderer/layout.rs`(+헬퍼), `src/renderer/typeset.rs`(①②③).

## 검증

### 시각 (한글 2022 대조)
- 74312 페이지 4 "투자목적회사에 관한 사항" 표 = 라벨·격자 완전 복원, 한글 페이지 5와 일치(export-png).

### 무회귀 (Stage 3)
- **80 baseline**: 변동 2건뿐(74312 81→18, 3690000 310→176 — 둘 다 한글 수렴), 78 불변.
- **300 랜덤(수정 전 pipage 대비)**: 변동 **0건** → 부동 폼 문서에만 국소 작용.
- **MORE 클러스터 44**: 개선 2, 동일 42, **악화 0** (#1937/#1921 표/줄 누적 클래스는 불변).
- `cargo test --lib`: **2143 passed / 0 failed**. `hwpx_roundtrip_baseline`: 4/4.
- **다단 통합 테스트**(process_multicolumn_break 경로): exam_eng_multicolumn·issue_1082/1156/1375/1488 전부 통과.
- `svg_snapshot` 8/0, `opengov_corpus_snapshot` 2/0.

### 회귀테스트
- `tests/issue_2019_floating_form_overpagination.rs`: 74312 페이지수 ≤20 assert.
- 픽스처 `samples/hwpx/issue2019_floating_form_74312.hwpx` (roundtrip PASS, baseline 자동포함).

## 이전 결론(취소됨)

부동 폼 앵커의 흐름 footprint 를 0 으로 통합 처리하여 74312 를 한글 정합(18p)으로 수렴시키고
서식 렌더를 정상화했다. 저장(roundtrip) 무영향, 다단·부동개체 문서 전 회귀 스위트 그린,
무회귀 0건. **#2019 해소.**
