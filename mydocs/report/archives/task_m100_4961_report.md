# 최종 보고 — Task M100 #4961 Font Decision Trace

- **이슈**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **PR**: [#5122](https://github.com/edwardkim/rhwp/pull/5122)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **선행**: [#4939](https://github.com/edwardkim/rhwp/issues/4939) W0·W1
- **후속**: [#4962](https://github.com/edwardkim/rhwp/issues/4962) W3·W4
- **기준 commit**: `418e5b191d23cf0618ce99f0cfec332c19ac1bc2`
- **브랜치**: `local/task4961-font-decision-trace`
- **작성일**: 2026-08-17 KST

## 1. 결과

특정 문자의 폭과 glyph가 왜 그렇게 결정됐는지를 Rust layout, native Skia, Canvas2D와 CanvasKit에 걸쳐
한 개의 versioned trace로 설명할 수 있게 됐다. 이 기능은 fallback을 통합하거나 교정하는 resolver가
아니다. 기존 선택이 끝난 뒤 계보와 capability를 읽는 별도 opt-in query다.

```text
document face / language slot / altType / substFont
  → normalized layout name
  → metric alias / exact·boldOnly·nameFirst / character hit·miss
  → overlay·heuristic / ratio·letter spacing·extra spacing / final advance
  → native·Canvas2D·CanvasKit candidate, resource, certainty와 failure
  → W1 candidateId·ruleId·evidence anchor
```

같은 입력의 portable `layoutHash`와 backend snapshot을 포함한 `normalizedHash`를 분리했다. 미지원,
미관찰, source join 실패와 상한 초과는 빈 성공이나 추정 face가 아니라 안정 reason으로 끝난다.

## 2. 공개 표면

| 표면 | 호출 |
| --- | --- |
| Rust native | `DocumentCore::get_font_decision_trace_native(page, options)` |
| Rust native renderer snapshot | `SkiaLayerRenderer::get_font_decision_trace(core, page, options)` |
| WASM | `HwpDocument.getFontDecisionTrace(page, optionsJson)` |
| Embed RPC | `getFontDecisionTrace({page, limits?})` |
| npm SDK | `editor.getFontDecisionTrace(page?, {maxCharacters?})` |

v1은 page 한 개, 기본 1,024문자, hard maximum 4,096문자를 지원한다. 입력을 clamp하지 않는다.
`font-decision-trace-v1` capability를 협상하지 못하면 SDK는 요청을 보내지 않고 실패한다.

machine-readable schema와 공개 fixture authority는
[`issue-4961`](../../tech/investigations/issue-4961/README.md)에 있다. 브라우저 전송·보안 경계는
[`browser_bridge.md`](../../tech/wasm_agent_surface/browser_bridge.md), SDK 예시는
[`npm/editor/README.md`](../../../npm/editor/README.md)가 설명한다.

## 3. 구현 계보

### Stage 1 — 계약

- trace schema v1, 상한, reason/certainty와 hash 계약
- W1 candidate identity golden과 ledger join validator
- repository-tracked 공개 HWP/HWPX fixture boundary
- 민감정보·절대 경로·private corpus fail-closed 검사

### Stage 2 — Rust layout

- 기존 name resolution, metric lookup과 문자폭 사다리를 decision-returning helper로 추출
- 기존 숫자·문자열 API는 같은 decision의 projection을 계속 사용
- native/WASM bounded query와 portable/normalized hash

### Stage 3 — backend

- 준비된 native Skia renderer snapshot에서 실제 replay와 동일한 custom → system → bundled → legacy 후보·glyph 검사
- Canvas2D CSS/local/web/generic supply와 실제 glyph face 미관찰 구분
- CanvasKit SFNT/typeface plan·glyph resource와 source record join
- Embed와 `@rhwp/editor`의 별도 capability/RPC

### Stage 4 — 공개 E2E

- exact face, missing face, document substitute profile 분리
- 공개 HWP/HWPX 6개, 반복/hash/order mutation, 상한과 unsupported 자동 검증
- header/footer의 target-dependent `usize::MAX` marker를 source coordinate가 아닌
  `null/unavailable`로 정규화해 native 64-bit/wasm32 portable hash 일치

### Stage 5 — 0-delta와 전항 감사

- W1·trace 계약, 전체 Rust·Studio·editor, native Skia와 fresh Docker WASM 통과
- 공개 6문서 page 0 native/WASM SVG byte parity
- headless·Windows host Chrome에서 실제 SDK trace 호출
- trace 전후 SVG·HWP bytes 동일, trace 중 font/network/permission trigger 0건
- FI-01~FI-14와 #4961 완료 조건 전항 disposition

### Stage 6 — self-review native snapshot 보정

- query 내부의 빈 `SkiaLayerRenderer` 임시 observer 제거
- 이미 준비된 renderer snapshot에서만 native 관측을 `complete`로 판정
- standalone `DocumentCore` native query는 `nativeRendererSnapshotRequired`로 fail-closed
- custom font inventory 보존과 standalone 거부를 native integration test로 고정

## 4. 검증 요약

| 범위 | 결과 |
| --- | --- |
| W1 ledger / trace contract / WASM E2E | 10 / 12 / 3 passed |
| release Rust lib | 4,076 passed, 13 ignored |
| release-test nextest | 6,523 passed, 38 skipped |
| native Skia 공식 3종 | 59 + 2 + 4 passed |
| Studio 전체 | 940 passed, 1 skipped |
| editor 전체 | 24 passed |
| native/WASM 공개 SVG | 6문서, mismatch 0 |
| headless/host Chrome | 양쪽 통과, 동일 0-delta 결과 |

Stage 6 보정 뒤 focused trace는 default 4건과 native Skia 6건, fresh WASM E2E 3건이 통과했다.
최신 `devel` merge 결과의 release-test nextest는 **6,542 passed, 38 skipped**, native 공식 gate는
**58 + 2 + 4 passed**였다. Studio는 **957 passed, 1 skipped**, editor는 **24 passed**였고 production
build도 통과했다.
상세 self-review 원인, 수정 경계와 명령 drift 처리는
[Stage 6 보고서](../../working/task_m100_4961_stage6.md)에 있다.

상세 명령과 환경 사실은 [Stage 5 보고서](../../working/task_m100_4961_stage5.md)에 있다.

## 5. 보호 불변식 결론

FI-01~FI-07, FI-10~FI-13은 자동·실환경 근거로 통과했다. FI-08·FI-09·FI-14는 W2의 비변경 경계를
보존했다. 즉 검증되지 않은 PDF oracle을 만들지 않았고, 저장 LineSeg와 fresh layout을 섞지 않았으며,
실제 10k 조판 위험 계측을 수행했다고 주장하지 않는다. FI-14에 필요한 ratio·자간·추가 advance 계보는
trace에 준비했고 실제 aggregate 판정은 #4962로 인계한다.

W1의 44개 `unknown`은 그대로 남는다. ledger를 runtime selection authority로 올리지 않았고, source digest
drift는 trace refactor가 발생했음을 reason으로 드러낼 뿐 선택을 바꾸지 않는다. metric DB와 fallback
target, font asset도 변경하지 않았다.

## 6. 알려진 한계

- Canvas2D actual glyph face는 브라우저가 공개하지 않아 `notObserved`다.
- 비활성 CanvasKit backend는 준비된 plan을 보이되 실제 선택을 `observed`로 승격하지 않는다.
- WASM 단독 query는 native Skia와 Studio snapshot을 `unsupported`로 표시한다.
- native build의 `DocumentCore` 단독 query도 준비된 renderer snapshot이 없으면
  `nativeRendererSnapshotRequired`로 `unsupported`를 반환한다. 이 경계는 빈 custom·bundled inventory를
  실제 관측으로 오인하지 않기 위한 fail-closed 계약이다.
- 검증된 oracle이 없는 record는 `notProvided`다.
- source coordinate가 없는 header/footer marker는 다른 문단으로 추정하지 않는다.
- v1은 page 범위와 4,096문자 상한만 제공한다.
- 현재 trace는 진단용이므로 전체 corpus에서 켜면 비용이 든다. 기본 renderer 경로에는 collector·hash
  비용이 없다.

## 7. W3·W4 인계 — #4962

#4962는 이 trace를 실제 renderer 계측 입력으로 사용한다.

1. private corpus를 로컬 읽기 전용으로 순회하고 공개 문서에는 aggregate·비식별 결과만 남긴다.
2. 선언 face가 아니라 trace record의 실제 문자 단위로 `exact-hit`, `identity-alias-hit`,
   `measured-overlay`, `metric-surrogate`, `char-miss`, `face-miss`, `heuristic`을 합산한다.
3. 전체 문자 수와 분류 합을 대사하고 같은 입력의 정규화 aggregate hash를 두 번 재현한다.
4. HWP/HWPX, stored LineSeg/fresh layout, language slot, bold·italic, 장평, 자간과 고정 frame을 교차한다.
5. miss 문자량 × 압축 노출 × 고정 frame 노출 × LineSeg 경계 민감도로 W4 위험 순위를 만든다.
6. 상위 face마다 backend 불일치, 정부·법정 서식 중요도와 exact source 가능성을 설명하고 W5 질문으로
   넘긴다.

#4962에서는 metric 값, fallback target과 font asset을 바꾸지 않는다. W4가 끝나기 전에 face를 대량
추가하지 않으며, 원문·본문·식별 파일 목록과 로컬 절대 경로를 게시하지 않는다.

## 8. 완료 판정과 다음 절차

#4961의 구현·로컬 검증·문서화 완료 조건은 충족했다. PR #5122 self-review에서 발견한 native snapshot
결함은 Stage 6에서 보정했으며, 보정 code candidate는 새 Full CI 검증 전이다. remote push와 GitHub
self-review 문서 추가는 각각 승인·CI gate 뒤 진행한다. #4961 close는 PR merge와 후속 검증 뒤 별도
판정한다.
