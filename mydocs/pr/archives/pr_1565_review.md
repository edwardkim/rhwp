# PR #1565 검토 문서 — HWPX 폼 컨트롤 caption 표시 정합 보정

## 1. PR 메타

| 항목 | 내용 |
|------|------|
| PR | #1565 — Task #1562: HWPX 폼 컨트롤 caption `&&` 표시 정합 보정 |
| 작성자 | @postmelee |
| base | `devel` |
| head | `postmelee:local/task1562` |
| 검토 경로 | collaborator self-merge 후보 예외 경로 |
| 문서 작성 위치 | `mydocs/pr/archives/pr_1565_review.md` |
| 문서 작성일 | 2026-06-27 |

작성 시점 참고값:

| 항목 | 값 |
|------|----|
| 최신 head SHA | `ceb736b30f5482b66a52b68aabcec357cdd8a6dd` |
| draft | false |
| mergeable | MERGEABLE |
| merge state | CLEAN |
| review decision | 없음 |
| labels | `hwpx`, `rendering`, `regression` |
| milestone | `v1.0.0` |
| assignee | @postmelee |

위 값은 문서 작성 시점 참고값이다. merge 전에는 최신 PR head, mergeable 상태, GitHub Actions 상태를 반드시 다시 확인한다.

## 2. 경로 판정

이 PR은 외부 contributor PR이 아니라 collaborator 본인 PR이다. 따라서 `mydocs/manual/pr_review_workflow.md` 8장의 collaborator self-merge 후보 예외 경로를 적용한다.

- PR 번호가 이미 확정되어 review 문서를 PR head에 포함할 수 있다.
- merge 후 별도 문서 커밋을 만들지 않기 위해 archive 경로의 review 문서를 PR diff에 포함한다.
- 작업지시자 승인 전에는 approve review, merge, issue close를 수행하지 않는다.
- 작성자와 reviewer 계정이 같으므로 별도 review request는 등록하지 않는다.

이번 PR은 별도 `pr_1565_review_impl.md`를 만들지 않고, 이 문서에 리뷰 계획과 검토 결과를 통합한다.

## 3. 관련 이슈

| 이슈 | 상태 | 비고 |
|------|------|------|
| #1562 — HWPX 폼 컨트롤 caption `&&`가 한컴과 다르게 `&&`로 표시됨 | OPEN | PR 본문에 `Closes #1562`로 연결됨 |
| #1534 — HWPX 저장 시 폼 컨트롤 속성값 XML 특수문자 이중 이스케이프 누적 | OPEN | PR 본문에 `Refs #1534`로 연결됨. #1536과 이 PR merge 후 close 판단 가능 |

#1562는 이 PR의 직접 해결 대상이다. #1534는 PR #1536에서 저장 계층의 XML escape 누적 손상이 해결됐고, 이 PR은 남아 있던 표시 계층 호환성 문제를 해결한다.

## 4. 변경 범위

핵심 변경:

- `src/renderer/form_caption.rs`에 form caption 표시 전용 helper `display_form_caption()` 추가
- form caption 표시 문자열에서만 `&&`를 literal `&`로 접음
- 단일 `&`는 보존하고, `&&`가 없는 caption은 `Cow::Borrowed`로 반환
- SVG, Web Canvas, Skia renderer의 PushButton, CheckBox, RadioButton caption 출력 경로에 helper 적용
- HWPX parser, serializer, 저장 모델의 caption 값은 변경하지 않음
- `samples/hwpx/form-002.hwpx` 기반 SVG 표시 회귀 테스트 추가
- `tests/golden_svg/form-002/page-0.svg` 갱신
- task 계획, 단계 보고, 최종 보고 문서 추가

변경 파일:

| 파일 | 검토 요약 |
|------|-----------|
| `src/renderer/form_caption.rs` | 표시 전용 `&&` collapse helper와 단위 테스트 추가 |
| `src/renderer/mod.rs` | helper module 등록 |
| `src/renderer/svg.rs` | form caption SVG text 출력 직전 display helper 적용 |
| `src/renderer/web_canvas.rs` | canvas `fill_text` 직전 display helper 적용 |
| `src/renderer/skia/renderer.rs` | Skia form caption draw path에 display helper 적용 |
| `tests/issue_1562_hwpx_form_caption_display.rs` | `form-002.hwpx` page 0 SVG의 `R&D` 표시 회귀 고정 |
| `tests/golden_svg/form-002/page-0.svg` | 폼 caption 3곳의 golden을 `R&amp;D` 표시로 갱신 |
| `mydocs/**/task_m100_1562*` | 내부 task 계획, 단계 보고, 최종 보고 |
| `mydocs/orders/20260626.md` | task 진행 기록. 최신 devel과 add/add 충돌 해소 완료 |

