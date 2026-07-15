# Task M100 #2125 최종 보고서 - assets/fonts canonical 이전

- 이슈: #2125
- 상위 추적: #2022
- 브랜치: `task2125-assets-fonts-canonical`
- 최초 검증 기준: `upstream/devel` `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- 최신 통합 기준: `upstream/devel` `37f5d64d60010582c39887ccdb60341d87a527af`
- 기간: 2026-07-13 ~ 2026-07-15
- 상태: PR #2254 Ready, 최신 `devel` 로컬 통합·재검증 PASS, commit/push 승인 대기

## 1. 결과 요약

legacy `web/fonts`가 소유하던 36개 WOFF2와 관련 license/inventory를 공통 `assets/fonts`로 byte-preserving
이전했다. Studio와 legacy `/web`은 compatibility link로 runtime `fonts/...` URL을 유지하고,
Chrome/Firefox/Safari와 VS Code는 build source만 canonical root로 변경했다.

font 내용, fallback, public API, extension security policy와 `@rhwp/editor` iframe/MessageChannel 계약에는
의미 변화가 없다. 향후 `assets/fonts/**`만 변경되는 경우에도 frontend package CI와 Render Diff가 실행된다.

## 2. 구현 결과

| 영역 | 결과 |
|------|------|
| canonical source | `assets/fonts`, WOFF2 36개, 22,651,296 bytes |
| Studio | `rhwp-studio/public/fonts -> ../../assets/fonts` |
| legacy web | `web/fonts -> ../assets/fonts`, 기존 URL 보존 |
| Chrome/Firefox | canonical 36개 전체를 `dist/fonts`로 복사 |
| Safari | Chrome dist를 상속하고 최종 extension resource에 36개 포함 |
| VS Code | 기존 승인된 11개만 `dist/media/fonts`로 복사 |
| npm/editor | font 미번들, runtime/peer dependency 0, iframe target이 font 제공 |
| CI | `assets/fonts`를 frontend 영향과 Render Diff 대상에 포함 |
| 보안 | CSP, WAR, inline script와 `publicDir: false` 의미 변화 없음 |

canonical inventory에서 `NotoSansKR-Regular.woff2`는 #2190/PR #2196이 반영된 562,220 bytes,
SHA-256 `d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a`를 보존한다. 전체 36개는
이동 전후 filename, bytes, SHA-256 mismatch가 0이다.

## 3. 주요 산출물

| 문서 | 역할 |
|------|------|
| `mydocs/plans/task_m100_2125.md` | 수행 범위, guardrail, 단계와 승인 관문 |
| `mydocs/plans/task_m100_2125_impl.md` | 파일별 구현·검증 순서 |
| `mydocs/tech/task_m100_2125_font_ownership.md` | current inventory, provenance, target copy matrix |
| `assets/fonts/FONTS.md` | canonical font inventory와 생성·license 근거 |
| `mydocs/working/task_m100_2125_stage1.md` | inventory와 dependency graph |
| `mydocs/working/task_m100_2125_stage2.md` | canonical move와 source consumer 변경 |
| `mydocs/working/task_m100_2125_stage3.md` | 운영 문서 정합 |
| `mydocs/working/task_m100_2125_stage4.md` | fresh build, E2E, artifact와 metrics gate |
| `mydocs/working/task_m100_2125_stage5.md` | 종료 대조, upstream 영향과 후속 판단 |

## 4. 이슈 체크리스트 추적

| #2125 산출물 | 근거 | 상태 |
|--------------|------|------|
| canonical ownership | ownership 문서와 `assets/fonts/FONTS.md` | 완료 |
| 이전 계획 또는 구현 PR | 수행·구현 계획, PR #2254 | 리뷰 중, 최신 `devel` 동기화 대기 |
| target build/copy matrix | ownership 문서와 본 보고서 2절 | 완료 |
| Studio 수신 경로 | symlink, manual, Studio build/runtime smoke | 완료 |
| Chrome/Firefox/Safari 수신 경로 | build manual, 36개 artifact parity | 완료 |
| VS Code 수신 경로 | build manual, 11개 exact subset parity | 완료 |
| npm/editor 계약 | package README, 15 tests, dependency 0 | 완료 |
| font/license/extension/npm 문서 | canonical inventory와 current 운영 문서 | 완료 |
| `/web` 후속 판단 | 별도 legacy 제거 이슈가 필요함 | 완료 |

## 5. 검증 결과

| gate | 결과 |
|------|------|
| fresh Docker WASM release + `wasm-opt` | PASS |
| WASM binding/editor embed | 3 PASS |
| `@rhwp/editor` | 15 PASS |
| shared/Chrome/Firefox service worker | 88 PASS |
| Studio unit/build/CanvasKit font/browser flow | 최초 230 PASS, 최신 통합 270 PASS / PASS |
| Chrome/Firefox build와 dist contract | PASS, 각 36개 |
| VS Code compile와 font subset | PASS, 11개 |
| canonical font contract와 legacy URL | PASS |
| Safari dist/manifest/unsigned Xcode/`.appex` parity | PASS, 36개 |
| Safari signed build | certificate 부재, 비차단 release gate |
| extension CSP/WAR/publicDir guard | 의미 변화 0 |
| repository 정적 검증 | PASS |

Safari signed build는 설치·배포 identity를 검증하는 release gate다. #2125의 source compile과 resource
packaging은 unsigned Xcode build와 생성된 `.appex` 36개 hash parity로 검증했다.

## 6. 복잡도와 SOLID 판정

이번 작업은 asset ownership 개선이므로 SOLID 총점을 새로 부여하지 않았다. OCP/DIP 관점에서는 소비자가
legacy app 내부 폴더 대신 공통 canonical contract를 의존하도록 바뀌었지만, 이를 수치 점수 상승으로
과장하지 않는다.

maintainer의 최신 교훈에 맞춰 Total CC, Top 20 합, CC>25 합·개수, CC>100 합·개수, max와 stable
function diff를 확인했다.

| 지표 | 동일 upstream 대비 #2125 delta |
|------|-------------------------------:|
| reported functions | 0 |
| Total CC | 0 |
| Top 20 합 | 0 |
| CC>25 합·개수 | 0 / 0 |
| CC>100 합·개수 | 0 / 0 |
| Max CC | 0 |
| stable function diff | 0건 |

#2124 공식 snapshot 대비 최신 통합 시점의 누적 functions +131, Total CC +463 등은 snapshot 이후 upstream
전체 변경 결과이며 #2125 직접 delta가 아니다. 공식 snapshot artifact는 변경하지 않았다.

## 7. 최근 upstream과의 정합성

- #2190/PR #2196의 Noto Sans KR subset 결과를 migration 기준으로 보존했다.
- #2217/PR #2227과 #2206은 CanvasKit의 동적 local font 등록 문제를 다루며 bundled font source/copy와
  독립적이다.
- #2224, #2216, #2187이 포함된 최신 base에서 Stage 4 전체 frontend gate를 실행했다.

따라서 최근 font/runtime 작업 때문에 canonical 경로를 재설계하거나 현재 구현을 보류할 이유는 없다.

## 8. legacy `/web` 결론

저장소 내부 production/build 소비자는 더 이상 legacy app을 사용하지 않는다. 하지만 한국어·영어 manual,
frontend detector, current metrics group, font compatibility contract와 일부 provenance가 남아 있어 이번
PR에서 폴더만 삭제하면 문서와 검증 계약이 불일치한다.

별도 후속 이슈에서 다음을 한 번에 처리하는 것이 적합하다.

- legacy JS/HTML/CSS/Python/cert/generated glue와 `web/fonts` compatibility link 제거
- current manual, CI detector, current metrics와 contract 갱신
- #2124 historical snapshot 보존
- Studio와 frontend package gate 재검증

## 9. 후속과 잔여 위험

| 항목 | 처리 |
|------|------|
| legacy `/web` 제거 | 별도 이슈 초안 작성 |
| Safari `xcodebuild | tail` 실패 전파 | 별도 build reliability 이슈 초안 작성 |
| signed Safari build/install | release 환경에서 검증 |
| 외부의 미확인 `/web` 직접 소비자 | removal issue에서 deprecation/공지 필요성 재검토 |

## 10. 완료 상태

로컬 구현, Stage 1부터 Stage 5 검증, PR #2254 게시와 최초 CI는 완료됐다. 최신 `devel@37f5d64d`를
로컬 통합한 뒤 전체 frontend/font gate를 다시 통과했다. 다음 순서는 작업지시자 승인 후 merge commit과
검증 문서 commit을 push하고, PR 본문·진행 코멘트를 갱신해 CI와 review를 다시 받는 것이다. PR merge와
별도 close 승인 전에는 #2125를 완료 처리하지 않는다.

## 11. PR #2254 최신 devel 통합 검증 (2026-07-15)

PR 게시 후 `upstream/devel`이 `e750e02f`에서 `37f5d64d`로 진행해 로컬 merge와 전체 gate를 다시
실행했다. 충돌은 `mydocs/orders/20260713.md` 1건뿐이었으며, #2125 진행 상태와 upstream의 #2253,
#2195 관련 기록을 모두 보존해 해소했다. 제품 코드·font binary 충돌은 없었다.

### 11.1 fresh build와 package gate

| gate | 최신 통합 결과 |
|------|----------------|
| Docker WASM release + `wasm-opt` | PASS, fresh `pkg/` 생성 |
| `rhwp_bg.wasm` | 6,842,044 bytes, SHA-256 `b23ac7491ba71a8f994b6834a23c4c284c7b193c5034c1fa5a1f917a831bc9f4` |
| WASM binding/editor embed | 3 PASS |
| `@rhwp/editor` | 15 PASS |
| service worker | 88 PASS |
| Studio unit/build | 270 PASS / PASS |
| browser text flow | PASS, 입력·문단 분할·2페이지 pagination·backspace merge |
| browser embed transport | PASS, HWP/HWPX export·peer 검증·destroy·legacy diagnostics |
| Chrome/Firefox | build PASS, canonical font 각 36개 exact parity |
| VS Code | compile PASS, 승인된 11개 exact subset/hash |
| font/extension contract | 7 PASS |
| CanvasKit symbol coverage | PASS |

Safari는 dist/source overlay를 통과했고, 서명 없이 Xcode macOS extension을 fresh derived data에서
빌드했다. 최종 `.appex` Resources와 canonical font 사이에는 36개 기준 missing/extra/hash mismatch가
모두 0이다. 로컬 keychain에 `Mac Development` 인증서가 없어 signed build는 기존대로 release gate로
남긴다.

### 11.2 최신 기준선 복잡도 비교

`devel@37f5d64d` 기준선과 로컬 merge 결과에 동일한 metrics 스크립트와 package lock을 적용했다.
기준선의 migration 전 font 경로는 임시 측정 worktree에서 canonical link 구조로만 투영했으며,
font 경로는 제품 복잡도 모집단에서 제외된다.

| 지표 | 최신 기준선 | 로컬 merge | 직접 delta |
|------|------------:|-----------:|------------:|
| reported functions | 2,413 | 2,413 | 0 |
| Total CC | 12,268 | 12,268 | 0 |
| Top 20 합 | 2,581 | 2,581 | 0 |
| CC>25 개수 / 합 | 64 / 3,994 | 64 / 3,994 | 0 / 0 |
| CC>100 개수 / 합 | 6 / 1,732 | 6 / 1,732 | 0 / 0 |
| Max CC | 453 | 453 | 0 |
| stable function diff | - | - | 0건 |

도구 hash는 metrics script
`4008301769eca77bfc25233556fa8fbe3bb9b2560f3083b49db030af9d5354d3`, metrics lock
`a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd`, Studio lock
`a9992df61824d3778c206e59ad89ecd8156e2835af728752e9ffc77bee4885dc`로 일치한다. #2124 공식
snapshot은 그대로 보존했으며, 공식 snapshot 대비 누적 변화는 functions +131, Total CC +463,
Top 20 0, CC>25 +2건/+62, CC>100 0, Max 0이다.
