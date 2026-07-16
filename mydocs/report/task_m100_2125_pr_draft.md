# Task M100 #2125 GitHub 게시 초안

- 작성일: 2026-07-13, 최신 통합 갱신 2026-07-15
- 상태: PR #2254 Ready, 최신 `devel` 로컬 통합·재검증 완료, commit/push 및 GitHub 갱신 승인 대기
- 원칙: 이 문서는 local draft다. 승인 전 commit/push, PR/issue comment와 body 편집을 수행하지 않는다.

## 1. 권장 게시 순서

1. 최신 `upstream/devel` fetch와 branch behind/conflict 확인
2. Stage 5 commit push
3. 아래 본문으로 draft PR 생성
4. PR 번호를 넣어 #2125 산출물 체크리스트 갱신
5. 아래 maintainer review 요청 코멘트 게시
6. CI와 초기 review 반영
7. 작업지시자 승인 후 Ready for review 전환
8. 최종 review/CI 뒤 merge는 작업지시자가 결정
9. merge 후 #2125 완료 근거 코멘트 게시
10. 작업지시자 승인 후 #2125 close
11. #2022 Phase A 체크와 다음 Phase B 재평가 상태 갱신
12. 별도 승인 후 legacy `/web`, Safari build reliability 후속 이슈 생성

PR #2254가 이미 생성된 현재는 1~6을 완료했다. 아래 10절의 최신 `devel` 통합 결과를 PR 본문에
반영하고 진행 코멘트로 게시한 뒤 CI와 review를 다시 받는다.

## 2. PR 제목 초안

```text
[프론트] assets/fonts를 canonical font root로 이전
```

## 3. PR 본문 초안

