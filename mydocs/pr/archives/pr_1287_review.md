# PR #1287 검토 - HWPX 맞쪽 편집 여백 교대 보정

- PR: https://github.com/edwardkim/rhwp/pull/1287
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1196
- 작성일: 2026-06-04
- 작성자: @postmelee
- 제목: `Task #1196: HWPX 맞쪽 편집 여백 교대 보정`
- base: `devel`
- head: `local/task1196` / `fe9fc2a622f9f00768d6895c474f0f539a9be65d`
- 상태: open, non-draft
- GitHub mergeable: false

## 1. PR 요약

PR #1287은 HWPX `pagePr gutterType="LEFT_RIGHT"` 문서에서 맞쪽 편집 여백이
최종 쪽번호 홀짝에 따라 교대되지 않는 문제를 보정한다.

대상 샘플:

- `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`
- `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf`

현재 `devel` 기준으로 HWPX 파서는 이미 `LEFT_RIGHT`를
`BindingMethod::DuplexSided`로 읽는다. 남은 문제는 `PageAreas` /
`PageLayoutInfo`가 최종 `page_number`를 받지 않아, 짝수쪽에서도 항상
홀수쪽 여백 방향을 사용한다는 점이다.

## 2. 변경 범위

| file | 변경 |
|---|---|
| `src/model/page.rs` | `PageAreas::from_page_def_for_page(page_def, page_number)` 추가, DuplexSided 짝수쪽 좌우 여백 교대 |
| `src/renderer/page_layout.rs` | `PageLayoutInfo::from_page_def_for_page(...)`, `apply_page_number_margins(...)` 추가 |
| `src/document_core/queries/rendering.rs` | 구역 간 쪽번호 carry 이후 `PageContent.layout`과 `zone_layout`을 최종 쪽번호 기준으로 보정 |
| `tests/issue_1196_hwpx_gutter_left_right.rs` | 온새미로 HWPX page 4/5/6 body area 홀짝 교대 회귀 테스트 |
| `mydocs/...task_m100_1196...` | 계획/단계/완료 보고 문서 |
| `mydocs/orders/20260604.md` | #1196 작업 기록 |

## 3. 코드 검토

### 3.1 PageAreas 여백 교대

`PageAreas::from_page_def()`는 기존 호환을 위해 page 1 기준으로 위임하고,
새 API `from_page_def_for_page(page_def, page_number)`가 실제 홀짝을 처리한다.

계약:

- `SingleSided`: page 1/2 좌표 동일
- `DuplexSided` odd: `left + gutter`가 왼쪽
- `DuplexSided` even: `right`가 왼쪽, `left + gutter`가 오른쪽
- `TopFlip`: 이번 범위에서는 좌우 교대하지 않음
- `page_number=0`: 미확정 상태로 보고 기존 방향 유지

이 방향은 #1196의 HWPX `LEFT_RIGHT` 계약과 맞다.

### 3.2 최종 page_number 기준 보정 위치

`DocumentCore::paginate()`에서 다음 순서로 적용한다.

```text
1. section-local pagination
2. page_number_pos 상속
3. 구역 간 page_number carry 보정
4. apply_page_number_layouts_for_section()
5. assign_master_pages_for_section()
6. header/footer 상속 및 hide 처리
```

#1276에서 바탕쪽 선택을 최종 `page.page_number` 기준으로 옮긴 직후 레이어와 같은
정책이다. section-local page index가 아니라 최종 쪽번호를 쓰므로, 앞 구역의 carry로
홀짝이 뒤집히는 문서에도 맞는다.

### 3.3 리플로우 영향 제한

PR은 페이지네이션 중 body width를 바꾸지 않고, 최종 결과의 x 좌표를 후처리한다.
대상 문서처럼 좌우 여백이 교대되어도 본문 폭이 같아야 하는 케이스에서는 안정적이다.

주의점:

- `margin_gutter`가 있는 Duplex 문서에서도 body width가 유지되는지 단위 테스트가 있다.
- 실제 line wrap을 다시 계산하지 않으므로, 향후 홀짝에 따라 본문 폭 자체가 달라지는 문서가
  발견되면 별도 이슈로 다뤄야 한다.
- render path가 `page.layout` / `column.zone_layout`을 기준으로 본문과 column을 배치하므로,
  이번 보정은 SVG/Canvas path 모두에 반영되는 구조다.

### 3.4 zone layout 보정

같은 페이지 안에서 ColumnDef가 바뀌어 `ColumnContent.zone_layout`이 존재하는 경우에도
`apply_page_number_margins()`를 적용한다.

이 helper는 zone의 단 너비/간격을 재계산하지 않고 x delta만 적용한다. 따라서 다단 zone의
상대 배치와 기존 pagination 결과를 보존한다.

## 4. 현재 devel과의 통합 상태

현재 `local/devel`은 PR base보다 앞서 있다.