## 5. 범위 밖

이번 PR은 다음을 변경하지 않는다.

- HWPX 저장값 변환 (`R&&D` 저장값 유지)
- XML attribute 전역 치환
- serializer에서 `&&`를 `&`로 바꾸는 처리
- 일반 본문 텍스트의 `&&` 표시 처리
- 표, 그림, 도형의 일반 `<hp:caption>` 표시 처리
- 단일 `&`를 mnemonic prefix로 숨기거나 access-key 밑줄을 표시하는 처리

단일 `&`의 mnemonic 처리까지 확장하려면 한컴 실물 샘플과 표시 근거를 추가로 확보한 뒤 별도 이슈로 다루는 편이 안전하다.

## 6. 리뷰 계획

검토는 다음 순서로 진행했다.

1. PR metadata 확인
   - base/head, 작성자, label, milestone, assignee, mergeable, check rollup 확인
2. 경로 판정
   - 외부 contributor PR이 아니라 collaborator self-merge 후보 예외 경로로 분류
3. 변경 코드 검토
   - display helper가 저장 계층이 아닌 표시 계층에만 적용되는지 확인
   - SVG, Web Canvas, Skia form caption 경로에만 영향이 제한되는지 확인
4. 테스트 검토
   - 신규 SVG 표시 회귀 테스트가 `R&&D` 잔존을 막는지 확인
   - #1534 저장 escape 회귀 테스트와 충돌하지 않는지 확인
5. 충돌 확인
   - 최신 `upstream/devel`과 최초 merge simulation에서 `mydocs/orders/20260626.md` add/add 충돌 확인
   - 작업지시자가 conflict 해소 후 최신 head `ceb736b3` 기준 `MERGEABLE/CLEAN` 확인
6. 검증
   - 로컬 targeted test와 GitHub Actions 최신 결과 확인

## 7. 코드 리뷰 결과

### 7.1 표시값과 저장값 분리

`display_form_caption()`은 form caption의 사용자 표시 문자열만 변환한다. helper는 `&&`가 있을 때만 새 문자열을 만들고, 그렇지 않으면 borrowed 값을 반환한다. parser/serializer/storage 경로에는 연결되지 않는다.

판정: 적절함. #1534의 저장값 보존 요구와 충돌하지 않는다.

### 7.2 변환 규칙

현재 규칙은 `&&`를 literal `&` 한 글자로 표시하는 데 한정된다. 단일 `&`는 그대로 보존한다.

검증 예:

- `R&&D` -> `R&D`
- `IP R&&D연계` -> `IP R&D연계`
- `&&&&` -> `&&`
- `R&D` -> `R&D`

판정: 이번 이슈의 한컴 뷰어 관측 결과와 UI caption escape 관례에 맞다. 단일 `&` mnemonic 처리는 근거가 부족하므로 제외한 판단이 안전하다.

### 7.3 렌더러 적용 범위

helper 적용 지점은 form control caption을 사용자에게 그리는 경로로 제한되어 있다.

- SVG: `<text>` 출력 전 `escape_xml(display_caption)` 적용
- Web Canvas: `fill_text(display_caption)` 적용
- Skia: `draw_str(display_caption)` 적용

PushButton의 name fallback은 caption이 비어 있을 때만 사용되며, name에는 이번 caption 표시 규칙을 적용하지 않는다. CheckBox와 RadioButton은 caption이 있을 때만 helper를 거친다.

판정: 변경 범위가 좁고 의도와 일치한다.

### 7.4 테스트 적합성

신규 테스트 `tests/issue_1562_hwpx_form_caption_display.rs`는 `samples/hwpx/form-002.hwpx` page 0을 SVG로 렌더링한 뒤 다음을 직접 확인한다.

- `IP R&amp;D연계` 포함
- `R&amp;D 자율성트랙(일반)` 포함
- `R&amp;D 자율성트랙(지정)` 포함
- `R&amp;&amp;D` 미포함

