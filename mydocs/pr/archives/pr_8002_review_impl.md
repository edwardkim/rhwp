# PR #8002 적용 기록

## 원 기여와 누적 순서

원 [PR #8002](https://github.com/edwardkim/rhwp/pull/8002)의 다음 5개 commit을 #8000 뒤에 순차 적용했다.
원 author `donggyun112`와 `-x` SHA를 보존했고 충돌은 없었다. 기준과 branch는 [#8000 적용](pr_8000_review_impl.md)과 같다.

| 원 commit | 적용 commit |
| --- | --- |
| `e08cc48185f6d197fb7dd669509e4a970b38b6b4` | `1797fcef9251c7513892881a987aa89ecaabbc81` |
| `e90bc5ba4b98cb78eb8599e80e50af6d044debb0` | `612f108f5f1390c79044f271f06de090e1b0ddd0` |
| `9a20880eb44d5c6161c19701263157e8585f5d07` | `4c3a4253285f488998b55f2b46729319a7636c08` |
| `eec97121feb927cb6b8dd137fe483a656da764b0` | `c03a1b4f2187bfe25bfa9bf91a43b451bb1e6cbc` |
| `5769c4e7bed9206403cc4f6c052d0ceaf6bb96ae` | `0895441b43366662163787a3448505b9459274db` |

## 근거·생산 결과·소비 지점

- `src/model/paragraph.rs`의 `normalize_paragraph_newlines`: CRLF/CR을 기존 LF로, 이미 LF인 입력은 borrowed로 유지한다.
- `src/document_core/commands/text_editing.rs`: 본문·일반/중첩 셀의 실제 변형과 응답/커서 계산 전 정규화한다.
- `src/document_core/queries/search_query.rs`: 범위·전체·첫째·n번째 치환의 삽입 길이와 실제 문자열을 맞춘다.
- `src/cli/commands/edit/document_text.rs`: dry-run/실행의 text·insertedChars 응답도 같은 정규화 결과를 쓴다.
- 공통 helper 뒤에 별도 길이 계산이 있던 Studio 명령·textarea 입력 경계를 추가로 추적했다.
  메인터너 `0cdfadcca4d7d858238120939a1a79d18ede1180`은 `normalizeTextNewlines`를
  `rhwp-studio/src/engine/command.ts`와 `input-handler-text.ts`가 공유하게 한다.
  caret·Undo 삭제 길이·서식·repaint가 실제 LF와 같은 scalar 범위를 소비한다.

독립 기대값은 원문 `앞뒤😀` 중간에 `가\r\n😀\r\n끝`을 넣으면
`앞가\n😀\n끝뒤😀`, 캐럿 6이고 Undo하면 원문으로 복귀한다는 것이다.
기존 core raw API는 CRLF/CR 6개 저장 보존 assertion FAIL, 원 Studio 명령은 fresh core에서도
14개 assertion FAIL·캐럿 8·Undo 후 `앞`만 남았다. 통합 후 같은 실제 브라우저 최종 51개 assertion은 PASS다.
정식 source `tests/cases/issue_8001_crlf_text_roundtrip.rs`에 전체 LF 문서·LineSeg·이벤트
동등성 1개를 추가해 기존 4개와 함께 검사한다. `15fae4eb76044bed952e0f8270b6ce27bbf6c837`은
실제 raw WASM·main input operation·HF/FN 입력 모형 대조군을 보강한 test commit이다.

## 기존 표 검사·시각 범위

원 기여의 #7288 합성 입력 3모듈 7검사를 실제 실행했다. 잘못된 무시 CR 표지를 제거하고
CELL 행 하나는 수용·전체 두 행은 초과하는 독립 예산 대조군과 row 소유를 보강했다.
기존 분할·원자성·overflow assertion·허용치는 유지했다. 테스트 통과 목적의 px pin/golden 갱신은 없다.
같은 LF 문단·저장본 동등성과 시각 범위 판정은 [개별 검토](pr_8002_review.md#조판-영향과-시각-범위)에 기록한다.
물리 배치 개선·기존 저장본 복구·HF/FN 직접 core API 전체·Windows 실행은 주장 범위가 아니다.

## 공통 필수 검증과 후속 조건

실제 전체 nextest **10,831 PASS / 0 FAIL / 기존 45 SKIP**, focused 15 PASS,
필수 fmt·Native/WASM·workspace Clippy와 build, Studio 2,328 PASS·타입·production build,
editor 36 PASS, 실제 CLI 8개·stdio MCP, fresh WASM Chrome 51 assertion을 확인했다.
정확한 code head·CI run과 초기 실패 이력·비해당/미검증 범위는 해당 개별 review에 연결한다.
검증한 제품 바이트는 이후 검사/문서 commit에서도 동일하다. 문서 trailing 때문에 전체 Rust·WASM을 반복하지 않는다.

통합 [PR #8004](https://github.com/edwardkim/rhwp/pull/8004)의 위 code head에서
[Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39059657983) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39059658110) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39059658256) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39059657994) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39059658020) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39059657909)가 모두 실제 완료·성공했다.
Native Skia·프런트엔드 패키지·lint·Archive A/B/C/D 회귀와 최종 Build & Test도 성공했다.
전체 check 35개 결과는 neutral 1개, skipped 4개, success 30개이며 미완료·실패 0개다.
해당 PR merge ref의 열린 CodeQL alert도 0건이었다. skipped를 실제 검사 실행 성공으로 바꾸어 적지 않았다.

다음 단계는 같은 PR의 문서 trailing 최신 CI·mergeability 확인 → 정상 merge → duration 결과·devel sync →
관련 이슈 상태/증거 댓글 → 원 PR의 개별 감사·통합 완료 close → 소유 임시 branch와 root output 전체 안전 정리다.
원격 후속 처리는 아직 완료로 단정하지 않는다. 영구 실행 영수증은 `/home/tsjang/rhwp-pr-evidence/pr-8004/`로 보존한다.
