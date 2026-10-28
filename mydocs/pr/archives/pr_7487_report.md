# PR #7487 사전 판단 보고 — 엔진 보정과 Studio 후속 범위

## 수용 판단

현재 시각 검증 source `c741f24135d03470588440d1bb67dff3defd3024`의 빈 문단 엔진 보정은 **승인**이다. 동일 Enter 입력 9개와 npx MCP 한컴 PDF, Native/fresh WASM 전체 24쪽씩을 대조해 쪽수·문단 소유 보존을 확인했다. 증적 head `2d26931d`의 Full CI는 10,053 PASS / 0 FAIL / 50 skipped이며, 검토 head `ebedf92e`의 필수 CI와 정책 Controller attempt 2가 성공했다. 흰 페이지의 100%는 캐럿 좌표의 한컴 일치 증거로 확대하지 않는다.

사용자는 부분 해결·시각 증적을 수용하고 Approve·merge를 승인했다. 원 기여자 본문 보존 지시에 따라 대표 이미지의 게시 위치는 [기존 COMMENT 리뷰](https://github.com/edwardkim/rhwp/pull/7487#pullrequestreview-5389416480)로 대체했다. 이번 최종 판정 변경은 review/report 두 Markdown 파일만 포함한다. 새 문서 head의 required gate·정책·mergeability를 확인한 뒤 원격 승인·병합을 수행한다. Studio 캐럿·스크롤은 병합된 최신 devel에서 별도 PR로 처리하고 표 뒤 Enter는 잔여 범위로 유지하므로 #7486을 닫지 않는다.

| 계보 | SHA·역할 |
| --- | --- |
| 원 기여 | `b28130e1bcea509d9969088c7e75a0e23e4974b2`, author semanticist / @semanticist21 |
| collaborator 보정 | `45863eb2b238929c22ecc606af801b6273dc8482`, author @postmelee, 원 head를 유일 parent로 보존 |
| 검토 기준 devel | `02530b9ed567a44663edb26c65fb565c4a79f00d` |
| code 후보 merge tree | `247d8bba8f2373ad90a192deba83f37a861d9e42`, 텍스트 충돌 없음 |
| 초기 체리픽 검토 기록 | `b28130e...` → `d10a64a0c9d4f85e1cad726feb7158585aa277a9`; 로컬 backup ref만 보존, 원격 통합하지 않음 |

보정은 승인된 contributor source 직접 보정 경로(9.3.1)다. 원 commit의 author·내용·credit을 유지하고 별도 보정 commit과 기록 commit을 추가한다. 다른 작업 branch나 별도 문서 PR로 보내지 않는다.

## 완료·제한 범위

| 범위 | 상태 |
| --- | --- |
| 빈 문단 Enter 경계 owner 소실 | 로컬 수정 및 200% Enter33 / 300% Enter22 전후 회귀 PASS |
| 본문 시작 좌표·저장 재열기·90회 연속 입력 | 정식 테스트 PASS |
| 필수 lint·전체 Native·Native Skia·fresh WASM | 실제 실행 PASS; 개별 명령·횟수는 상세 리뷰에 기록 |
| Chrome 실제 입력 | 새 쪽 즉시 생성 확인; 100%/66%에서 남은 Studio 지연 재현 |
| 기존 p122 저장 문서 시각·geometry | Native/fresh WASM control gate passed, 그림 geometry delta=0 |
| 새 합성 Enter 문서의 한컴 기준 출력 | 완료: 같은 입력 9개·독립 PDF 9개; 전쪽 비교·쪽수·소유 검사. 빈 출력 픽셀 점수의 한계 명시 |
| Studio caret/scroll refresh | 별도 후속 수정 범위, 이번 코드에 포함하지 않음 |
| 표 뒤 Enter | 미해결 범위 유지, 이번 보정의 해결 주장에 포함하지 않음 |
| 원격 push·새 head CI | 한컴 증적 `2d26931d` 정상 push; Full CI 실제 10,053 PASS / 0 FAIL / 50 skipped, required Build & Test·CodeQL·Render Diff·CI Impact Policy 통과 |
| review·merge·PR 본문 갱신 | c741f241의 본문·COMMENT 게시 완료. 새 증적과 후속 기록은 진행 승인 범위; Approve·merge는 별도 |

## 게시·병합 준비 순서

1. 두 commit의 실제 diff·검증·리뷰 문서를 제시하고 source push 승인받음(완료).
2. PR head·contributor ref·maintainerCanModify를 재확인하고 승인 범위 정상 push(완료).
3. 기존 `38c0af21` Full CI와 contributor 병합·문서 tail의 재사용 기록을 보존했다. 새 9개 HWPX·9개 PDF 등 증적 `2d26931d`를 정상 push하고 최신 base `4a7cf61c`와의 실제 merge checkout `5d926c85`에서 Full CI 10,053 PASS / 50 skipped 및 관련 check를 확인했다(완료). 서로 다른 CI 구성의 수치를 혼용하지 않는다.
4. 후속 review 문서·provenance·오늘할일만 single-parent로 push하고 정확한 새 head의 candidate identity·required gate·mergeability를 확인한다. 승인된 본문 갱신은 새 head 고정 raw URL로 이미지를 표시한다.
5. 새 COMMENT 재검토를 게시하고 실제 GitHub 본문/이미지/Unicode를 확인한다. 작업지시자의 시각·빈 문단 부분 범위 확인과 Approve 게시 승인은 별도다.
6. merge는 별도 승인 뒤 진행하고 최종 merge SHA·실제 시각·CI 증적을 후속 comment에 남긴다.

이 문서는 사전 판단이다. 미래 merge SHA·merge 시각·이슈 종료를 완료 사실로 적지 않는다. [#7486](https://github.com/edwardkim/rhwp/issues/7486)은 부분 해결이므로 OPEN을 유지한다.

## Contributor review 문안 초안

아래 인용은 기존 COMMENT 리뷰의 당시 초안이며 현재 판단으로 재사용하지 않는다. 실제 게시본은 상세 리뷰의 링크로 확인한다. 당시 미확보였던 한컴 기준 출력과 새 source 재출력은 아래 후속 검증으로 보완했고, 최종 Approve는 작업지시자 확인과 별도 게시 승인 전까지 제출하지 않는다.

> Enter로 넘친 빈 문단의 끝 쪽을 보존하는 변경을 확인했습니다. 추가 경계 검증에서 줄간격 200%의 33번째 Enter와 300%의 22번째 Enter는 앞단의 빈 문단 흡수에서 쪽 소유를 잃는 것을 확인하여, 승인된 범위에서 같은 저장 줄 overflow 판별을 앞단에도 공유하는 보정 commit `45863eb2`를 별도로 추가하고 기록과 함께 head `38c0af21`로 정상 push했습니다.
>
> 보정 후 두 경계에서 새 쪽과 문단 소유가 즉시 생기고, 본문 시작 좌표 및 HWPX 저장·재열기 검사가 통과했습니다. 전체 Native 회귀 10,273개, 필수 Clippy 세 경로, Native Skia 및 fresh WASM도 통과했습니다. 이후 새 head의 CI 회귀 10,087개와 필수 lint·Native Skia·CodeQL·Render Diff·CI Impact Policy가 통과했습니다. 실제 Chrome 입력에서 새 쪽 생성은 확인했으며, 별도로 캐럿·스크롤 갱신이 늦는 기존 Studio 문제가 남아 있는 것도 확인했습니다.
>
> 이번 범위는 빈 문단의 엔진 쪽 소유 보존입니다. Studio 갱신과 표 뒤 Enter까지 해결된 것으로 보고하지 않으며 #7486은 계속 열어 두겠습니다. 원 기여와 추가 보정, 같은 source SHA의 검증 및 남은 한컴 대조 미검증을 리뷰 기록에 구분했습니다.

> 동일 합성 Enter 입력의 한컴 기준 출력은 아직 미검증입니다. CI 통과와 기존 p122 대조군만으로 이 범위를 충족으로 바꾸지 않으며, 현재는 머지 보류로 기록합니다.

> 기여자님의 devel 병합 head `f2f96733`의 CI 게이트도 확인했습니다. 해당 CI는 앞서 녹색인 candidate `38c0af21`과 문서 경로의 병합 충돌 해소를 검증하여 재사용한 결과입니다. 스크린샷은 보정 당시 source의 기록으로 구분했고, 갱신한 기준선의 직접 시각 재검증은 아직 수행하지 않았습니다.

## 메인테이너 요구 대응과 새 재검토 문안

- 문서 충돌: contributor `f2f96733...`에서 양쪽 기록을 보존해 해결했고 최신 base `4a7cf61c...`에서도 증적 후보 merge tree가 충돌 없이 생성됐다.
- 동일 입력 한컴 PDF: `samples/issue7486/`와 `pdf/pr7487-spacing*-2020.pdf`, 9개 입력·9개 PDF·24쪽/backend의 새 직접 비교로 보완했다. npx client 0.9.0, profile 2020, Hancom 11.0.0.9136, input preprocessing none, client/server/local hash 일치를 확인했다.
- 새 head CI: 증적 `2d26931d`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/36976613881)를 실제 실행해 10,053 PASS / 0 FAIL / 50 skipped와 관련 check를 확인했다. 입력/PDF 추가를 review-only로 가정하지 않았다. 후속 문서 head·본문은 게시 단계에서 별도로 확인한다. Render Diff 보고용 경고 4건의 수치 변화와 미확정 원인도 상세 리뷰에 남겼다.
- 작업지시자: 대표 PNG와 빈 출력의 의미·Studio/표 뒤 Enter 잔여 범위를 확인한 뒤 별도 Approve 게시 판단. merge는 그 다음 별도 승인 단계다.

새 COMMENT 문안:

> 동일 Enter 입력의 한컴 기준 PDF 요구를 보완했습니다. 기여해 주신 빈 쪽 보존 변경과 별도 보정을 유지한 source `c741f241`에서 실제 편집 API로 생성한 HWPX 9개를 수정 없이 npx MCP로 한컴 PDF로 변환했습니다. 200% Enter33·300% Enter22는 한컴·Native·fresh WASM 모두 2쪽이고, 나머지 입력도 전체 쪽수가 일치합니다. 두 backend의 전체 24쪽씩과 문단 누락·중복 및 소유를 확인했습니다. 빈 출력의 자동 100%는 캐럿 좌표 정확성으로 확대하지 않습니다. 현재 source의 p122 대조군도 새 review/overlay로 직접 확인했습니다.
>
> 오늘할일 충돌은 기여자님의 devel 병합에서 양쪽을 보존해 해결됐고 최신 base와의 후보 병합도 충돌 없이 확인했습니다. 새 입력·기준 PDF·대표 PNG·전쪽 지표를 포함해 재검토를 요청드립니다. Studio 캐럿·스크롤은 별도 PR로 분리하고 표 뒤 Enter는 잔여 범위로 유지하므로 #7486은 닫지 않습니다. 증적 head의 새 Full CI 10,053 PASS / 50 skipped 및 관련 check를 확인했습니다. 후속 문서 head gate와 작업지시자의 시각·부분 범위를 확인한 뒤 Approve·merge를 각각 별도로 진행하겠습니다.

## Studio 별도 PR 계획

#7487 병합 후 최신 devel을 기준으로 Enter의 비동기 페이지 배치 완료와 caret reveal을 연결한다. 100%·66% 배율의 실제 브라우저에서 추가 입력·배율 변경 없이 DOM 캐럿과 viewport가 새 쪽에 맞는지 검증하고 쪽/단 나누기·undo/redo 대조군을 유지한다. 이번 source와 증적 commit에는 Studio 구현을 넣지 않았으며 별도 PR 생성·병합 완료로 기록하지 않는다.
