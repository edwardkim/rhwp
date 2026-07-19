# PR #308 검토 — Task #306 조사 + Task #310 vpos 진단 도구 추가

## PR 정보

- **PR**: [#308](https://github.com/edwardkim/rhwp/pull/308)
- **이슈**:
  - [#306](https://github.com/edwardkim/rhwp/issues/306) — 21_언어 페이지네이션 과잉 (PDF 15쪽 vs SVG 19쪽) — **OPEN 유지**
  - [#310](https://github.com/edwardkim/rhwp/issues/310) — LINE_SEG vpos 노출 + 검증 도구 추가 — **CLOSED (이 PR로 해결)**
  - [#309](https://github.com/edwardkim/rhwp/issues/309) — Epic: 페이지네이션 엔진 LINE_SEG vpos 우선 모드 전환 — **OPEN (향후 작업)**
- **작성자**: @planet6897 (Jaeuk Ryu)
- **base/head**: `devel` ← `task306`
- **Mergeable**: ✅ MERGEABLE (자동 merge 성공)
- **CI**: ✅ 전부 SUCCESS
- **검토일**: 2026-04-25

## PR 제목 vs 실제 변경 (주목할 점)

PR 제목/본문은 "**#306 조사 (수정 미완)**" 으로 표현되어 있으나, 실제 커밋을 보면 **Task #310 (Epic #309 의 첫 단계) 까지 진행** 완료:

| 커밋 | 내용 | 상태 |
|------|------|------|
| `ef82d1a` | Task #306 수행계획서 | 조사용 |
| `8c77215` | Task #306 분석 보고서 (코드 미수정) | 조사 결론 |
| `7e7d99a` | **Task #310 1단계: dump-pages vpos 출력** | 코드 변경 |
| `621d0ba` | **Task #310 2단계: debug-overlay vpos=0 리셋 표시** | 코드 변경 |
| `e56c18b` | **Task #310 3단계: 4샘플 분석 + 최종 보고서** | 분석 |

### 핵심 가치 — Epic 범위 축소

작성자 분석의 결정적 발견:

> **21_언어 +4쪽 과잉의 원인이 FullParagraph 내부 vpos-reset 7건으로 특정됨. 다른 3개 샘플은 이 패턴 0건이므로 Epic #309 다음 단계 작업 범위가 전면 재설계에서 단일 조건 분기로 축소 가능.**

→ 초기 가설 ("페이지네이션 엔진 전면 재설계, Epic 수준") 을 4샘플 비교로 **단일 조건 분기 규모로 축소**. Epic #309의 위험·비용을 크게 줄임.

## 변경 내역

### 코드 (4파일, 디버그 도구만)

| 파일 | 변경 | 영향 |
|------|------|------|
| `src/document_core/queries/rendering.rs` | +52 -11 | `dump_page_items` 에 vpos 정보 노출 |
| `src/renderer/render_tree.rs` | +11 -2 | `TextLineNode` 에 `line_index/vpos` 필드 추가 (기본 None) |
| `src/renderer/svg.rs` | +53 | `--debug-overlay` 시 vpos=0 리셋을 앰버 점선+라벨로 표시 |
| `src/renderer/layout/paragraph_layout.rs` | +4 -1 | `with_para_vpos` 사용 |

**중요**: 일반 출력 동작 변경 **0**. CLI 진단 출력 + `--debug-overlay` 시각 표시만 추가. 옵트인 디버그 도구.

### 문서 (다수)

- `mydocs/plans/task_m100_306.md` — 수행계획서
- `mydocs/working/task_m100_306_analysis.md` — Task #306 분석 보고서
- `mydocs/plans/task_m100_310{,_impl}.md` — Task #310 수행/구현 계획서
- `mydocs/working/task_m100_310_stage{1,2,3}.md` — 단계별 보고서
- `mydocs/report/task_m100_310_report.md` — 최종 보고서
- `mydocs/tech/investigations/issue-310/line_seg_vpos_analysis.md` — 4샘플 비교 분석 (영구 자료)
- `mydocs/orders/20260425.md` — orders 갱신

## 메인테이너 검증 결과

### PR 브랜치 + devel merge

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ **992 passed / 0 failed / 1 ignored** |
| `cargo test --test svg_snapshot` | ✅ 6 passed (golden 유지) |
| `cargo test --test issue_301` | ✅ 1 passed |
| `cargo test --test tab_cross_run` | ✅ 1 passed |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32` | ✅ clean |

### 진단 도구 동작 검증

`cargo run --release --bin rhwp -- dump-pages "samples/21_언어_기출_편집가능본.hwp" -p 2` 출력:

```
FullParagraph  pi=44  h=14.7 ... vpos=0       "[4~6] ..."
FullParagraph  pi=45  h=14.7 ... vpos=1816    "(빈)"
FullParagraph  pi=46  h=88.0 ... vpos=2476..11556  "15세기 초 ..."
...
PartialParagraph  pi=50  lines=0..9   vpos=75116..0  [vpos-reset@line9]
PartialParagraph  pi=50  lines=9..12  vpos=0..3632   [vpos-reset@line9]
```

**작성자 분석의 핵심 패턴 자동 검증 가능**: `[vpos-reset@line9]` 마커가 정확히 표시됨.

### CI (원본 브랜치)

| Check | 결과 |
|-------|------|
| CI / Build & Test | ✅ SUCCESS |
| CodeQL rust/js/python | ✅ 전부 SUCCESS |

## 충돌 분석

- **자동 merge 성공** (충돌 0건)
- 어제 PR #305 가 머지됐지만 영역이 겹치지 않음 (PR #305: parser/body_text.rs `flags & 0x0004` / 이 PR: rendering.rs `dump_page_items` + render_tree.rs)

## 평가 포인트

1. **조사 → Epic 분리 → 범위 축소** 의 모범 — Task #306 조사로 "전면 재설계 필요" 결론 → Task #310 진단 도구로 **4샘플 패턴 분석** → "단일 조건 분기로 축소 가능" 입증
2. **무위해 디버그 도구** — 일반 출력 동작 변경 0. `dump-pages` 와 `--debug-overlay` 만 보강
3. **영구 자료 등록** — `mydocs/tech/investigations/issue-310/line_seg_vpos_analysis.md` 4샘플 비교 분석 저장
4. **이슈 흐름 명확** — #306 OPEN 유지 (실제 수정은 미완), #310 CLOSED (도구 등록), #309 OPEN (다음 단계)
5. **검증 도구의 가치** — vpos-reset 마커 자동 표시로 Epic #309 후속 작업 시 회귀 검증 즉시 가능

## 약간의 아쉬움 (개선 가능)

- **PR 제목/본문이 실제 변경과 약간 불일치** — 제목 "Task #306 조사 (수정 미완)" 인데 실제로는 Task #310 코드 변경 + 문서 다수 포함. PR 본문 하단에 "Task #310 단계 1~3 도 함께 진행" 명시했으면 더 명확했을 것
- 단, 본문 핵심 결론 ("LINE_SEG vpos 우선 모드 재설계 필요") 은 정확히 기록됨

## 리스크 평가

| 리스크 | 판정 |
|--------|------|
| 일반 사용자 영향 | ✅ 없음 — `dump-pages` 와 `--debug-overlay` 만 변경 |
| `TextLineNode` 필드 추가 | ✅ Optional `line_index/vpos: None` 기본값. 호출자 변경 최소 |
| Golden SVG 회귀 | ✅ svg_snapshot 6 passed |
| WASM Canvas 영향 | ✅ debug 도구는 SVG 전용 |
| 향후 Epic #309 작업 기반 | ✅ vpos 진단 도구가 회귀 검증 자동화 가능 |

## 판정

✅ **Merge 권장**

**사유:**
1. **Epic 범위 축소** — "전면 재설계" → "단일 조건 분기" 로 4샘플 데이터 기반 입증. Epic #309 의 위험·비용을 크게 줄인 핵심 기여
2. **무위해 디버그 도구** — 일반 출력 변화 0, 옵트인 도구만
3. **영구 자료 등록** — `tech/line_seg_vpos_analysis.md` 누적 자료
4. **이슈 흐름 정직** — #306 OPEN 유지 (실제 수정 미완 명시), #310 CLOSED (도구), #309 OPEN (Epic 다음 단계)
5. **CI + 로컬 검증 + 진단 도구 동작** 모두 통과
6. **CLAUDE.md 절차 준수** — 두 타스크 (#306, #310) 모두 계획서/보고서 완비

**Merge 후속:**
- 이슈 #310 close 확인 (이미 CLOSED)
- 이슈 #306 OPEN 유지 (Epic #309 작업 시 참고 자료)
- 이슈 #309 (Epic) 향후 작업 시 본 PR 의 도구 활용

**WASM 시각 검증 권장 (선택)** — `--debug-overlay` 의 vpos=0 리셋 시각 표시 효과 확인 가능. 단 일반 출력은 영향 없으므로 우선순위 낮음.
