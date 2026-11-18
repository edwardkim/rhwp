# planet6897 PR 4개 통합 검토 증적

#9078·#9080·#9086·#9098의 고유8개 커밋을 적용하고 필요한 CJK 결측 전진·한양중고딕 원본 별칭을 보완했다. 원 contributor branch는 보존했다.

- [최종 로컬 검증](final/acceptance.json): 원본6개 Native/fresh WASM 각18쪽, base/후보36개 대응 페이지의2%p 초과 하락0. base에만 있는 추가 쪽은 원 결함으로 남기고 기준 PDF에 없는 쪽을 채점하거나 base 통과로 처리하지 않았다.
- [현재 원문 TSV·생산자](claim-evidence-index.json), [base/후보 대조](final/base-head-comparison.json), [직접 판독](final/direct-reading.json). 기준은 데스크톱 한컴2022 12.0.0.4547 Print이며 웹한글기안기 출력이 아니다.
- [전체 Rust 검증](final/rust-validation.json): 최소 글꼴13,634PASS·0FAIL·45skip, Native Skia3종4,072PASS, fmt/Clippy3종/workspace/정책 PASS. 전체 실행 head와 frozen 렌더 source는 실제 SHA대로 기록했다.
- [리베이스 보존](final/rebase-validation.json): base27ba0333, 일반 Native/WASM 읽기 경로 불변. 원 주장·신규 portable 경계30PASS, Native GPU5PASS. [Studio 단위·UI E2E](final/frontend-validation.json): 별도 단위2,805PASS와 TypeScript/dialog/autosave PASS.
- [정본 보호](final/protected-validation.json): 18문서×1,771쪽×2backend=3,542행, 하락0·상승0. 상승이 없어 기존 보호 baseline은 수정하지 않았다.
- [실제 소비42상태](final/actual/validation.json): 대표7쪽의 screen/print·Canvas2D/CanvasKit compat/default. software CK이며 root pkg/Studio/실제 초기화 응답을 확인했다. 기존 nested 글꼴 준비 fallback은 CK 통과로 쓰지 않고 공급 조건을 명시해 보완했다. backend 간 report-only 픽셀 대조28건은 동일성 문턱을 넘으므로 픽셀 parity PASS로 보고하지 않는다.
- [편집·HWP/HWPX 반복 저장](final/repeated-save.json), [독립 Type3 폭 재감사](final/source-type3-width-oracle-audit.json), [사용 입력57개 감사](final/committed-input-audit.json).
- [원본 TTF 선택](font-order/current-selection/README.md), [실제 Native 이전/이후 영수증](final/actual-h2gtrm-before-after-contract.json). 기호 하나만의 임시 폰트는 [제거](final/withdrawn-symbol-font-removal.json)했고 비공개 글꼴 바이너리를 Git에 넣지 않았다.

## source와 환경

frozen 렌더 source는 `0d587554b804183b5cc6d090f396acc47525c089`, Native SHA는 `a8f9b5c0f8f0ee7c14926a743aab2e4784ebfe57ac2e2506f3c08e5237cbce19`, WASM SHA는 `d1702a3b077f849804f63bace85f6e8c5547ca8e6c26b3bef3382beab96e166f`다. 이후 리베이스·문서 커밋과 실행 당시 SHA를 동일하다고 바꿔 쓰지 않는다. Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 검증이 아니다.

Rust 회귀는 검증된 최소 글꼴 macOS sandbox에서, 시각 출력은 원 글꼴을 공급하는 일반 Mac 환경에서 실행했다. 정식 보호 및 일반 base/후보 기준과 직접 내용 판독을 구분한다. 미세한 기존 글리프·래스터 차이를 모든 픽셀100%로 표현하지 않는다.

## 개별 검토

- [#9078](../../archives/pr_9078_review.md): 단독 저장 줄을 소유한 표 전진, 시험 결과서·보훈부 보도자료.
- [#9080](../../archives/pr_9080_review.md): 먹황새 보도자료3쪽 옆 캡션 예약·재조판.
- [#9086](../../archives/pr_9086_review.md): 시설 사용료 별표1쪽 SQUEEZE 문단.
- [#9098](../../archives/pr_9098_review.md): 신명 순명조 일반·굵게 탐침 각4쪽의 Type3 전진.

## 원격 CI와 후행 기록

[통합 PR #9140](https://github.com/edwardkim/rhwp/pull/9140)의 code head `c6996300203fec553a57b0c16b2e48329ebc083a`에서 CI·CodeQL·Render Diff·Adapter·Proptest와 trusted 정책 검사가 모두 성공했다. [exact-head CI 원장](final/code-ci.json)에 실제 실행한 job·skip·정책을 구분했다. 원격 Archive A–E는 합계 **13,465 PASS·0 FAIL·45 skip**이며 별도 macOS 전체 회귀 수와 혼동하지 않는다.

[추가 공통 보호·SVG 소비 검증](final/supplemental/README.md): 공통 4문서 Native/fresh WASM 8건은 공식 raw 100.00%, SVG 대표 4건은 98.44–100.00%다. 공통 검사 기록의 최초 제출 누락을 재대조에서 발견해 같은 코드에서 보완했으며, 최초 제출 전에 완료한 것으로 소급하지 않는다. 모든 review·overlay를 직접 판독했다.

[PR 본문 원격 asset 응답](final/published-assets-preflight.json)과 [실제 이미지 27개 표시](final/published-image-audit-code-head.json)를 code head에서 확인했다. [통합 self-review](../../archives/pr_9140_review.md)와 오늘할일은 이 녹색 code CI 뒤에 문서 전용 trailing으로 반영한다. trailing head의 실제 required check·fast-pass와 정상 merge, source PR/issue 후속 처리·정리는 별도 완료 조건이다.
