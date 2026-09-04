# Task M100 #2313 GitHub 게시 초안

- 작성일: 2026-07-17
- 상태: local 구현·검증·metrics 결산 완료, push와 GitHub 게시 승인 대기
- 원칙: 승인 전 push, PR 생성, issue/PR 본문·코멘트 편집과 issue close를 수행하지 않는다.

## 1. 권장 게시 순서

1. 최신 `upstream/devel` fetch와 behind/conflict 재확인
2. Stage 5 commit push
3. 아래 본문으로 draft PR 생성
4. #2313에 PR 생성·local gate 완료 진행 코멘트 게시
5. PR에 maintainer review 요청 코멘트 게시
6. CI와 review 반영
7. 작업지시자 승인 후 Ready for review 전환
8. 최종 CI/review 뒤 merge 여부 결정
9. merge 후 #2313 완료 근거 코멘트 게시
10. 작업지시자 승인 후 #2313 close
11. #2022 body의 #2313 checkbox와 상태 갱신
12. 별도 승인 후 Phase B 실행 이슈 결정

## 2. PR 제목 초안

```text
[프론트] legacy /web 개발 앱과 current tooling 결합 제거
```

## 3. PR 본문 초안

```md
## 요약

- repository production/build/package에서 소비하지 않는 tracked `web/` 18 entries를 제거했습니다.
- frontend CI detector, metrics와 font contract에서 legacy `/web` 결합을 제거했습니다.
- 한/영 local server manual과 current font/contract guardrail을 Studio-only post-removal 상태로 현행화했습니다.
- Git history를 archive로 사용하고 compatibility stub이나 Studio 내부 legacy 복사본은 만들지 않았습니다.

Refs #2313
Related #2022, #2125, #2124

## 변경 범위

- legacy HTML/CSS/JavaScript app, Python HTTPS server, cert/key와 clipboard test 제거
- tracked generated WASM glue/declaration과 `web/fonts` compatibility link 제거
- `.github/workflows/ci.yml`의 제거된 `web/` frontend prefix 정리
- metrics의 `legacy-web` group/exclude/link metadata 정리
- font contract를 Studio canonical link 단독 검증으로 현행화
- 한국어·영어 local server manual을 Studio Vite workflow로 동기화
- font ownership과 #2023 frontend guardrail을 post-removal 상태로 갱신

다음은 변경하지 않았습니다.

- `assets/fonts` 36개 binary·license·fallback과 Studio runtime `fonts/...`
- Rust/WASM API와 renderer output
- Chrome/Firefox/Safari/VS Code 보안·font 배포 계약
- `@rhwp/editor` iframe/MessageChannel public contract와 dependency 0
- runtime UI framework 정책
- #2124 공식 snapshot과 과거 plans/reports/changelog
- Phase B hotspot 구현

## 검증

- fresh Docker WASM release build + `wasm-opt`: PASS
- WASM binding/editor embed static contract: 3/3
- `@rhwp/editor`: 15/15
- shared/Chrome/Firefox service worker: 88/88
- Studio: unit/contract 298/298, production build PASS
- Studio browser: text-flow 5 assertions, iframe embed 10/10, CanvasKit font coverage PASS
- Chrome/Firefox: build PASS, fonts 각 36개
- extension dist contract: 3/3
- VS Code: compile PASS, approved fonts 11개
- full font asset contract: 4/4
- Safari: shell/JavaScript syntax, Chrome dist inheritance, `fonts/*`와 stricter WAR static gate PASS

Safari source, manifest와 Xcode project에는 diff가 없어 signed/unsigned Xcode build는 이번 removal gate에
포함하지 않았습니다.

## Metrics

maintainer의 #1904/#2130 교훈을 반영해 Max CC뿐 아니라 Total CC, global Top 20, CC>25/100 합·개수와
stable function diff를 함께 비교했습니다.

| same-base 직접 delta | 결과 |
|----------------------|------:|
| files / reported CC functions | -10 / -149 |
| Total CC / global Top 20 | -828 / -38 |
| CC>25 개수 / 합 | -4 / -207 |
| CC>100 개수 / 합 | 0 / 0 |
| Max CC | 0 |
| non-legacy stable function diff | 0건 |

149개 function diff는 모두 삭제된 `web/` 함수입니다. Total CC 감소는 current Studio의 구조 개선이 아니라
dead legacy 모집단 삭제 결과로 분리했습니다. #2124 official 대비 current Total CC +485 등 누적 변화는
snapshot 이후 upstream 전체 변경이며 이 PR에 귀속하지 않았고, official artifact는 변경하지 않았습니다.

legacy 제외 current hotspot은 Studio에 집중되어 있습니다. `diff-engine`은 Total CC, mouse/keyboard handler는
Max CC와 함수 LOC, picture props dialog는 격리 가능성이 서로 다른 후보이므로 이 PR에서 자동으로 Phase B
구현 이슈를 만들지 않습니다.

## 문서

- `mydocs/plans/task_m100_2313.md`
- `mydocs/plans/task_m100_2313_impl.md`
- `mydocs/working/task_m100_2313_stage1.md`부터 `stage5.md`
- `mydocs/report/task_m100_2313_report.md`
```

## 4. #2313 PR 생성 후 진행 코멘트 초안

```md
## 구현 PR 및 local gate 상태

구현 PR #PR_NUMBER를 생성했습니다.

- tracked `web/` 18 entries 제거
- current production/build/package 직접 소비자 0
- current 한/영 manual, CI detector, metrics와 font contract 현행화
- fresh WASM과 Studio/extension/VS Code/npm editor package gate PASS
- Studio text-flow/embed/font browser smoke PASS
- same-base function diff 149건은 모두 legacy removal, non-legacy diff 0
- #2124 official snapshot과 historical evidence diff 0

현재 상태는 local 구현 완료, PR CI와 maintainer review 대기입니다. merge와 완료 근거 확인 전에는 이 이슈를
close하지 않습니다.
```

## 5. maintainer review 요청 코멘트 초안

```md
@edwardkim #2313 구현과 local fresh frontend gate를 완료했습니다. 리뷰 부탁드립니다.

특히 다음 항목을 확인 부탁드립니다.

1. Git history만 archive로 사용하고 `/web` compatibility stub이나 Studio 내부 복사본을 만들지 않은 판단
2. CI detector, schema v2 metrics와 font contract에서 legacy 결합만 제거한 범위
3. current manual/guardrail은 현행화하되 #2124 snapshot과 historical evidence를 보존한 경계
4. same-base Total CC -828을 current 코드 개선이 아닌 legacy 모집단 삭제로 분리하고 non-legacy diff 0을 확인한 판정
5. #1904/#2130 교훈에 따라 Total CC, Top 20, CC>25/100 합·개수, Max와 함수별 diff를 함께 사용한 산식
6. Safari는 변경 없는 Chrome dist inheritance/security static gate까지만 요구한 범위
7. Phase B 후보를 이 PR에 섞지 않고 merge 후 별도 승인으로 재분리하는 판단

fresh WASM 기준 Studio 298 tests/build, browser text-flow·embed·font coverage, Chrome/Firefox/VS Code와
font/extension contract를 모두 통과했습니다.
```

## 6. PR CI 완료 후 갱신 코멘트 틀

```md
@edwardkim PR #PR_NUMBER GitHub Actions 결과를 공유드립니다.

| 항목 | 결과 | 링크/소요 |
|------|------|-----------|
| Frontend package gates | RESULT | URL_OR_DURATION |
| Build default-feature tests | RESULT | URL_OR_DURATION |
| Native Skia tests | RESULT | URL_OR_DURATION |
| Canvas visual diff | RESULT | URL_OR_DURATION |
| CodeQL | RESULT | URL_OR_DURATION |
| Build & Test aggregate | RESULT | URL_OR_DURATION |

현재 head `HEAD_SHA`, base `BASE_SHA` 기준 mergeability와 review 상태도 함께 재확인하겠습니다.
```

CI 결과를 확인하기 전 placeholder 상태로 게시하지 않는다.

## 7. #2313 merge 후 완료 코멘트 초안

```md
## #2313 완료 결과

PR #PR_NUMBER가 `devel`에 merge되어 legacy `/web` 제거와 current 계약 정리를 완료했습니다.

| 검토 항목 | 반영 결과 |
|-----------|-----------|
| legacy tree | tracked `web/` 18 entries 제거, 별도 archive/stub 없음 |
| current consumer | production/build/package 직접 소비자 0 |
| font ownership | `assets/fonts` canonical 36개와 Studio `fonts/...` 유지 |
| tooling | CI detector, metrics schema v2와 font contract post-removal 현행화 |
| current docs | 한/영 Studio-only server manual과 guardrail 동기화 |
| historical evidence | #2124 snapshot과 과거 문서 diff 0 |
| frontend gate | fresh WASM, Studio/browser/extension/VS Code/npm/font gate PASS |
| metrics | same-base Total CC -828은 legacy deletion, non-legacy stable function diff 0 |
| Phase B | legacy 제외 current hotspot 입력만 정리, 구현은 별도 승인으로 분리 |

- merge commit: `MERGE_SHA`
- CI: `CI_URL`
- 최종 보고서: `mydocs/report/task_m100_2313_report.md`

issue의 완료 기준을 모두 충족했습니다. 작업지시자 승인 후 이 이슈를 close하고 상위 #2022의 #2313
checkbox와 Phase B 재평가 상태를 갱신하겠습니다.
```

## 8. #2022 merge 후 body 갱신 초안

기존 본문 전체를 다시 쓰지 않고 다음 지점만 수정한다.

```md
## 현재 상태

- 계획 수립 이슈 #2023은 v2 승인과 #2080 병합 후 `COMPLETED`로 종료했습니다.
- Phase 0 #2124, frontend package CI #2183과 embed #2186/#2187은 완료했습니다.
- Phase A #2125 `assets/fonts` canonical 이전은 PR #2254로 완료했습니다.
- legacy `/web` 정리는 #2313, PR #PR_NUMBER, merge `MERGE_SHA`로 완료했습니다.
- 다음 단계는 legacy 제외 최신 metrics와 smoke를 기준으로 Phase B 후보의 위험·경제성을 비교하고,
  작업지시자 승인 후 실행 이슈를 분리하는 것입니다.
```

추적 범위에서 다음 항목만 갱신한다.

```md
- [x] legacy `/web` current app·문서·metrics 정리
  - 근거: #2313, #PR_NUMBER, merge `MERGE_SHA`
- [ ] Phase B 이후 `rhwp-studio` 복잡도 해체 실행 이슈 분리
  - 조건: legacy 제외 최신 metrics/smoke의 후보별 위험·경제성 검토와 작업지시자 승인
```

`하위 이슈와 연계 작업`, `진행 방식`의 #2313 상태도 완료로 바꾸되 Phase B는 실제 실행 이슈 승인 전까지
open으로 유지한다.

## 9. #2022 merge 후 진행 코멘트 초안

```md
## legacy 정리 완료 및 Phase B 재평가 입력

#2313이 PR #PR_NUMBER (`MERGE_SHA`)로 `devel`에 반영됐습니다.

- tracked `web/` 18 entries와 current tooling/document 결합 제거
- canonical `assets/fonts`, Studio와 package/extension public·security contract 유지
- fresh frontend gate와 browser smoke PASS
- same-base Total CC -828은 legacy 함수 149건 삭제로 전부 설명, non-legacy diff 0
- #2124 official snapshot artifact 보존

legacy 제외 current 모집단은 214 files, Total CC 12,290이며 Studio가 10,751(87.5%)입니다. 후보별로는
`diff-engine.ts`가 Total CC 897/CC>25 8개, mouse/keyboard handler가 Max CC 453/444,
`picture-props-dialog.ts`가 Max CC 348입니다. 반면 `wasm-bridge.ts`는 2,334 lines지만 Max CC 12이므로 파일
크기와 함수 복잡도를 같은 순위로 보지 않습니다.

다음에는 characterization coverage, public/security 계약과 예상 감소량을 함께 비교해 한 후보를 선택합니다.
이 판단과 작업지시자 승인 전에는 Phase B 구현 이슈를 생성하지 않습니다.
```

## 10. close와 Phase B 생성 경계

- #2313 close는 PR merge, 완료 코멘트 게시와 작업지시자 승인 뒤 수행한다.
- #2022는 umbrella이므로 #2313 완료 뒤에도 open으로 유지한다.
- Phase B는 #2313 PR review에서 경계가 수용된 뒤 별도 승인으로 생성한다.
- metrics 수치만으로 `InputHandler` 또는 dialog 구현 이슈를 자동 생성하지 않는다.