```text
2ff4c337 docs: Task #823 macOS 직접 검증 기록
562123c1 Task #823: headless macOS Skia font lookup hang 방지
7d2068f3 fix: narrow TAC alignment width handling
37991102 fix: compact endnote pagination refinements
62f3bf74 fix: HWPX 바탕쪽 글상자 번호 배치 보정
f2292883 fix: HWPX 글뒤로 표 페이지 밀림 보정
```

예상 충돌:

- `mydocs/orders/20260604.md`

현재 `devel`에는 이미 #1285, #823 작업 기록이 있으므로 PR의 새 파일 버전을 그대로
받으면 안 된다. 기존 표를 유지하고 #1196 행을 추가하는 방식으로 병합해야 한다.

## 5. 검증 계획

PR 본문 기준 통과:

- `cargo fmt --all -- --check`
- `cargo test`
- `cargo clippy -- -D warnings`
- `wasm-pack build --target web`
- rhwp-studio 수동 시각 확인

수용 전 로컬 검증 권장:

```text
cargo fmt --check
cargo test --lib page_areas
cargo test --lib page_layout
cargo test --test issue_1196_hwpx_gutter_left_right -- --nocapture
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
cargo test --test issue_1285_tac_sequence_right_align -- --nocapture
```

필요 시 확장:

```text
cargo test --tests --quiet
```

시각 판정 권장:

- `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`
- page 4/5/6 SVG debug overlay
- wasm build 후 rhwp-studio page 4/5/6 확인

확인 포인트:

- page 4와 page 6의 body 시작 x가 page 5보다 오른쪽인지
- page 4가 section 1 본문 시작이자 final `page_num=4` 상태를 유지하는지
- #1276의 바탕쪽/하단 글상자 번호 배치가 회귀하지 않는지
- #1285의 TAC 표 right-align 회귀가 없는지

## 6. 권장 처리

권장: 수용.

근거:

- #1196 원인인 `DuplexSided` page parity 미반영을 직접 해결한다.
- 파서 중복 수정 없이 이미 구현된 `gutterType` materialization 위에 레이아웃 정책만 추가한다.
- 최종 쪽번호 carry 이후 보정하므로 #1276의 master page parity 정책과 정합적이다.
- 코드 변경 범위가 작고, 회귀 테스트가 실제 대상 HWPX 샘플의 page 4/5/6을 고정한다.

권장 절차:

1. 최신 `local/devel` 기준 통합 브랜치 생성
2. PR commit `fe9fc2a6` cherry-pick
3. `mydocs/orders/20260604.md`는 기존 #1285/#823 기록 유지 + #1196 행 추가로 해결
4. PR 문서는 현행 archive 정책에 맞춰 이동 여부 확인
   - `mydocs/plans/archives/task_m100_1196*.md`
   - `mydocs/working/archives/task_m100_1196_stage*.md`
   - `mydocs/report/archives/task_m100_1196_report.md`
5. 포커스 테스트 실행
6. wasm 빌드 후 메인테이너 시각 판정
7. 통과 시 `devel` 병합/push 및 PR #1287, issue #1196 종료 처리

## 7. PR 코멘트 초안

```markdown
검토했습니다. 현재 `gutterType="LEFT_RIGHT"` 파싱은 이미 `BindingMethod::DuplexSided`로
materialize되고 있으므로, 이번 PR처럼 최종 `page_number` 기준으로 `PageAreas` /
`PageLayoutInfo`의 좌우 여백을 보정하는 접근이 #1196의 남은 문제를 잘 겨냥하고 있습니다.

특히 #1276에서 바탕쪽 선택을 구역 간 page number carry 이후로 옮긴 정책과 같은 위치에서
layout을 보정하므로, section-local page index가 아니라 실제 최종 쪽번호 홀짝을 사용할 수
있다는 점이 좋습니다.

다만 최신 `devel`에는 `mydocs/orders/20260604.md`가 이미 존재하고 #1285/#823 기록이 들어
있어 문서 충돌이 예상됩니다. 메인테이너 통합 브랜치에서 cherry-pick 후 기존 기록을 보존하고
#1196 행을 추가하는 방식으로 정리하겠습니다.

감사합니다.
```

## 8. 통합 진행 기록

통합 브랜치:

```text
local/pr1287-integration
```

적용:

```text
git fetch origin pull/1287/head:local/pr1287-upstream
git cherry-pick fe9fc2a622f9f00768d6895c474f0f539a9be65d
```

결과:

- `mydocs/orders/20260604.md` add/add 충돌 발생
- 기존 #1285/#823 기록 유지
- #1196 행 추가로 충돌 해결
- PR 작업 문서는 현행 정책에 맞춰 archive 위치로 이동

archive 이동:

