# PR #3400 검토 — 스타일 생성·수정·삭제 undo 라우팅

Issue: #3387
base route: maintainer_general
modifiers: intake_and_review, local_validation, post_merge

## 1. metadata (작성 시점 참고값, merge 전 재확인)

| 항목 | 값 |
|---|---|
| 제목 | Task #3387: [undo] 스타일 생성·수정·삭제를 히스토리에 기록 |
| author | lpaiu-cs (기존 contributor — merged 4, closed 7, open 1) |
| base | `devel` |
| head SHA | 검토 시점 `baaec70db` → update branch 후 `305230038` |
| 규모 | +173 -10, 5파일, commit 1개 |
| 연결 issue | `closes #3387` (issue OPEN) |
| mergeable | MERGEABLE / BEHIND |
| CI | SUCCESS 21, SKIPPED 1 |

reviewer: edwardkim / assignee: lpaiu-cs (작업지시자 지정)

## 2. 문제와 변경 범위

스타일 다이얼로그의 생성·수정·삭제가 편집 라우터(`InputHandler.executeOperation`)를 우회해
`Ctrl+Z` 로 되돌아가지 않았다. 삭제는 특히 전문서 효과다 — 해당 스타일을 쓰던 모든 문단이
바탕글로 바뀌고, 삭제된 ID 보다 큰 `style_id` 가 전부 밀리며 다른 스타일의 `next_style_id` 도
보정된다(`wasm_api.rs:6120-6167`). 되돌릴 방법이 없었다.

근인은 두 다이얼로그가 `(wasm, eventBus)` 로만 생성돼 라우터를 받을 길이 없었던 배선 누락이다.

| 파일 | 변경 |
|---|---|
| `src/command/commands/format.ts` | +3 -3 — 다이얼로그 3곳 생성 지점에 `services` 전달 |
| `src/ui/style-dialog.ts` | +51 -1 — `services` 수용, `history-jumped` 구독, 삭제를 snapshot 으로 기록 |
| `src/ui/style-edit-dialog.ts` | +26 -5 — 생성·수정을 모양 적용까지 한 스냅샷으로 원자화 |
| `tests/mutation-routing-guard.test.ts` | +1 -1 — 뮤테이션 원장 baseline |
| `tests/style-undo-routing.test.ts` | +92 신규 — undo 라우팅 계약 테스트 |

범위 밖 변경은 없다. Rust·renderer·fixture·CI workflow 를 건드리지 않는다.

## 3. 렌더 영향과 시각 검증 판정

**시각 검증 불필요.** [intake_and_review 2.6](../manual/pr_review/intake_and_review.md) 네 조건 중
어디에도 해당하지 않는다.

- `src/renderer`·`src/wasm_api.rs`·Canvas/render 출력 경로 무변경 (studio TypeScript 만 변경)
- typeset·layout·paint·pagination·page count·table split·wrap·clipping·margin/spacing 무관
- 기준 PDF·한컴 출력·페이지 수·render-diff 주장 없음
- HWP/HWPX sample·기준 PDF·golden·visual fixture 추가·갱신 없음

편집 명령의 히스토리 라우팅 배선 변경이며 렌더 결과 자체를 바꾸지 않는다.

## 4. 로컬 검증

`local_validation` 4.3 의 "rhwp-studio만 변경" 행에 따라 TypeScript 검사·npm test 를 수행했다.
Cargo 계열은 변경 범위 밖이라 생략했다.

검토 branch `review/lpaiu-cs-3400-20260726` (기준 `origin/devel` = `ce2156dad` 이후 최신).

| 검증 | 결과 |
|---|---|
| merge simulation (`--no-commit --no-ff`) | 충돌 0, 5파일 전부 studio 범위 |
| `npm ci` | 성공 |
| `npx tsc --noEmit` | 오류 0 |
| `npm test` | **641 pass / 0 fail** |
| PR 신규 테스트 focused | **9 pass / 0 fail** |

focused 9건에는 이슈 핵심을 검증하는 항목이 포함된다.

- 스타일 삭제는 snapshot 으로 기록된다
- 스타일 생성·수정은 모양 적용까지 한 스냅샷으로 원자화된다
- 스타일 다이얼로그는 services 를 받고 `history-jumped` 로 목록을 무효화한다
- 스타일 다이얼로그 생성 지점이 services 를 넘긴다

## 5. 코드 판단

- 삭제를 snapshot 으로 기록한 선택이 타당하다. 전문서 style_id 재배정은 역연산으로 되돌리기
  어렵고, 스냅샷 복원이 스타일 해석 캐시까지 재구성한다(`restore_snapshot_native`). 근거가
  주석으로 남아 있다.
- `history-jumped` 구독은 #2341 이 연 기존 경로를 find-dialog 와 같은 방식으로 재사용한다.
  새 이벤트 계약을 만들지 않았다.
- 생성·수정을 한 스냅샷으로 원자화해 undo 1회로 되돌아간다. 매니저형 다이얼로그라 작업마다
  개별 스냅샷을 남기는 것도 의도가 분명하다.
- `mutation-routing-guard` 원장 baseline 을 함께 갱신해 드리프트 사각을 막았다.

발견한 문제·risk 는 없다. 후속 이슈 후보도 없다.

## 6. 최종 권고

**merge 권고.** 변경이 소형이고 범위가 studio 로 한정되며, 이슈 핵심을 검증하는 신규 테스트가
포함됐다. 로컬 검증 전 항목 통과.

merge 전 조건:

- 최신 PR head 의 GitHub Actions 통과
- 작업지시자 승인
- BEHIND 상태 해소(update branch) 또는 admin merge 판단

### 처리 결과 (완료)

작업지시자가 merge 를 승인하고 update branch 로 BEHIND 를 해소했다(head `305230038`).
merge 방식은 **merge commit** 을 권고했다 — contributor authorship 이 커밋에 보존되고,
같은 contributor 의 #3323 도 merge commit(`9c26be219`)으로 처리한 선례가 있다.

| 항목 | 결과 |
|---|---|
| 최신 head CI | `305230038` 기준 SUCCESS 21 / SKIPPED 1 / 실패 0 |
| merge commit | **`91bd617583dc8b3b816e220d6168e631924e6977`** |
| devel 포함 | `git branch --contains` 로 확인 |
| issue #3387 | **auto-close 성공** (2026-07-26T10:15:43Z) |
| 후속 작업 | 없음 |

## 7. 릴리즈 연계

작업지시자 지시로 이 PR 을 v0.8.1 릴리즈(#3401) 범위에 포함한다. merge 후 릴리즈 기준선을
새 `devel` HEAD 로 재확정하고, Rust 전체 테스트를 재실행한다(기준선 변경이므로 기존 통과분을
재사용하지 않는다).