```md
## 요약

- `web/fonts`가 소유하던 WOFF2 36개와 관련 license/inventory를 공통 `assets/fonts`로 byte-preserving 이전했습니다.
- Studio, Chrome, Firefox, Safari, VS Code의 source/copy 계약을 canonical root 기준으로 정리했습니다.
- runtime `fonts/...` URL, VS Code 11개 subset, extension 보안 정책과 `@rhwp/editor` public contract는 유지했습니다.
- `assets/fonts/**` 변경이 frontend package CI와 Render Diff를 실행하도록 변경 감지를 갱신했습니다.

Refs #2125
Related #2022, #2124, #2190

## canonical과 소비자 계약

| 소비자 | source | 유지한 runtime/distribution 계약 |
|--------|--------|-----------------------------------|
| Studio | `assets/fonts` | `public/fonts` link, `fonts/...` URL |
| legacy `/web` | `assets/fonts` | `web/fonts` compatibility link, 기존 URL |
| Chrome/Firefox | canonical 36개 전체 | `dist/fonts`, 기존 WAR/CSP |
| Safari | Chrome dist 상속 | `dist/fonts`, 최종 extension Resources 36개 |
| VS Code | canonical의 승인된 11개 | `dist/media/fonts`, 기존 webview URI/CSP |
| `@rhwp/editor` | font 미번들 | iframe target이 font 제공, dependency 0 |

현재 canonical inventory는 WOFF2 36개, 22,651,296 bytes입니다. #2190/PR #2196으로 보강된
`NotoSansKR-Regular.woff2`를 포함해 이동 전후 filename/bytes/SHA-256 mismatch는 0입니다.

## 변경 범위

- font binary, `FONTS.md`, OFL 문서를 `assets/fonts`로 rename
- Studio/legacy compatibility link와 extension/VS Code build source 갱신
- font contract, metrics, subset tool과 frontend/Render Diff 변경 감지 갱신
- canonical ownership, license, extension/npm/current font 운영 문서 갱신

다음은 변경하지 않았습니다.

- font 파일 구성·내용·fallback·runtime URL
- Rust/WASM API와 renderer 동작
- extension CSP, WAR 범위, inline script, `publicDir: false`
- `@rhwp/editor` dependency와 iframe/MessageChannel contract
- #2124 공식 metrics snapshot
- legacy `/web` app 삭제

## 검증

- fresh Docker WASM release build + `wasm-opt`: PASS
- WASM binding/editor embed: 3 PASS
- `@rhwp/editor`: 15 PASS, runtime/peer dependency 0
- service worker: 88 PASS
- Studio: unit 230 PASS, production build, CanvasKit font coverage, browser flow PASS
- Chrome/Firefox: build PASS, canonical WOFF2 각 36개 exact parity
- VS Code: compile PASS, 승인된 11개 exact subset/hash
- font/extension contract: 최종 7 PASS
- Safari: dist/manifest PASS, unsigned Xcode build와 최종 `.appex` 36개 parity PASS
- repository/action 정적 검증: PASS

로컬 keychain에 team certificate가 없어 Safari signed build는 실행 환경상 통과하지 못했습니다. signed
build/install은 release gate로 유지하고, 이 PR의 source compile과 resource packaging은 unsigned Xcode build와
최종 `.appex` hash parity로 확인했습니다.

## Metrics

maintainer의 최신 복잡도 교훈을 반영해 Max CC뿐 아니라 Total CC, Top 20 합, CC>25 합·개수,
CC>100 합·개수와 stable function diff를 함께 비교했습니다.

최초 `upstream/devel@e750e02f`, 최신 통합 `upstream/devel@37f5d64d` 대비 #2125의 reported functions,
Total CC, Top 20, CC>25, CC>100, Max CC와 stable function diff는 모두 0-delta입니다. #2124 공식
snapshot 대비 누적 변화는 이후 upstream 전체 변경에서 발생했으며 이 PR에 귀속하지 않았고, 공식
snapshot artifact도 변경하지 않았습니다.

## 최근 upstream font 작업

#2217/PR #2227과 #2206의 결론은 CanvasKit local font의 동적 등록에 관한 변경이며 bundled WOFF2의
canonical ownership/copy 계약과 독립적입니다. 해당 변경과 #2190 결과가 포함된 최신 base에서 전체 gate를
재검증했습니다.

## 후속 분리

- legacy `/web` 제거: current 한/영 manual, CI detector, metrics와 compatibility contract를 함께 정리하는 별도 이슈
- Safari build script: `xcodebuild | tail`이 실패를 성공으로 보고할 수 있는 pipeline exit propagation 이슈

두 항목 모두 source ownership 이전과 책임이 달라 이 PR에는 섞지 않았습니다.

## 문서

- `mydocs/tech/task_m100_2125_font_ownership.md`
- `mydocs/working/task_m100_2125_stage1.md`부터 `stage5.md`
- `mydocs/report/task_m100_2125_report.md`
```

## 4. maintainer review 요청 코멘트 초안

```md
@edwardkim Phase A #2125 구현과 local gate를 완료했습니다. 리뷰 부탁드립니다.

특히 다음 항목을 확인 부탁드립니다.

1. `assets/fonts`를 공통 canonical source로 두고 Studio/legacy는 link, extension/VS Code는 명시적 copy consumer로 둔 경계
2. `assets/fonts/**`를 frontend CI와 Render Diff 변경 감지에 포함한 범위
3. Chrome/Firefox/Safari 36개 전체 parity와 VS Code 11개 subset 유지
4. 동일 upstream 대비 Total CC, Top 20, CC>25, CC>100, max와 stable function diff 0-delta 판정
5. legacy `/web` 제거와 Safari pipeline failure propagation을 별도 후속 이슈로 분리한 판단
6. Safari signed build는 release gate로 유지하고 unsigned compile과 최종 `.appex` resource parity를 PR gate로 사용한 판정

#2217/PR #2227과 #2206의 최근 local font 등록 변경도 다시 대조했으며, bundled source/copy ownership과
독립적임을 PR 본문과 최종 보고서에 명시했습니다.
```

## 5. PR 생성 후 #2125 body 체크리스트 갱신 초안

기존 본문의 다른 내용은 유지하고 `## 산출물`만 다음으로 바꾼다. `#PR_NUMBER`와 링크는 생성 후 실제 값으로
치환한다.

```md
## 산출물

- [x] `assets/fonts` canonical root ownership 문서
  - 근거: `mydocs/tech/task_m100_2125_font_ownership.md`
- [x] `web/fonts` -> `assets/fonts` 이전 계획 또는 구현 PR
  - 근거: #PR_NUMBER
- [x] target별 build/copy 계약 표
  - 근거: ownership 문서와 PR의 canonical/consumer matrix
- [x] `rhwp-studio` font 수신 경로 문서
  - 근거: Studio manual, public link와 Stage 4 runtime smoke
- [x] Chrome/Firefox/Safari extension font 수신 경로 문서
  - 근거: extension build manual과 3-browser artifact parity
- [x] VS Code webview font 수신 경로 문서
  - 근거: VS Code build manual과 11개 subset parity
- [x] npm/editor 배포 font 계약 문서
  - 근거: package README, font 미번들·runtime dependency 0 계약
- [x] `FONTS.md`, `THIRD_PARTY_LICENSES.md`, extension/npm 문서 갱신 범위 정리
  - 근거: #PR_NUMBER 문서 diff
- [x] `/web` legacy 삭제 후속 이슈 생성 여부 판단
  - 결론: current manual/CI/metrics/contract를 함께 정리하는 별도 후속 이슈가 필요함
```

PR review와 merge 전이므로 body의 `현재 상태` 또는 별도 코멘트에는 `구현 PR review 중`이라고 표시한다.
체크리스트의 `완료`는 산출물이 PR에 공개돼 리뷰 가능하다는 뜻이며 이슈 close를 뜻하지 않는다.

## 6. #2125 merge 후 완료 코멘트 초안

```md
## Phase A #2125 완료 결과

PR #PR_NUMBER가 `devel`에 merge되어 `assets/fonts` canonical 이전을 완료했습니다.

| 검토 항목 | 반영 결과 |
|-----------|-----------|
| canonical ownership | 36개 WOFF2와 license/inventory를 `assets/fonts`의 단일 tracked source로 이전 |
| Studio와 legacy URL | compatibility link로 기존 `fonts/...` URL 유지 |
| 3-browser extension | Chrome/Firefox/Safari 36개 exact parity, CSP/WAR/publicDir 의미 변화 없음 |
| VS Code와 npm/editor | VS Code 11개 subset 유지, editor font 미번들·dependency 0 유지 |
| CI와 Render Diff | 향후 `assets/fonts/**` 변경을 frontend 영향으로 감지 |
| metrics | 동일 upstream 대비 Total CC, Top 20, CC>25, CC>100, max와 stable function diff 전부 0-delta |
| 최근 font 작업 | #2190 결과 보존, #2217/#2206 runtime 등록 변경과 canonical ownership의 독립성 확인 |
| legacy `/web` | app 제거는 current manual/CI/metrics/contract와 함께 별도 후속 이슈로 분리 |
| Safari build | unsigned compile·`.appex` 36개 parity PASS, signed build는 release gate 유지 |

- merge commit: `MERGE_SHA`
- CI: `CI_URL`
- 최종 보고서: `mydocs/report/task_m100_2125_report.md`

이로써 issue body의 산출물과 완료 기준은 충족했습니다. 작업지시자 승인 후 이 이슈를 close하고, 상위
#2022에서 Phase A 완료와 Phase B 재평가 단계로 전환하겠습니다.
```

## 7. #2022 merge 후 body 갱신 초안

기존 본문에서 다음 항목만 갱신한다.

```md
## 현재 상태

- 계획 수립 이슈 #2023은 v2 승인과 #2080 병합 후 `COMPLETED`로 종료했습니다.
- Phase 0 #2124는 PR #2174 병합과 함께 완료했습니다.
- frontend package CI gate #2183은 PR #2216 병합과 trusted `devel` 실측까지 완료했습니다.
- #2186의 구현 PR #2187은 merge commit `e750e02f`로 완료했습니다.
- Phase A #2125 `assets/fonts` canonical 이전은 PR #PR_NUMBER 병합과 함께 완료했습니다.
- 다음 단계는 최신 metrics와 smoke 결과를 바탕으로 Phase B 경제성과 우선순위를 재평가하고 실행 이슈를 분리하는 것입니다.
```

추적 범위와 진행 방식의 Phase A checkbox를 `[x]`로 바꾸고 근거를 `#2125, #PR_NUMBER, MERGE_SHA`로
추가한다. Phase B checkbox는 실제 실행 이슈가 승인·생성될 때까지 `[ ]`로 유지한다.

## 8. #2022 merge 후 코멘트 초안

```md
## Phase A 완료 및 다음 판단 단계

#2125가 PR #PR_NUMBER (`MERGE_SHA`)로 `devel`에 반영되어 Phase A `assets/fonts` canonical 이전을
완료했습니다.

- canonical 36개 font의 byte/hash 보존
- Studio, Chrome, Firefox, Safari, VS Code와 npm/editor 수신 계약 명시
- frontend CI와 Render Diff의 `assets/fonts/**` 감지 반영
- 동일 upstream 대비 frontend complexity 전 항목 0-delta
- legacy `/web` 제거와 Safari build reliability는 독립 후속 이슈로 분리

다음에는 #2124 공식 baseline과 현재 smoke/metrics를 기준으로 Phase B 후보(`InputHandler`, dialog,
`WasmBridge`, `diff-engine`)의 위험·경제성을 다시 비교합니다. 이 재평가와 작업지시자 승인 전에는 대형
모듈 실행 이슈를 임의로 생성하지 않습니다.
```

## 9. legacy `/web` 후속 이슈 초안

### 제목

```text
[프론트] legacy /web 개발 앱 제거 및 current 문서·metrics 정리
```

### 본문

```md
## 목표

Phase A #2125에서 font canonical ownership이 `assets/fonts`로 이전된 뒤 남은 legacy `/web` 개발 앱과
current 문서·검증 결합을 제거합니다.

상위 추적: #2022
선행: #2125, #PR_NUMBER

## 배경

저장소 내부 production/build/package 소비자는 legacy `/web` app을 사용하지 않습니다. 다만 한국어·영어
local web server manual, frontend 영향 detector, current metrics group, font compatibility contract와
일부 provenance가 남아 있어 directory만 삭제하면 current 계약이 불일치합니다.

## 범위

- `web/`의 legacy HTML/CSS/JavaScript, Python HTTPS server, 개발 인증서 제거
- tracked generated WASM glue와 `web/fonts` compatibility link 제거
- 한국어·영어 current manual에서 legacy server 안내 제거 또는 폐기 상태 명시
- frontend 영향 detector의 `web/` 범위 재평가
- current frontend metrics의 legacy group/exclusion/link metadata 정리
- font contract에서 legacy compatibility assertion 제거
- Studio/Rust의 provenance comment와 root current 안내 정합
- Studio와 frontend package/font gate 재실행

## 보존 원칙

- Git history를 archive로 사용하며 repository에 별도 legacy 복사본을 만들지 않음
- #2124 공식 metrics/smoke snapshot과 과거 report는 당시 사실로 보존
- `assets/fonts` canonical source와 font binary를 변경하지 않음
- Studio runtime `fonts/...` URL과 fallback을 변경하지 않음
- 기능 추가나 Phase B hotspot refactor를 혼합하지 않음

## 완료 기준

- current build/runtime/package에서 `/web` 직접 소비자 0
- current 한국어·영어 manual이 `/web` 실행을 지원 경로로 안내하지 않음
- current CI detector, metrics와 contract가 제거 후 구조와 일치
- #2124 historical snapshot은 변경되지 않음
- Studio build/E2E와 frontend package/font gate PASS
- 제거 또는 deprecation에 필요한 외부 공지가 있는지 명시적으로 판단
```

## 10. Safari build reliability 후속 이슈 초안

### 제목

```text
[Safari] build.sh에서 xcodebuild 실패의 pipeline exit propagation 보장
```

### 본문

```md
## 문제

`rhwp-safari/build.sh`는 `set -e`를 사용하지만 Xcode build를 `xcodebuild ... | tail -3` 형태로 실행합니다.
현재 shell에서는 앞쪽 `xcodebuild`가 실패해도 마지막 `tail`의 성공 status가 script status가 되어,
`BUILD FAILED` 뒤에 완료 문구를 출력하고 성공 exit code를 반환할 수 있습니다.

#2125 Stage 4에서 로컬 signing certificate 부재로 `xcodebuild`가 실패했을 때 이 동작을 확인했습니다.
font canonical path 변경과 무관한 기존 build reliability 결함이므로 별도 이슈로 분리합니다.

## 범위

- `set -o pipefail` 또는 명시적인 pipeline status capture로 `xcodebuild` 실패를 script 실패로 전달
- 현재 tail 중심 출력은 가능한 범위에서 유지
- 성공 경로에서는 기존 완료 문구와 exit code 0 유지
- macOS/Xcode 환경에서 의도적인 실패와 unsigned 성공 경로를 각각 검증

## 범위 제외

- signing team, certificate, provisioning 정책 변경
- Safari manifest, extension 기능 또는 font source/copy 경로 변경
- release credential 추가

## 완료 기준

- `xcodebuild` non-zero 시 script도 non-zero
- `xcodebuild` 성공 시 script 0과 완료 문구 유지
- CI 또는 재현 가능한 local test 근거 기록
```

## 11. PR #2254 최신 devel 통합 갱신 초안

### 11.1 PR 본문 변경 초안

기존 `## 검증` 목록의 Studio 항목은 다음처럼 갱신한다.

```md
- Studio: 최초 unit 230 PASS, 최신 `devel@37f5d64d` 통합 270 PASS, production build, CanvasKit font coverage, browser flow PASS
```

기존 `## Metrics`의 동일 upstream 문단은 다음으로 교체한다.

```md
최초 `upstream/devel@e750e02f`와 최신 통합 `upstream/devel@37f5d64d` 각각에 대해 #2125의
reported functions, Total CC, Top 20, CC>25, CC>100, Max CC와 stable function diff는 모두
0-delta입니다. 최신 기준선과 merge 결과는 동일 metrics script·package lock으로 비교했습니다.
#2124 공식 snapshot 대비 functions +131, Total CC +463 등 누적 변화는 이후 upstream 전체 변경에서
발생했으며 이 PR에 귀속하지 않았고, 공식 snapshot artifact도 변경하지 않았습니다.
```

`## 최근 upstream font 작업` 뒤에 다음 절을 추가한다.

```md
## 최신 devel 통합 검증

`upstream/devel@37f5d64d`를 로컬 merge했습니다. 충돌은 `mydocs/orders/20260713.md` 1건뿐이었고,
#2125 진행 상태와 upstream의 #2253/#2195 기록을 함께 보존했습니다. 제품 코드와 font binary 충돌은
없었습니다.

- fresh Docker WASM release + `wasm-opt`: PASS
- WASM binding/editor embed 3, `@rhwp/editor` 15, service worker 88, Studio unit 270 PASS
- Studio production build, Chrome/Firefox build, VS Code compile: PASS
- browser text flow와 embed transport E2E: PASS
- Chrome/Firefox/Safari font 36개, VS Code 11개 exact parity: PASS
- unsigned Xcode build와 최종 `.appex` font 36개 parity: PASS
- 최신 기준선 대비 frontend complexity 전 항목 및 stable function diff: 0-delta

fresh `rhwp_bg.wasm`은 6,842,044 bytes이며 SHA-256은
`b23ac7491ba71a8f994b6834a23c4c284c7b193c5034c1fa5a1f917a831bc9f4`입니다. signed Safari build는
로컬 team certificate 부재로 기존 release gate에 남깁니다.
```

### 11.2 PR 진행 코멘트 초안

```md
## 최신 devel 통합 및 재검증 완료

`upstream/devel@37f5d64d`를 로컬 merge하고 전체 frontend/font gate를 다시 실행했습니다.

| 항목 | 결과 |
|------|------|
| conflict | `mydocs/orders/20260713.md` 1건 해소, 양쪽 진행 기록 보존 |
| fresh WASM | release + `wasm-opt` PASS, 6,842,044 bytes |
| package tests | binding/embed 3, editor 15, service worker 88, Studio 270 PASS |
| builds | Studio, Chrome, Firefox, VS Code PASS |
| browser E2E | text flow와 embed transport PASS |
| font parity | Chrome/Firefox/Safari 36개, VS Code 11개 PASS |
| Safari | unsigned Xcode build와 최종 `.appex` 36개 parity PASS |
| metrics | 최신 기준선 대비 Total CC, Top 20, CC>25/100, Max, stable function diff 전부 0-delta |

동일 metrics script·lockfile을 사용했고, #2124 공식 snapshot 이후 누적 변화(functions +131,
Total CC +463)는 최신 upstream 변경으로 분리했습니다. merge commit과 검증 문서를 push한 뒤 CI 결과를
다시 확인하겠습니다.
```

## 12. 게시 전 치환 항목

- `#PR_NUMBER`: 생성된 PR 번호
- `MERGE_SHA`: 실제 merge commit
- `CI_URL`: 최종 성공 run 링크
- branch가 최신 `upstream/devel` 기준인지 재확인한 결과

close, 후속 이슈 생성과 #2022 Phase B 전환은 각각 작업지시자 승인을 받은 뒤 수행한다.