```text
mydocs/plans/task_m100_1196.md -> mydocs/plans/archives/task_m100_1196.md
mydocs/plans/task_m100_1196_impl.md -> mydocs/plans/archives/task_m100_1196_impl.md
mydocs/report/task_m100_1196_report.md -> mydocs/report/archives/task_m100_1196_report.md
mydocs/working/task_m100_1196_stage1.md -> mydocs/working/archives/task_m100_1196_stage1.md
mydocs/working/task_m100_1196_stage2.md -> mydocs/working/archives/task_m100_1196_stage2.md
mydocs/working/task_m100_1196_stage3.md -> mydocs/working/archives/task_m100_1196_stage3.md
mydocs/working/task_m100_1196_stage4.md -> mydocs/working/archives/task_m100_1196_stage4.md
mydocs/working/task_m100_1196_stage5.md -> mydocs/working/archives/task_m100_1196_stage5.md
```

## 9. 로컬 검증 결과

자동 검증:

```text
cargo fmt --check
통과

cargo test --lib page_areas
test result: ok. 4 passed

cargo test --lib page_layout
test result: ok. 6 passed

cargo test --test issue_1196_hwpx_gutter_left_right -- --nocapture
test result: ok. 1 passed

cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
test result: ok. 3 passed

cargo test --test issue_1285_tac_sequence_right_align -- --nocapture
test result: ok. 2 passed

cargo test --tests --quiet
종료 코드 0

cargo clippy --all-targets -- -D warnings
통과
```

clippy 확인 중 기존 테스트 helper에서 `clippy::never_loop`가 발견되어 함께 보정했다.

보정 파일:

- `tests/issue_1156_chart_column_flow.rs`

검증:

```text
cargo test --test issue_1156_chart_column_flow -- --nocapture
test result: ok. 2 passed
```

## 10. WASM/SVG 시각 판정 준비

WASM 빌드:

```text
docker compose --env-file .env.docker run --rm wasm
Done in 2m 56s
```

rhwp-studio public 번들 반영:

```text
pkg/rhwp.js -> rhwp-studio/public/rhwp.js
pkg/rhwp_bg.wasm -> rhwp-studio/public/rhwp_bg.wasm
pkg/rhwp.d.ts -> rhwp-studio/public/rhwp.d.ts
pkg/rhwp_bg.wasm.d.ts -> rhwp-studio/public/rhwp_bg.wasm.d.ts
```

참고:

- `rhwp-studio/public/rhwp_bg.wasm`은 ignored 산출물이다.
- `rhwp-studio/public/rhwp.js`/d.ts는 이번 build 결과와 tracked 파일이 동일해 git diff에는 잡히지 않았다.

SVG debug overlay 산출물:

```text
output/poc/pr1287-task1196/[2027] 온새미로 1 본교재_004.svg
output/poc/pr1287-task1196/[2027] 온새미로 1 본교재_005.svg
output/poc/pr1287-task1196/[2027] 온새미로 1 본교재_006.svg
```

dump-pages 좌표:

```text
page 4: body_area x=189.0 y=113.4 w=510.2 h=895.8
page 5: body_area x=94.5  y=113.4 w=510.2 h=895.8
page 6: body_area x=189.0 y=113.4 w=510.2 h=895.8
```

메인테이너 시각 판정:

```text
2026-06-04 통과
```

## 11. CI 사전 점검

GitHub CI 최근 실패 빈도가 높아져, 일반 PR/push에서 실행되는 workflow 기준으로 추가 사전 점검을 수행했다.

Build & Test job 대응:

```text
cargo fmt --all -- --check
통과

cargo build --verbose
통과

cargo check --target wasm32-unknown-unknown --lib
통과

cargo test --features native-skia skia --lib --verbose
test result: ok. 39 passed

cargo test --verbose
통과

cargo clippy -- -D warnings
통과
```

참고:

- 로컬 `cargo build/test`에서 `failed to save last-use data` / `readonly database` 경고가 출력되었으나, cargo global cache 사용 기록 저장 경고이며 빌드/테스트는 종료 코드 0으로 통과했다.

Render Diff job 대응:

```text
node --check e2e/canvas-render-diff.test.mjs
통과

node --check e2e/run-render-diff.mjs
통과

npm run e2e:render-diff:ci
통과
```

Render Diff 결과:

```text
PASS: basic/KTX.hwp page 1: 94/889746 pixels differ (0.01056%), max channel delta 84
PASS: biz_plan.hwp page 1: 0/889746 pixels differ (0.00000%), max channel delta 0
PASS: tac-case-001.hwp page 1: 0/889746 pixels differ (0.00000%), max channel delta 0
```

참고:

- sandbox 내부에서는 Vite dev server port listen이 제한되어 render-diff가 포트 탐색 단계에서 실패했다.
- 동일 명령을 sandbox 외부에서 실행해 CI 성격의 실제 Vite/browser render-diff는 통과했다.

잔여 절차:

- `devel` 병합/push
- PR #1287 및 관련 이슈 종료 처리
