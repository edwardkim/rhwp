# Task M100 #2125 Stage 5 완료 보고 - 문서, metrics, 종료 정리

- 이슈: #2125
- 상위 추적: #2022
- 브랜치: `task2125-assets-fonts-canonical`
- 기준 브랜치: `upstream/devel`
- 기준 commit: `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- Stage 4 commit: `15583a38`
- 완료일: 2026-07-13
- 상태: 로컬 Stage 5 완료, GitHub 게시와 push 승인 대기

## 1. Stage 5 판정

#2125가 요구한 canonical ownership, 소비자별 copy 계약, 운영 문서, CI 변경 감지와 검증 근거를 모두
연결했다. `assets/fonts`는 36개 WOFF2와 관련 license/inventory의 유일한 tracked source이며 Studio,
Chrome, Firefox, Safari, VS Code와 legacy `/web`은 source를 복제하지 않고 같은 canonical tree를 받는다.

이번 branch는 product 함수나 공개 API를 변경하지 않았다. 동일 `upstream/devel` 기준 frontend complexity
지표는 전 항목 0-delta이며 #2124 공식 snapshot은 변경하지 않았다.

로컬 구현과 검증은 완료됐지만 다음 외부 상태가 남아 있으므로 이슈를 완료 또는 close로 표시하지 않는다.

- branch push와 PR 생성
- maintainer review와 CI
- PR merge
- #2125, #2022 갱신과 #2125 close 승인

## 2. #2125 산출물 체크리스트 대조

| 이슈 산출물 | 구현 및 근거 | 판정 |
|-------------|--------------|------|
| `assets/fonts` canonical root ownership 문서 | `mydocs/tech/task_m100_2125_font_ownership.md` | 완료 |
| `web/fonts`에서 `assets/fonts` 이전 계획 또는 구현 PR | 수행/구현 계획과 local implementation, PR 초안 | 구현 완료, PR 대기 |
| target별 build/copy 계약 표 | ownership 문서와 최종 보고서의 소비자 matrix | 완료 |
| Studio font 수신 경로 문서 | `rhwp-studio/public/fonts` link와 Studio build/manual | 완료 |
| Chrome/Firefox/Safari font 수신 경로 문서 | extension build manual, build scripts, Stage 4 artifact parity | 완료 |
| VS Code webview font 수신 경로 문서 | VS Code build manual, webpack 11개 subset, Stage 4 parity | 완료 |
| npm/editor 배포 font 계약 문서 | package README와 root README, zero dependency test | 완료 |
| font/license/extension/npm 문서 갱신 범위 | `assets/fonts/FONTS.md`, `THIRD_PARTY_LICENSES.md`, 관련 운영 문서 | 완료 |
| `/web` 삭제 후속 이슈 판단 | 별도 이슈가 필요하다고 판정하고 게시 초안 작성 | 완료 |

GitHub issue body의 산출물 체크리스트는 PR 생성 후 PR 번호와 근거 링크를 넣어 갱신한다. 로컬 완료만으로
GitHub 항목을 먼저 닫지 않는다.

## 3. 최근 upstream font 작업 영향

| upstream 작업 | 현재 branch 포함 여부 | #2125 영향 판정 |
|---------------|-----------------------|-----------------|
| #2190 / PR #2196 Noto Sans KR subset 보강 | 포함 | `NotoSansKR-Regular.woff2`의 현행 562,220 bytes와 SHA-256을 migration 기준으로 보존 |
| #2217 / PR #2227 CanvasKit local font 동적 등록 | 포함 | runtime alias/등록 로직이며 bundled source root와 build copy 계약은 변경하지 않음 |
| #2206 static font metrics 등록 검토 | #2227로 해결되어 close | 별도 bundled asset 변경이 없어 canonical 이전과 충돌하지 않음 |
| #2224 CanvasKit readiness | 포함 | Stage 4 Studio gate에서 현재 runtime 기준으로 재검증 |
| #2216 frontend package CI | 포함 | `assets/fonts`가 frontend 영향 경로가 되도록 detector를 확장 |
| #2187 editor embed transport | 포함 | package/editor embed gate를 재실행했고 public contract delta 없음 |

#2217/#2227의 동적 local font 등록과 #2206의 결론은 runtime font 발견·등록 문제를 해결한다. #2125는 저장소에
포함되는 WOFF2의 source ownership과 배포 복사 계약을 다루므로 책임이 겹치지 않는다. 최신 upstream을 다시
rebase하거나 구현을 바꿔야 할 사유는 발견되지 않았다. PR 생성 직전에는 `upstream/devel`을 다시 fetch해
base drift를 한 번 더 확인한다.

## 4. legacy `/web` 조사와 후속 판정

### 4.1 현재 구성

`web/`에는 HTML/CSS/JavaScript 기반 legacy editor, Python HTTPS server, self-signed 개발 인증서, 생성된
WASM glue 파일과 `web/fonts -> ../assets/fonts` compatibility link가 남아 있다. 현재 Studio와 extension
build/package는 이 legacy app을 production source로 사용하지 않는다.

저장소 밖 소비자는 확인할 수 없으므로 바로 삭제하지 않는다. 저장소 안에서는 다음 current reference가
남아 있어 별도 정리가 필요하다.

| 분류 | 남은 결합 |
|------|-----------|
| manual | 한국어·영어 local web server manual이 legacy Python server 실행법을 안내 |
| CI | frontend 영향 detector가 `web/` 변경을 추적 |
| metrics | legacy web group, 제외 규칙과 `web/fonts` link metadata |
| contract test | font asset contract가 compatibility link를 검증 |
| provenance comment | Studio/Rust 일부 주석이 legacy 구현 기원을 언급 |
| root 안내 | Node/web editor 관련 current 설명 재확인 필요 |

따라서 `/web` 제거는 가능성이 높지만 단순 directory 삭제가 아니다. current manual, detector, metrics current
schema, compatibility contract와 provenance를 함께 정리해야 한다. #2124 역사 snapshot은 과거 사실이므로
변경하지 않는다. 이 범위는 별도 후속 이슈로 분리한다.

### 4.2 후속 이슈 경계

- legacy app, 개발 인증서, tracked generated glue와 compatibility link 제거
- 한국어·영어 current manual에서 legacy 실행 경로 제거 또는 폐기 표시
- CI detector와 current metrics group/contract 갱신
- Studio를 유일한 current web app으로 확인하는 frontend gate 실행
- font canonical source, runtime font URL, fallback, Studio 기능은 변경하지 않음

Git history가 archive 역할을 하므로 repository 안에 legacy 복사본을 새로 만들 필요는 없다.

## 5. 추가로 분리할 Safari build reliability 결함

`rhwp-safari/build.sh`는 `set -e` 상태에서 `xcodebuild | tail -3`을 실행한다. 현재 shell에서는 앞쪽
`xcodebuild` 실패가 script exit status에 전달되지 않아 signing 실패 뒤에도 완료 문구와 성공 exit code를
낼 수 있다. font path 변경과 무관한 기존 결함이므로 별도 후속 이슈 초안으로 분리했다.

수정 이슈는 pipeline failure propagation만 다루고 signing team/certificate 정책, font 경로와 extension
기능은 변경하지 않는다.

## 6. metrics 결산

maintainer의 최신 복잡도 교훈에 따라 최대 CC만 보지 않고 Total CC, Top 20 합, CC>25 합·개수,
CC>100 합·개수, max와 stable function diff를 함께 비교했다.

| 비교 | 결과 |
|------|------|
| #2124 공식 snapshot 대비 현재 devel 누적 | functions +91, Total CC +354, Top 20 0, CC>25 +2건/+62, CC>100 0, max 0 |
| 동일 `e750e02f` 대비 #2125 직접 delta | 모든 complexity 항목 0, stable function diff 0건 |
| 공식 snapshot artifact | hash 변경 0 |

공식 snapshot 이후 누적 증가는 #2125의 path-only migration에 귀속하지 않는다. direct base 비교를 함께
제시해 복잡도를 다른 함수로 옮기거나 총량을 늘린 refactor가 아님을 확인했다.

## 7. 검증 요약

- fresh Docker WASM release build와 `wasm-opt`: PASS
- binding/editor embed: 3 PASS
- `@rhwp/editor`: 15 PASS, runtime/peer dependency 0
- service worker: 88 PASS
- Studio unit 230 PASS, production build와 browser E2E PASS
- Chrome/Firefox build, extension dist contract: PASS
- VS Code compile과 11개 font subset parity: PASS
- canonical font/consumer contract: PASS
- Safari dist 36개 parity, manifest와 unsigned Xcode compile, 최종 `.appex` resource parity: PASS
- signed Safari build: 로컬 certificate 부재, 비차단 release gate
- `git diff --check`와 문서 정합: PASS

## 8. GitHub 게시 경계와 다음 순서

`mydocs/report/task_m100_2125_pr_draft.md`에 다음 게시물의 초안을 모았다.

1. PR 제목·본문
2. maintainer review 요청 코멘트
3. PR 생성 후 #2125 체크리스트 갱신안
4. merge 후 #2125 완료 코멘트와 close안
5. merge 후 #2022 Phase A 갱신안
6. legacy `/web` 제거 후속 이슈
7. Safari build pipeline 후속 이슈

작업지시자의 별도 승인 전에는 위 내용을 GitHub에 게시하지 않고 branch도 push하지 않는다.
