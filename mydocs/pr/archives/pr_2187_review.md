# PR #2187 리뷰 - `@rhwp/editor` MessageChannel 임베드 transport v1

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | [#2187](https://github.com/edwardkim/rhwp/pull/2187) |
| 관련 이슈 | [#2186](https://github.com/edwardkim/rhwp/issues/2186) |
| 작성자 | `cskwork` |
| base / head | `edwardkim:devel` / `cskwork:feature/embed-message-channel-v1` |
| 검토 전 head | `7ce03c01ad83f44d7b3b4099eba80c35c58f2e7e` |
| 최신 `devel` | `3c1cba9630cef6506c6c608cbfc57a099ca86ec1` |
| 로컬 검토 head | `82815ffd077932a4c3a7ac32aed53af2a24c3bbd` |
| 권한 | `maintainerCanModify=true` |
| 규모 | 최신 `devel...HEAD`: 24 files, +2,028/-217 |
| 작성 시점 원격 상태 | `CONFLICTING` / `DIRTY`; 검토 commit은 아직 contributor head에 push하지 않음 |

최종 merge 조건은 contributor head에 검토 commit과 운영 문서를 반영한 뒤 최신 head 기준 GitHub
Actions, `mergeable`, review 결과와 작업지시자 승인을 다시 확인하는 것이다.

## 처리 경로와 contributor credit

원 PR을 유지하는 collaborator-mediated Route A를 적용한다. PR diff가 과거 변경으로 오염된 것이 아니라
최신 `devel`과 `rhwp-studio/src/main.ts` 한 파일에서만 의미 충돌이 발생했으므로 별도 integration PR은
필요하지 않다.

contributor 원본 commit은 다음과 같으며 rewrite·squash하지 않는다.

| Commit | 작성자 | 내용 |
|---|---|---|
| `023041f5` | `cskwork` | MessageChannel 임베드 transport v1 |
| `c7cbcf5f` | `cskwork` | session ID secure random 제한 |
| `9713a84f` | `cskwork` | malformed envelope 명시적 실패 |
| `7ce03c01` | `cskwork` | 당시 `devel` merge 및 후속 보정 |

로컬 `82815ffd`는 `7ce03c01`에 최신 `devel@3c1cba96`을 merge한 collaborator commit이다. 첫 부모가
contributor head, 둘째 부모가 최신 `devel`이므로 contributor history와 저자 정보가 보존된다.

## 변경 범위와 계약 검토

- `npm/editor/transport.js`
  - `studioUrl`에서 HTTP(S) exact origin을 계산하고 최초 연결만 window message를 사용한다.
  - v1 연결 후 session-bound `MessagePort`로 전환하며 caller binary의 복사본만 transfer한다.
  - 일반 요청 10초, load/export 60초 timeout과 pending/timer/port/listener cleanup을 분리한다.
- `rhwp-studio/src/embed/{protocol,rpc-router,runtime}.ts`
  - parent source, 유효 HTTP(S) origin, version, capability, session과 최초 port를 검증한다.
  - 연결 뒤 legacy dispatch를 차단하고 거부·잉여 port를 닫는다.
  - 공개 RPC method만 router에서 기존 Studio handler로 전달한다.
- 호환성
  - `createEditor`, load/page/export/destroy의 이름과 반환 의미를 유지한다.
  - old SDK/new Studio와 new SDK/old Studio의 제한된 legacy request/response를 유지하되 source와 origin을
    exact 비교한다.
  - runtime dependency를 추가하지 않는다.

allowlist/JWT가 없는 exact-origin binding은 host 인증을 의미하지 않는다. 이 PR은 transport 경계와 binary,
cleanup을 다루며 배포자 인증·저장 충돌·필드 정책 등은 범위 밖으로 유지한다.

## 최신 `devel` 충돌 해소

실제 merge conflict는 `rhwp-studio/src/main.ts` 한 파일이었다. 최신 `devel`의 renderer readiness 작업이
legacy iframe API에 `getRendererDiagnostics`를 추가한 뒤 #2187이 같은 switch 전체를 새 embed runtime으로
교체해 충돌했다.

한쪽 변경만 선택하면 renderer diagnostics 또는 새 transport가 사라지므로 다음처럼 결합했다.

- `installEmbedRuntime` 구조를 유지한다.
- `getRendererDiagnostics`를 `EmbedRpcHandlers`와 `routeEmbedRequest`에 편입한다.
- page index의 비음수 정수 검증을 router에서 공통 적용한다.
- Studio handler는 기존 renderer request/initialized/error/backend/page diagnostics 결과를 그대로 반환한다.
- renderer 정적 계약, embed router unit test와 실제 legacy browser E2E를 함께 보강한다.

## 복잡도 비교

공식 metrics 도구의 최신 산식으로 clean source 두 개를 새로 측정했다.

| 지표 | `devel@3c1cba96` | 검토 head `82815ffd` | Delta |
|---|---:|---:|---:|
| reported functions | 2,340 | 2,373 | +33 |
| CC 총합 | 12,086 | 12,159 | +73 |
| 상위 20 합 | 2,581 | 2,581 | 0 |
| CC>25 개수 | 64 | 64 | 0 |
| CC>25 합 | 3,994 | 3,994 | 0 |
| CC>100 개수 | 6 | 6 | 0 |
| 최대 CC | 453 | 453 | 0 |

총량 증가는 transport/runtime/router라는 새 동작과 분리된 함수 surface에서 발생했다. 기존 고복잡도 함수와
상위 20·임계 초과 총량은 증가하지 않았다. 이 수치는 advisory 비교이며 단일 SOLID 점수나 merge fail gate로
사용하지 않는다.

## 로컬 검증

| 게이트 | 결과 |
|---|---|
| Docker `wasm-pack build --target web --dev` | PASS, fresh `pkg/` 생성 |
| WASM binding + editor embed contract | 3/3 PASS |
| `npm --prefix npm/editor test` | 15/15 PASS |
| `npm --prefix rhwp-studio test` | 230/230 PASS |
| renderer contract guard | PASS |
| shared/Chrome/Firefox service worker | 88/88 PASS |
| Studio production build | PASS |
| Chrome/Firefox build | PASS / PASS |
| extension dist contract | 3/3 PASS |
| VS Code compile | PASS |
| 실제 headless Chrome embed E2E | 9개 계약 PASS |
| `git diff --check` | PASS |

Chrome E2E는 `CHROME_PATH`와 Vite `127.0.0.1:7700`을 명시한 최종 실행에서 통과했다. public
load/export, caller buffer 보존, sibling forged peer 거부, destroy, legacy ready/pageCount와
`getRendererDiagnostics`를 실제 iframe에서 확인했다. 최초 두 실행은 Chrome 경로와 Vite 서버가 없는 환경
전제 때문에 시작 전에 중단됐으며 코드 assertion 실패는 없었다.

## 잔여 위험과 범위 제외

- 실제 브라우저의 50 MiB memory peak는 측정하지 않았다. Node MessageChannel의 50 MiB 전송과 caller buffer
  보존 계약만 확인했다.
- host allowlist/JWT, save/autosave/revision, field/policy, PDF export와 Web Component는 후속 설계 대상이다.
- Firefox/Safari 실제 browser matrix는 이 PR에 포함하지 않는다.
- latest contributor head의 authoritative 결과는 push 후 새 `Frontend package gates`와 전체 GitHub Actions다.

## PR head 반영 계획

1. merge commit `82815ffd`를 contributor head에 fast-forward push한다.
2. 이 review, 사전 판단 report와 `mydocs/orders/20260713.md`를 별도 docs commit으로 push한다.
3. contributor가 작성한 PR 본문은 수정하지 않고, collaborator 리뷰 댓글에 #2183 완료와 최신
   `devel` merge·충돌 해소·검증 결과를 기록한다.
4. #2022 본문과 우리가 작성한 최신 상태 댓글의 `rebase` 표현만 contributor history를 보존한 merge 기반
   반영으로 정정한다. 과거 #2187 댓글은 당시 계획의 역사적 기록으로 유지한다.
5. 최신 head GitHub Actions와 `Frontend package gates`, merge 가능 상태를 확인한 뒤 review를 제출한다.

## 검토 결론

blocking finding은 없다. 공개 API·무의존 계약과 제한된 legacy 호환을 유지하면서 wildcard target과 전역
응답 신뢰, number-array binary, cleanup 누락을 좁은 transport 계층으로 분리했다. 최신 `devel` 충돌도 기존
renderer diagnostics를 잃지 않는 방식으로 해결했다.

위 PR head 반영과 최신 GitHub Actions 통과 후 original PR merge를 권고한다.
