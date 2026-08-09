---
kind: pr-review
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4271 유지보수자 검토 — 캐럿·셀 블록 글자 서식 no-op 수정

| 항목 | 값 |
| --- | --- |
| PR | [#4271](https://github.com/edwardkim/rhwp/pull/4271) — `studio: 캐럿·셀 블록 상태의 글자 서식(색/글꼴/크기) no-op 수정` |
| 작성자 | `humdrum00001010` |
| 관련 이슈 | [#4162](https://github.com/edwardkim/rhwp/issues/4162), [#4270](https://github.com/edwardkim/rhwp/issues/4270) |
| base / head | `devel` / `humdrum00001010:task_m100_4162` |
| 검증 코드 후보 | `edc1869ea` (contributor `2a3ab45e1`, maintainer 보정 `034df2ec0`, 최신 devel merge) |
| 최종 PR head | `06d3ec7e5` (검토 기록 commit 포함) |
| 코드 변경 규모 | `edc1869ea` 기준 최신 `devel...HEAD` 10파일, +849 / -54, 7커밋 |
| merge | `d4834d737` — 2026-08-09 12:32:50 KST, `devel` 반영 확인 |
| merge 직후 issue | #4162·#4270은 기본 branch 차이로 OPEN, 승인된 후속 절차에서 수동 종료 대상 |
| 검토 브랜치 | `review/humdrum00001010-20260809` (격리 worktree) |
| GitHub review | 이전 후보 [CHANGES_REQUESTED](https://github.com/edwardkim/rhwp/pull/4271#pullrequestreview-4889311426) 뒤 최종 head [APPROVED](https://github.com/edwardkim/rhwp/pull/4271#pullrequestreview-4890430796) |

## 판정 — 메인테이너 보정·검증·merge 완료

캐럿 상태에서 선택한 글자 서식을 다음 삽입 런에 적용하고 F5 셀 블록에는 즉시 적용하는
방향은 #4162 요구와 맞는다. contributor의 pending 수명주기 보정을 보존하면서, fresh Node
WASM 검증에서 발견한 삽입 undo의 원문 서식 오염을 Rust 문단 모델에서 메인테이너가
수정했다. 최신 `devel` 충돌도 양쪽 기능을 보존해 해소했고 통합 head의 Rust 5,486건과
Studio 823건을 모두 통과했다. 최종 head `06d3ec7e5`의 Full CI·CodeQL·Render Diff도
통과한 뒤 승인 review를 게시했고, merge commit `d4834d737`로 `devel`에 반영했다.

## 메인테이너 보정 결과 (`034df2ec0`, 통합 head `edc1869ea`)

`Paragraph::delete_text_at()`에서 삭제 범위의 글자 모양 경계가 같은 위치로 모일 때,
오른쪽 원문이 남아 있으면 마지막 `CharShapeRef`를 보존하도록 고쳤다. 마지막 ref는 삭제
끝에서 이동한 오른쪽 원문의 서식이다. 문단 끝까지 삭제해 오른쪽 원문이 없으면 #3576의
기존 첫 ref 보존 계약을 유지한다. 별도의 문단 전체 스캔이나 snapshot을 추가하지 않고
기존 `char_shapes` 정리 패스에서 처리해 입력·undo hot path의 복잡도는 기존과 같은
O(글자 모양 경계 수)다.

모델 unit test는 가운데 별도 서식 런 삭제 시 오른쪽 서식 보존과 끝 삭제의 기존 계약을
각각 고정했다. Studio 행위 runner의 시나리오 2도 undo 뒤 원문 `bold`·`textColor`가
baseline으로 돌아오는지 직접 단언한다. 이로써 선행 시나리오 오염이 뒤 시나리오의
pending 누수처럼 보이던 테스트 순서 의존도 제거했다.

기능 보정 커밋 뒤 최신 `upstream/devel` `f94fe5e4f`을 별도 merge commit `edc1869ea`로
통합했다. `input-handler.ts`는 이 PR의 `applyCharShapeModsToRange`와 #4272의
`cellAxisPath`·`cellParaIndexOf`를 모두 보존했고, `mydocs/orders/20260809.md`도 #4271과
#4272/#4276 기록을 모두 유지했다. 현재 merge base가 최신 `devel`이므로 충돌은 해소됐다.

## contributor 응답 뒤 재검토 (`2a3ab45e1`)

기여자는 기존 두 요청을 반영했다. `stagePendingCharShape()`는 새 속성을 병합하기 전에
낡은 앵커를 검증하고, 머리말·꼬리말/각주 모드의 `applyCharFormat()`은 pending 예약을
차단한다. 같은 수명주기 축에서 모드 진입 직전 예약의 IME 누출과 무관한 붙여넣기가
낡은 예약을 되살리는 경로도 추가로 막았다. 변경 방향은 기존 review 요청과 맞는다.

그러나 프로젝트 표준 Docker 환경에서 fresh `pkg-node`를 만든 뒤 실행한 최신 테스트는
**817/818, 1 fail**이었다. 새 시나리오 5가 삽입 글자의 `bold`를 `false`로 기대했지만
실제 값은 `true`였다.

원인은 이번 보정의 stale pending 병합이 아니라, 같은 runner의 시나리오 2가 만든 서식
상태가 undo 뒤 원문에 남는 것이다. `InsertTextCommand.execute()`는 텍스트 삽입 뒤
`applyCharShapeModsToRange()`로 굵게·색을 적용하지만, `undo()`는 삽입 텍스트만 삭제한다.
fresh 문서에서 offset 3에 굵은 파랑 `ABC`를 삽입하고 삭제하면 원래 offset 3 이후 글자가
계속 `bold=true`, `textColor=#0000ff`로 남았다. 이후 시나리오 5가 같은 `HwpDocument`를
재사용하므로 offset 6에 삽입한 `X`도 이미 오염된 굵은 서식을 상속한다.
재검증 명령·수치와 원문 서식 조회 결과는 로컬
`output/4271/pending_undo_revalidation.md`에 기록했다.

따라서 당시 다음 보정이 필요하다고 판정했다.

1. pending 글자 모양이 적용된 `InsertTextCommand`의 undo가 삽입 전 원문 글자 모양까지
   정확히 복원하도록 한다.
2. 시나리오 2에서 undo 뒤 원래 텍스트뿐 아니라 주변 글자의 `bold`와 `textColor`도
   baseline으로 복원됐는지 직접 단언한다.
3. 상태 전이 시나리오는 fresh 문서를 사용하거나 선행 시나리오의 복원이 완전함을 먼저
   단언해, 테스트 순서에 따른 오염을 숨기지 않는다.

현재 GitHub `Frontend package gates`의 동일 테스트는 `pkg-node`가 없어 skip됐다. 최신 run
`31273643404` 로그도 `ok 440 ... # SKIP pkg-node 빌드가 없어 wasm 왕복 테스트 skip`을
기록하므로 녹색 CI는 위 실패를 반증하지 않는다.

최신 `upstream/devel`은 PR merge base 뒤 17커밋 전진했다. merge simulation은
`mydocs/orders/20260809.md`와 `rhwp-studio/src/engine/input-handler.ts`에서 충돌했다. 코드 충돌은
`command.ts` import 목록에서 #4272의 `cellAxisPath`·`cellParaIndexOf`와 이 PR의
`applyCharShapeModsToRange`를 함께 보존해야 하는 형태다. 위 기능 보정 뒤 contributor branch를
최신 `devel` 기준으로 갱신하고 두 변경을 모두 보존한 새 head가 필요하다.

## 변경 범위와 렌더 영향

- `InputHandler`에 캐럿 대기 글자 모양과 앵커 위치를 추가했다.
- 일반 입력은 `InsertTextCommand`에 예약 서식을 전달하고, IME raw 입력은 삽입 범위에
  직접 서식을 적용한다.
- F5 셀 블록과 실제 텍스트 선택은 즉시 서식 적용 경로를 유지한다.
- `deactivate()`와 `dispose()`에서 예약 상태를 지워 #4270 문서 전환 누수를 막는다.
- 메인테이너 보정으로 Rust 문단 삭제 경계가 추가됐지만 Canvas paint, renderer, fixture,
  sample과 정적 렌더 결과는 바꾸지 않는다. 편집 상태 복원은 실제 Node WASM 왕복으로
  검증했고 별도 정적 시각 sweep은 적용하지 않았다.

## 이전 후보에서 보정을 요청한 결함 (`46df7071a`)

아래 두 결함은 최초 `CHANGES_REQUESTED`의 근거이며, 최신 `2a3ab45e1`에서는 해당 방향으로
수정됐다. 위 재검토 절의 undo 서식 복원 실패와 최신 `devel` 충돌도 당시 병합 차단
사유였으나, 현재는 메인테이너 보정과 최신 `devel` 통합으로 해소됐다.

### 1. 커서 이동 뒤 새 서식이 이전 위치의 pending 속성을 되살린다

`stagePendingCharShape()`는 현재 위치가 기존 앵커와 같은지 확인하지 않고 기존
`pendingCharShape`를 펼친 뒤 새 속성을 병합하고, 앵커를 현재 위치로 덮어쓴다.
`getPendingCharShape()`의 지연 무효화는 다음 입력 때만 실행되므로 아래 순서에서는 늦다.

1. 위치 A에서 굵게를 예약한다: `{ bold: true }`.
2. 위치 B로 커서를 이동한다.
3. 입력 전에 글자색을 고른다.
4. 기존 굵게와 새 색이 합쳐지고 앵커가 B로 갱신된다.
5. B에서 입력한 글자에 요청하지 않은 굵게까지 적용된다.

예약 병합 전에 기존 앵커를 검증해 이동으로 낡은 상태를 버리고, `A에서 굵게 → B로 이동
→ 색 지정 → 입력` 시 B의 새 글자에는 색만 적용되는 회귀 테스트를 추가해야 한다.

### 2. 머리말·꼬리말/각주의 서식 조작이 본문 pending으로 누출된다

머리말·꼬리말과 각주 모드에서 `cursor.getPosition()`은 진입 전 본문 위치를 유지한다.
변경된 `format-char` listener는 모드 구분 없이 `applyCharFormat()`을 호출하므로, 이들
모드에서 서식바를 조작하면 그 본문 위치를 앵커로 pending이 예약된다. 반면 해당 모드의
입력은 `InsertTextInHeaderFooterCommand`/`InsertTextInFootnoteCommand` 전용 분기에서
끝나 pending을 소비하지 않는다. 편집 모드를 나가면 저장된 본문 위치가 복원되어 다음
본문 입력에 예약 서식이 적용될 수 있다.

해당 모드의 캐럿 서식을 정식 지원해 전용 입력 경로에서 소비하거나, 지원 범위 밖이면
pending을 만들지 않도록 명시적으로 차단해야 한다. 어느 쪽이든 `머리말/각주에서 서식
조작 → 본문 복귀 → 입력`에 서식이 새지 않는 회귀 테스트가 필요하다.

### 로컬 Node WASM 상태 전이 시뮬레이션

후보의 `stagePendingCharShape()` 구현 형태를 먼저 확인한 최소 상태 harness에 실제
`InsertTextCommand`와 fresh `pkg-node` 문서를 연결했다. 첫 경로에서는 B에 삽입한 `X`가
새로 지정한 빨강뿐 아니라 A의 굵게까지 받았고(`staleBoldLeaked: true`), 두 번째
경로에서는 본문 복귀 뒤 삽입한 `Y`가 머리말·각주 상태에서 예약된 초록을 받았다
(`bodyFormatLeaked: true`). 상세 결과는 로컬
`output/4271/pending_input_handler_simulation.md`에 기록했다.

## 이전 후보 메인테이너 WASM 보완 검증 (`46df7071a`)

`pending-char-shape.test.ts`의 유일한 실제 문서 왕복 테스트는 `pkg-node/rhwp.js`가
없으면 skip한다. 현재 PR의 Frontend package gates에서는 다음처럼 skip되었다.

```text
ok 440 - 굵게/색/캐럿 대기 서식이 실제 문서에 반영된다 ...
# SKIP pkg-node 빌드가 없어 wasm 왕복 테스트 skip
```

메인테이너 환경에 CI와 같은 `wasm-pack 0.15.0`을 설치하고 후보 head에서 nodejs 대상
WASM을 새로 빌드한 뒤 집중 테스트를 다시 실행했다.

```text
CARGO_INCREMENTAL=0 wasm-pack build --target nodejs --out-dir pkg-node
node --test rhwp-studio/tests/pending-char-shape.test.ts
5 pass / 0 fail / 0 skip
```

따라서 CI skip은 이 PR의 병합 차단 사유로 사용하지 않는다. 다만 실제 WASM 테스트는
커맨드 계층을 검증하고, 나머지 네 건은 소스 문자열 계약 테스트이므로 위 두
`InputHandler` 상태 전이 결함은 여전히 검출하지 못한다.

## 검증 기록

| 검증 | 결과 |
| --- | --- |
| 최종 local candidate / devel | `edc1869ea` / `f94fe5e4f`; 최신 devel을 merge parent로 포함, 7커밋 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 통과, 5,486 pass / 0 fail / 35 configured skip |
| 표준 Docker web WASM | 통과, `docker compose --env-file .env.docker run --rm wasm` |
| Docker Node WASM | 통과, `wasm-pack build --target nodejs --out-dir pkg-node` |
| web / Node WASM SHA-256 | 둘 다 `4c20ac6dd46466d727decdcab05c771f334d178217df8c2309c96a66582e3311` |
| WASM 산출물 소유권 | 둘 다 `edward:edward`, UID/GID `1002:1002` |
| `npx tsc --noEmit` | 통과 (fresh web `pkg/` 생성 뒤) |
| `node --test rhwp-studio/tests/pending-char-shape.test.ts` | 통과, 5 pass / 0 fail / 0 skip |
| `npm test` | 통과, 823 pass / 0 fail / 0 skip |
| `git diff --check` | 통과 |
| 최신 GitHub CI | 통과 — head `06d3ec7e5`, CI `31291755667`, CodeQL `31291755574`, Render Diff `31291755573` |
| 최신 `devel` 통합 | 통과, 2파일 충돌에서 양쪽 기록·helper를 보존해 `edc1869ea` 생성 |
| 승인 / merge | [APPROVED](https://github.com/edwardkim/rhwp/pull/4271#pullrequestreview-4890430796), merge commit `d4834d737` |

현재 CI workflow는 nodejs WASM artifact를 만들지 않아 실제 WASM 왕복 테스트를 skip한다.
따라서 최종 판정에는 메인테이너가 표준 Docker로 fresh web·Node WASM을 모두 만든 뒤
수행한 위 로컬 결과를 사용한다. 실패→원인→보정 전후 상세 근거는 로컬
`output/4271/pending_undo_revalidation.md`에 남긴다.

## 최종 결과

메인테이너 보정과 최신 `devel` 통합을 마친 code candidate `edc1869ea`에 검토 기록을
더한 최종 PR head는 `06d3ec7e5`다. required CI와 mergeability를 재확인하고 승인 review를
게시한 뒤 merge commit `d4834d737`로 `devel`에 반영했다. merge 직후 자동 종료되지 않은
#4162·#4270의 수동 종료와 contributor 완료 comment는 이 archive 반영 뒤 후속 절차로
분리했다.
