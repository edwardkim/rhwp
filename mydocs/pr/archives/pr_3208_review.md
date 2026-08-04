# PR #3208 검토 기록 — 서브모드 진입형 삽입·머리말/꼬리말 구조 조작 undo 기록

collaborator-mediated 외부 PR 경로(워크플로우 9장). 외부 contributor 원 코드 위에 maintainer 보정
2건과 본 검토 문서를 PR head 에 별도 커밋으로 얹었다.

## 메타

| 항목 | 내용 |
|---|---|
| PR | [#3208](https://github.com/edwardkim/rhwp/pull/3208) |
| 작성자 | `lpaiu-cs` (외부 contributor) |
| base / head 브랜치 | `devel` / `fix/track3-submode-insert-undo` |
| 관련 이슈 | [#3207](https://github.com/edwardkim/rhwp/issues/3207) (Fixes), 로드맵 [#2369](https://github.com/edwardkim/rhwp/issues/2369) Track 3 Phase A |
| maintainer_can_modify | true (9장 경로 적용 조건 충족) |
| 문서 작성 시점 참고값 | 원 PR head `5199eb8d6`, 전건 CI success, mergeable `MERGEABLE` |

`draft`/`mergeable`/`head SHA`/`CI 상태`는 3.3 절에 따라 참고값이며, 최종 merge 조건은 PR head 최신 커밋
기준 GitHub Actions 통과 + 작업지시자 승인이다.

## 관련 이슈 요약 (#3207)

기록이 옵트인(계급 1)이라 서브모드·모달로 진입하는 삽입과 머리말/꼬리말 구조 조작이 `services.wasm.*` 를
직접 호출해 히스토리를 우회했다. 각주/미주/수식 삽입과 HF 생성/삭제/마당은 문서 구조·본문 문자 수를
바꾸므로, 미기록 상태에서 이후 편집을 undo 하면 오프셋 오염·스냅샷 무언 파괴로 이어진다.

## 변경 범위 분석

### contributor 원 변경 (3파일, +174/−53)

- `rhwp-studio/src/command/commands/insert.ts` — 각주/미주를 `insertNote(services, kind)` 로 통합하고
  수식과 함께 `executeOperation({kind:'snapshot'})` 로 라우팅. 실패 시 `throw`, 커맨드 층 try/catch 제거.
- `rhwp-studio/src/command/commands/page.ts` — HF 생성(조건부 뮤테이션 분기)·삭제·마당을 snapshot 라우팅.
  감추기 2종은 세션 상태라 라우팅하지 않고 종전 호출 유지(2차 커밋에서 P1 반영해 철회).
- `rhwp-studio/tests/undo-submode-insert.test.ts` (신규) — 라우팅·원자화·설계 전제 소스 가드.

### collaborator 보정 변경 (별도 커밋, 본 검토에서 추가)

non-blocking 2건. 원 코드의 라우팅 골격과 감추기 제외 판단은 그대로 두고 두 지점만 보정했다.

1. **`applyHfTemplate` no-op 엔트리 가드** — 원 operation 은 결과를 검사하지 않아 의미상 실패
   (`ok:false`, 비-throw) 시 before==after 무변 스냅샷이 남을 수 있었다. bridge 가 `{ok}` 를 반환
   ([wasm-bridge.ts:1947](../../../rhwp-studio/src/core/wasm-bridge.ts))하므로 삽입류와 동형으로
   `if (!r.ok) throw` 추가. 소스 가드 어서션도 함께 추가.
2. **HF 삭제·마당 성공 경로 이중 `afterEdit` 제거** — `executeOperation` 이 이미 `refreshAfterOperation('full')`
   → `afterEdit` 로 리프레시하는데 뒤에서 `afterEdit` 를 한 번 더 불러 성공 경로에서 `document-changed`
   이중 emit·이중 페이지네이션 flush 가 있었다. 삭제는 중복 호출을 삭제. 마당의 뒤쪽 `afterEdit` 는 wasm
   throw 실패 경로의 유일한 리프레시라 단순 삭제하면 실패 시 stale 렌더가 되므로 `catch` 안으로 옮겨
   성공 경로 중복만 제거하고 실패 경로 리프레시는 보존했다.

## 렌더 영향 · visual sweep 판정

**visual sweep 대상 아님.** 변경은 rhwp-studio 커맨드 층의 undo 히스토리 라우팅이며 renderer/layout/
typeset/paint, WASM 렌더 출력, golden/baseline, HWP/HWPX 샘플, 기준 PDF 를 건드리지 않는다. SVG/PDF 결과가
달라지지 않으므로 2.6 절의 시각 검증 트리거에 해당하지 않는다. 사용자-visible 동작(undo/redo 시 문서 상태
복원·서브모드 이탈)은 브라우저 왕복으로 확인 대상이며, 원 PR 작성자가 실제 dispatcher 왕복으로 검증했다
(PR 본문). collaborator 보정 2건은 성공 경로 happy-path 를 바꾸지 않는다(가드는 실패 경로, afterEdit
정리는 중복 재렌더 제거).

## 핵심 주장 소스 대조 검증

| 주장 | 결과 | 근거 |
|---|---|---|
| SnapshotCommand 는 `editContext()` 미노출 → undo 가 본문 분기로 서브모드 이탈 | 확인 | [command.ts:1699](../../../rhwp-studio/src/engine/command.ts) 미구현, [input-handler.ts:2208-2217](../../../rhwp-studio/src/engine/input-handler.ts) 본문 분기가 `exitHeaderFooterMode()`/`exitFootnoteMode()` 호출 |
| `document-changed` emit·dirty 마킹 보존 | 확인 | snapshot 경로 `refreshAfterOperation(...,'full')` → `afterEdit()` 가 `document-changed` emit |
| 실패 시 throw 로 no-op 엔트리 방지 | 확인 | [history.ts:80](../../../rhwp-studio/src/engine/history.ts) 가 `command.execute()` 를 `undoStack.push`(96행) 전에 실행 → throw 시 미등록. SnapshotCommand 는 throw 시 before/after 스냅샷 discard |
| 로깅은 CommandDispatcher 위임 | 확인 | [dispatcher.ts:58-66](../../../rhwp-studio/src/command/dispatcher.ts) try/catch + `console.error` |
| 감추기 2종은 세션 상태라 제외해야 옳음 | **확인 (핵심)** | `hidden_header_footer` 는 `document` 가 아니라 엔진 구조체 필드([mod.rs:125](../../../src/document_core/mod.rs)). `save_snapshot_native` 는 `self.document.clone()` 만, `restore_snapshot_native` 는 `self.document` 만 교체 → 스냅샷이 감추기를 담지도 되돌리지도 못함. 라우팅 시 redo 스택 파기 + 예산 소모만 발생. 레포 가드 [issue_2724:154](../../../tests/issue_2724_passthrough_invalidation_guard.rs) 도 "직렬화 비대상"으로 분류 |

**form 모드 무회귀도 확인**: 직접 호출을 snapshot 으로 바꾸면 form 모드에서 snapshot 이 드롭
([input-handler.ts:3397](../../../rhwp-studio/src/engine/input-handler.ts))되지만, `insert:`/`page:` prefix 가
dispatcher 에서 선차단([dispatcher.ts:17-22](../../../rhwp-studio/src/command/dispatcher.ts))되어 execute 에
도달하지 않는다. 두 층은 의도된 defense-in-depth(dispatcher.ts:11-14 주석).

## 사전 검증

| 항목 | 결과 |
|---|---|
| `npx tsc --noEmit` | PASS (0 error) |
| `npm test` | PASS 550/550 (신규 소스 가드 4 포함, 보정 어서션 1 추가) |
| 신규 `undo-submode-insert.test.ts` 단독 | PASS 4/4 |
| `git diff --check` | PASS |

보정 커밋이 rhwp-studio 코드를 바꾸므로 후속 기록 fast-pass 대상이 아니다(9.3.1). push 후 PR head 최신
커밋 기준 heavy CI(shard·CodeQL·Render Diff) 전체 재통과를 확인한 뒤 merge 판단한다.

## 범위 외 (원 PR 이 명시)

- `insertFieldInHf`(HF 모드 **내부** 편집) — undo 시 HF 모드를 유지해야 옳아 `editContext` 역연산 커맨드가
  필요 → Track 3 Phase B 별도 PR.
- 재진입 시 쪽-탐색이 HF 를 못 찾는 사전존재 결함 → [#3206](https://github.com/edwardkim/rhwp/issues/3206)
  분리(stock `devel` 변경 0 에서 동일 재현해 본 PR 과 무관함을 확인).

## 판단

원 PR 의 핵심 주장(서브모드 이탈 무배선, 감추기 세션-상태 제외, no-op 엔트리 방지, 로깅 위임)이 소스와
테스트로 모두 확인됐다. 2차 커밋에서 P1(감추기 라우팅 철회)을 정확히 반영했다. collaborator 보정 2건
(마당 ok 가드·이중 afterEdit 정리)까지 반영해 라우팅류의 정합을 마쳤다.

**merge 수용 권고.** 최종 조건은 PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인이다.
merge 후 [#3207](https://github.com/edwardkim/rhwp/issues/3207) close 확인·기여자 감사 코멘트·브랜치 정리를
7장에 따라 수행한다.
