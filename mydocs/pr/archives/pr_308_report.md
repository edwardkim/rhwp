# PR #308 최종 보고서 — Task #306 조사 + Task #310 vpos 진단 도구

## 결정

✅ **Merge 승인 (admin merge)** — Investigation/Spike PR 의 모범 사례로 평가

## PR 정보

- **PR**: [#308](https://github.com/edwardkim/rhwp/pull/308)
- **이슈**:
  - [#306](https://github.com/edwardkim/rhwp/issues/306) — 21_언어 페이지네이션 과잉 — **OPEN 유지** (본 증상 미해결, 정직 표기)
  - [#310](https://github.com/edwardkim/rhwp/issues/310) — LINE_SEG vpos 노출 + 검증 도구 — **CLOSED (이 PR 핵심 산출물)**
  - [#309](https://github.com/edwardkim/rhwp/issues/309) — Epic: 페이지네이션 LINE_SEG vpos 우선 모드 전환 — **OPEN (다음 단계)**
- **작성자**: @planet6897 (Jaeuk Ryu)
- **처리일**: 2026-04-25
- **Merge commit**: `a3d9039`

## 처리 절차

1. ✅ PR 브랜치 체크아웃 (`task306`)
2. ✅ devel 자동 merge 성공 (충돌 0건)
3. ✅ 검증: 992 passed / 0 failed, dump-pages vpos 출력 정상
4. ✅ WASM Docker 빌드 → 디버그 오버레이 1·2쪽 시각 확인 (작업지시자 판정)
5. ✅ admin merge → 이슈 #310 close 확인 (이미 CLOSED)

## 승인 사유 — Investigation/Spike PR 의 가치

작성자가 정직하게 "수정 미완" 으로 표기하셨지만, 본 PR 은 다음과 같은 실질적 산출물을 갖춘 **Investigation/Spike PR** 모범 사례:

### 1. 조사 결과 영구 보존
- `mydocs/tech/investigations/issue-310/line_seg_vpos_analysis.md` — 4샘플 비교 분석 (영구 자료)
- `mydocs/working/task_m100_306_analysis.md` — 가설 4개 + 휴리스틱 실험 결과
- 동일한 함정 반복 방지 (Column break 비활성화 → exam_math 회귀 등)

### 2. Epic 범위 축소의 데이터 입증
- 초기 가설: "페이지네이션 엔진 전면 재설계 (Epic 수준)"
- 4샘플 분석 결과: 21_언어만 vpos-reset 7건, 다른 3개 샘플 0건
- → "**단일 조건 분기로 축소 가능**" 으로 Epic #309 의 위험·비용 대폭 감소

### 3. 검증 도구 즉시 활용 가능 (Task #310)
- `dump-pages` vpos 정보 출력 (`vpos=N..M, [vpos-reset@lineN]` 마커)
- `--debug-overlay` vpos=0 리셋 앰버 점선+라벨 표시
- Epic #309 후속 작업 시 회귀 검증 자동화 가능

### 4. 메타 수준 기여
- 휴리스틱 실험 결과 명시 기록 → 동일 시도 반복 방지
- 후속 작업자(메인테이너 또는 다른 기여자) 즉시 진입 가능

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 992 passed / 0 failed / 1 ignored |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 유지) |
| `cargo test --test issue_301 / tab_cross_run` | ✅ 전부 통과 |
| `cargo clippy / wasm32 check` | ✅ clean |
| CI (원본) | ✅ 전부 SUCCESS |
| `dump-pages` vpos 출력 | ✅ `[vpos-reset@line9]` 마커 정확히 표시 |
| WASM Docker 빌드 | ✅ 성공 (pkg/rhwp_bg.wasm 10:17) |
| 디버그 오버레이 1·2쪽 시각 | ✅ 일반 동작 변화 0, 옵트인 도구만 추가 |

## 변경 내역

**코드 (4파일, 디버그 도구만):**
- `src/document_core/queries/rendering.rs` +52 -11 — `dump_page_items` vpos 정보 노출
- `src/renderer/render_tree.rs` +11 -2 — `TextLineNode` 에 `line_index/vpos: Option` 필드 추가
- `src/renderer/svg.rs` +53 — `--debug-overlay` 시 vpos=0 리셋 시각 표시
- `src/renderer/layout/paragraph_layout.rs` +4 -1 — `with_para_vpos` 사용

**문서 (다수):**
- `mydocs/plans/task_m100_306.md`
- `mydocs/working/task_m100_306_analysis.md`
- `mydocs/plans/task_m100_310{,_impl}.md`
- `mydocs/working/task_m100_310_stage{1,2,3}.md`
- `mydocs/report/task_m100_310_report.md`
- `mydocs/tech/investigations/issue-310/line_seg_vpos_analysis.md` (영구 자료)
- `mydocs/orders/20260425.md`

## 후속 작업 (Epic #309)

본 PR 의 도구 + 자료를 기반으로 다음 단계 작업 가능:
- vpos-reset 패턴 (FullParagraph 내부 리셋) 단일 조건 분기 구현
- 21_언어 19쪽 → 15쪽 정상화
- 다른 샘플 무회귀 보장 (vpos-reset 0건 패턴)

## 성과 — 모범 사례 정착

- "큰 문제를 작은 단위로 쪼개고, 각 단위 결론을 영구 보존" 패턴
- Investigation PR 의 실질 가치 증명
- 다른 기여자에게 "조사도 머지 가능하다" 는 모범 사례
