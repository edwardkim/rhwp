# PR #256 검토 문서

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#256](https://github.com/edwardkim/rhwp/pull/256) |
| 작성자 | @planet6897 (Jaeuk Ryu) |
| 제목 | Task #146: text-align.hwp SVG ↔ 한컴 PDF 렌더링 일치 |
| base ← head | **devel ← devel** (기여자 fork 의 devel 브랜치 사용) |
| 관련 이슈 | [#146](https://github.com/edwardkim/rhwp/issues/146) |
| 변경 | +1,766 / -10 (24 파일, 12 커밋) |
| mergeable | CONFLICTING · DIRTY |
| 충돌 | 1 파일 (`mydocs/orders/20260423.md` — add/add) |
| Reviews/Comments | 0 |

## 기여자 배경

@planet6897 님은 v0.2.1 사이클에서 PR #221 (OLE/Chart/EMF 네이티브 렌더링) 을 14 Stage 에 걸쳐 기여하신 주요 기여자. 본 PR 은 두 번째 대형 기여.

## 변경 범위

### 코드 수정 3건 (정확히 근거 있는 진단)

기여자의 진단 방법: **`mutool draw -F stext` 로 PDF 문자 좌표 추출 → SVG 좌표와 1:1 비교 → 수치로 원인 특정**.

#### 1. Geometric Shapes 전각 처리 (`src/renderer/layout/text_measurement.rs`)

- `is_fullwidth_symbol` 의 범위 리스트에 **U+25A0-U+25FF** (□■▲◆○ 등 Unicode Geometric Shapes 블록) 추가.
- 문제: 섹션 머리 기호 □ 가 반각으로 측정되어 이후 전체 텍스트가 좌측으로 ≈em/2 만큼 붕괴.
- 효과: 제목 "국" 시작 x 94.40 → 105.40 (PDF 환산 105.39 와 **0.01 px 일치**).

#### 2. TAC 표 선행 텍스트 폭 반영 (`src/renderer/layout.rs`)

- `layout_table_item` 에 `compute_tac_leading_width` 헬퍼 추가.
- 문제: pagination 이 TAC 문단에 `PageItem::Table` 만 발행 (FullParagraph 미발행) 할 경우, `paragraph_layout` 의 TAC 분기가 호출되지 않아 선행 공백이 표 x 좌표에 반영되지 않음.
- block 취급 TAC (너비 ≥ 90% seg_width) 도 커버하는 fallback 포함.
- 효과: 표 첫 셀 x 75.59 → 109.59 (PDF 환산 ≈112 와 2.4 px 이내).

#### 3. Heavy display face → visual bold (`src/renderer/style_resolver.rs` · `src/renderer/mod.rs` · `src/renderer/svg.rs`)

- `is_heavy_display_face` 헬퍼 + `TextStyle::is_visually_bold` 메서드 추가.
- 문제: HY헤드라인M / HY견고딕 / HY견명조 / HY그래픽 등 face 자체가 굵은 display 폰트가 fallback 으로 regular 렌더됨.
- 효과: 제목이 PDF 와 시각적으로 근사한 굵기로 렌더.

### 기각된 초기 가설 (진단 품질)

기여자가 초기 3가지 가설 중 2개를 **좌표 비교로 기각**한 기록을 남김. 이는 "디버깅 지식 자산" 으로 가치 있음.

- ② Justify SVG 미반영 — **기각** (실제 0.12 pt 오차로 정상 동작)
- ③ Hanging indent 어긋남 — **기각** (실제 0.04 pt 오차로 정상 동작)

### 문서

- `mydocs/plans/task_m100_146{,_v2,_v3,_v4}{,_impl}.md` — 9 파일
- `mydocs/report/task_m100_146_report{,_v4}.md` — 2 파일
- `mydocs/working/task_m100_146_stage{1,2,4,5,6}.md` — 5 파일
- `mydocs/orders/20260423.md` — 1 파일 (충돌 원인)

기여자가 수행계획서를 **v1→v2→v3→v4 로 점진 확장**. 각 단계마다 작업지시자 승인을 가정한 하이퍼-워터폴 구조 완전 준수.

### 테스트

- 단위 테스트 6건 신규 (`src/renderer/layout/tests.rs` +141 라인)
- `form-002/page-0.svg` golden 갱신 (+6/-6)

### 샘플

- `samples/text-align.hwp` 신규 추가 (진단 대상 샘플)

## 사전 검증 결과 (로컬)

`pr256-merge-test` 브랜치에서 `git merge origin/devel` 시뮬레이션 수행:

| 항목 | 결과 |
|------|------|
| 자동 머지 충돌 | **1 파일** (`mydocs/orders/20260423.md` — add/add) |
| 충돌 해결 방식 | 우리 orders 전체 유지, 기여자 Task #146 섹션을 후속 편집에서 간결하게 추가 (작업지시자 지시) |
| `cargo build --lib` | 성공 |
| `cargo test --lib` | **947 passed** / 0 failed / 1 ignored |
| `cargo clippy --lib -- -D warnings` | **0 warning** |
| `cargo test --test svg_snapshot` | 3 passed (form-002 · table-text · determinism) |

기여자 본인이 보고한 "933 passed / 14 failed" 는 구 devel 기반 측정. 현재 최신 devel 기반으로는 **모두 그린** (우리 사이클에서 개선된 것들이 반영됨).

## 주요 관찰 사항

### 긍정

1. **PDF 좌표 정밀 비교 방법론** — 우리가 이슈 #253 으로 구상 중인 Visual Diff 의 **정확한 사례**. 기여자가 이미 `mutool` 로 좌표 추출하여 사용 중이라는 사실이 중요.
2. **기각된 가설 기록** — "디버깅 과정의 투명성" 자산. 다른 기여자가 같은 가설을 다시 검증하지 않아도 됨.
3. **수행계획서 v1~v4 점진 확장** — 범위를 한 번에 확정 안 하고 발견에 따라 확장. rhwp 방법론 정합.
4. **샘플 (text-align.hwp) 동반** — 회귀 재현 가능성 담보.
5. **스모크 회귀 검증** — exam_kor / biz_plan / draw-group 등 다른 샘플에서 회귀 없음 확인.

### 주의할 점

1. **base = devel, head = devel** — 기여자 fork 의 devel 브랜치 자체에서 작업. feature 브랜치 사용 권장 관례와 다르나, 실질적 문제는 없음. PR #237 때 언급한 "Fork main 에서 직접 작업 금지" 와 달리 **기여자 devel** 은 덜 민감. 다만 다음 기여 때는 `local/taskN` 같은 feature 브랜치 권장 코멘트 남기기.

2. **orders 충돌** — add/add 충돌은 드문 편. 작업지시자 지시대로 "우리 orders 유지 + 기여자 Task #146 섹션을 간결하게 추가" 로 해결.

3. **golden 갱신 포함** — `tests/golden_svg/form-002/page-0.svg` +6/-6. 수정 1 (Geometric Shapes) 의 영향으로 "□" 이후 텍스트 위치가 9px 달라진 결과. 기여자 보고대로 **의도된 변경** 이 맞음 (한컴 PDF 와 일치 방향).

## 처리 방식 권고

**Admin merge** (기존 패턴 — PR #209, #214, #215, #221, #224, #251 동일).

단, 충돌이 있어 GitHub UI 의 단순 "Merge pull request" 는 작동 안 함. 두 경로 중 하나:

- **경로 A — GitHub UI 에서 직접 충돌 해결** (GitHub 웹 에디터 에서 `mydocs/orders/20260423.md` 편집)
- **경로 B — 로컬에서 머지 커밋 생성 후 push** (우리 방식)

권고: **경로 B** — 로컬 `local/devel` 에 `git merge local/pr256` 실행 + orders 충돌 해결 + 검증 후 push. GitHub 쪽에는 해당 커밋이 push 되면 PR 이 자동 `MERGED` 로 판정되지 않을 수 있어, 머지 후 수동으로 close + 링크 코멘트 필요할 수 있음.

## 구현 계획서

→ `mydocs/pr/pr_256_review_impl.md`