기존 #1534 회귀 테스트도 함께 통과해 저장 XML의 double escape 방지와 표시 계층 보정이 분리되어 있음을 확인했다.

판정: 이번 변경 범위에 맞는 직접 회귀 테스트다.

## 8. 검증 결과

### 8.1 GitHub Actions

최신 head SHA `ceb736b30f5482b66a52b68aabcec357cdd8a6dd` 기준 GitHub check rollup:

| Check | 상태 |
|-------|------|
| CI preflight | SUCCESS |
| CodeQL preflight | SUCCESS |
| Render Diff preflight | SUCCESS |
| Build & Test | SUCCESS |
| CodeQL | SUCCESS |
| Analyze (javascript-typescript) | SUCCESS |
| Analyze (python) | SUCCESS |
| Analyze (rust) | SUCCESS |
| Canvas visual diff | SUCCESS |
| WASM Build | SKIPPED |

이 review 문서 커밋이 PR head에 추가되면 최신 head SHA가 다시 바뀐다. 따라서 merge 전에는 최신 PR head 기준 GitHub Actions 통과 또는 문서 전용 후속 커밋 fast-pass 조건을 다시 확인해야 한다.

### 8.2 로컬 검증

초기 검토 head `f7ec8a0aebef56c846a760e52bc41ee394997b5b` 기준 로컬 검증:

| 명령 | 결과 |
|------|------|
| `cargo test --lib renderer::form_caption` | 통과 — 3 passed |
| `cargo test --test issue_1562_hwpx_form_caption_display` | 통과 — 1 passed |
| `cargo test --test issue_1534_hwpx_form_caption_escape` | 통과 — 4 passed |
| `cargo test --test svg_snapshot form_002` | 통과 — 1 passed |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `git diff --check 8d2d78c897eef937032bb982e37e044e19e96905..HEAD` | 통과 |

conflict 해소 후 최신 head `ceb736b30f5482b66a52b68aabcec357cdd8a6dd` 기준 재확인:

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_1562_hwpx_form_caption_display` | 통과 — 1 passed |

### 8.3 시각 검증

로컬 SVG export 산출물:

```text
/private/tmp/rhwp-pr1565-review/output/poc/pr1565/form-002_001.svg
```

확인 결과:

- `IP R&amp;D연계` 포함
- `R&amp;D 자율성트랙(일반)` 포함
- `R&amp;D 자율성트랙(지정)` 포함
- `R&amp;&amp;D` 미검출

## 9. 잔여 리스크

| 리스크 | 판단 |
|--------|------|
| 단일 `&` mnemonic 처리 미지원 | 의도된 범위 밖. 한컴 실물 근거 확보 후 별도 이슈로 분리하는 편이 안전 |
| Web Canvas/Skia의 별도 pixel artifact 부재 | 동일 helper를 적용했고 GitHub Canvas visual diff가 최신 head에서 SUCCESS |
| 문서 커밋 후 head SHA 변경 | merge 전 latest checks 또는 review-doc fast-pass 조건 재확인 필요 |
| #1534 수동 close 여부 | #1536과 이 PR merge 후 작업지시자 승인 하에 판단 필요 |

현재 PR 범위에서 merge를 막는 잔여 코드 이슈는 발견하지 못했다.

## 10. 최종 권고

권고: merge 준비 가능.

단, 실제 merge 전 최종 조건은 다음을 모두 만족해야 한다.

- 최신 PR head 기준 GitHub Actions 통과 또는 문서 전용 후속 커밋 fast-pass 조건 충족
- `mydocs/pr/archives/pr_1565_review.md`가 PR diff에 포함됨
- merge 직전 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 merge 승인
- GitHub review 또는 PR comment를 남길지 작업지시자 최종 확인
- merge 후 #1562 auto-close 여부 확인
- #1534는 #1536과 #1565 반영 후 작업지시자 승인 하에 close 여부 판단

## 11. merge 후 확인 계획

merge 후에는 다음을 확인한다.

1. PR #1565 merge commit SHA와 merged timestamp 확인
2. #1562 state 확인
3. #1562가 자동 close되지 않았으면 작업지시자 승인 후 수동 close comment 작성
4. #1534 state 확인
5. #1534는 #1536과 #1565로 해결 범위가 충족됐는지 작업지시자 승인 후 close 판단
6. 필요 시 PR merge comment에 검증 결과와 #1534 처리 계획을 남김
