# 구현계획서 — Task #1700: 표 직후 빈 문단 페이지 배치 보존

- 이슈: edwardkim/rhwp#1700 / 브랜치: `local/task1700`
- 수행계획서: `task_m100_1700.md`

## 설계 결정 — 접근법 B (쿼리 표면화)

표 뒤 빈 문단은 이미 `ColumnContent.wrap_around_paras`(해당 페이지/단에 귀속)에
`para_index` + `has_text` 와 함께 **모델에 존재**한다. 레이아웃/렌더 트리
(`build_render_tree`)는 이 목록을 받아 표 옆에 정상 배치(시각 정상)한다.
누락은 오직 **`dump-pages` 쿼리(`rendering.rs`)가 `cc.items`만 순회**하기 때문이다.

→ 레이아웃 기하·페이지 수·시각을 **전혀 바꾸지 않고**, dump-pages 출력에
`cc.wrap_around_paras`를 `pi=` 라인으로 **추가 표면화**한다.

**안전성 근거**: 현재 MATCH 문서는 wrap-around 문단이 0개다(있으면 한글 대비
off-by-one으로 이미 불일치). 따라서 본 변경은 **불일치 문서에만** 영향하며,
MATCH 문서를 회귀시킬 수 없다(구조적 차단).

대안 A(빈 문단을 zero-height `PageItem`으로 `items`에 삽입)는 `wrap_around_paras`와
이중 배치/이중 렌더 위험이 있어 채택하지 않는다.

## 단계 (4단계)

### Stage 1 — 베이스라인/재현 고정
- 고정 18건(`output/poc/task1700_fixture_18.txt`) 수정 전 상태 기록.
- 대표 2건(별표 `17978249`, 예규 `2957879`) dump-pages에서 누락 PI 재확인.
- 산출: `mydocs/working/task_m100_1700_stage1.md`.

### Stage 2 — 구현 (rendering.rs dump-pages 표면화)
- `src/document_core/queries/rendering.rs` 의 dump-pages `for item in &cc.items`
  루프 직후, `cc.wrap_around_paras`를 순회하여 각 문단을
  `    WrapAroundPara  pi={para_index}  table_pi={table_para_index}  "{preview|(빈)}"`
  형식으로 출력(현재 페이지 헤더 하위 → 표가 있는 페이지에 귀속).
- 텍스트/빈 문단 모두 표면화(한글은 둘 다 본문 문단으로 카운트).
- `cargo build --release`.
- 산출: `mydocs/working/task_m100_1700_stage2.md`.

### Stage 3 — 검증
- 고정 18건 `verify_pi_page_vs_hangul.py --files …` → **18건 MATCH** 확인.
- 회귀: `cargo test`, `cargo test --test hwpx_roundtrip_baseline`.
- 광역: `verify_pi_page_vs_hangul.py --batch hwpdocs --sample 1000 --seed 42`
  재실행 → MATCH 932 → ~950, 신규 회귀(이전 MATCH→불일치) 0 확인.
- 산출: `mydocs/working/task_m100_1700_stage3.md`.

### Stage 4 — 최종 보고 + 커밋
- `mydocs/report/task_m100_1700_report.md` 작성.
- 소스 + 단계 보고서 + 계획서 커밋(`Task #1700: …`), `git status` 클린 확인.

## 완료 조건 (수행계획서 §3 재확인)
1. 18건 MATCH 전환. 2. 페이지 수 회귀 0. 3. 시각 회귀 0(렌더 트리 불변).
4. 신규 검증 회귀 0.
