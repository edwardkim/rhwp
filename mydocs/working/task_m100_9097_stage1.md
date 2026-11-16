# #9097: frontend Canvas CI 분리

## 문제와 선행 분석

- 이슈: [#9097](https://github.com/edwardkim/rhwp/issues/9097).
- #9077의 frontend 글꼴 수정 PR #9093에서도 전체 Render Diff가 31분 02초 실행됐다.
  기존 [CI 기록](../pr/assets/issue_9077/code-ci-f68c4d9bd7.json)의 WASM 빌드는
  18분 31초, Native 빌드는 3분 49초로 합계 22분 20초였다.
- #3790의 trusted-base 분류·실패 시 전체 실행 계약을 유지한다.
- #7473의 원 해결 PR #7474, 통합 SHA `def947c2149df5bc812d801e6e79be19a403b4d1`의
  standard release, 실제 wasm-opt 실행, 브라우저 응답의 WASM hash 확인을 유지한다.
- 기존 legacy/layer 비교는 두 경로가 같은 잘못된 글꼴을 선택하는 경우를 구별하지 못한다.
  따라서 기존 #9077의 실제 setter/원시 setter 대조 검사를 CI에 연결한다.

## 구현

`frontend_canvas_required` 축을 추가했다. 알려진 Studio 화면 소스는 package 검사와 Canvas
검사를 선택하고 Native PDF를 포함한 전체 Render Diff는 선택하지 않는다. Rust renderer,
글꼴·공개 자원, package/build 설정, 알 수 없는 경로와 분류 실패는 기존 전체 경로를 유지한다.
혼합 변경은 필요한 축을 합치며 command·unit의 기존 선택 계약은 보존한다.

Canvas 축을 선택한 frontend job은 개발용 WASM 대신 standard release + wasm-opt를 한 번
빌드한다. 같은 package를 기존 frontend 검사, #9077 글꼴 검사, legacy/layer 비교, 선택
CanvasKit readiness에 재사용한다. 새 실행기는 빌드 manifest의 JS/WASM hash를 확인하며,
브라우저 검사도 실제 초기화 응답의 WASM hash를 확인한다. Native/PDF 명령은 실행하지 않는다.

정책과 소비 workflow를 v9로 함께 갱신했다. Canvas 축의 최적화 빌드·브라우저 step은 각각
정확히 하나의 completed/success 증거가 있어야 한다. 누락·중복·skip·실패는 package job이
성공해도 수용하지 않는다. 새 실행기도 CI 실행 계약으로 취급해 fail-closed 경로에 포함했다.

기존 실행 경로에 #9077 검사가 없음을 확인해 새 공용 실행기에 연결했다. 새 artifact 계약
단위 검사는 기존 frontend WASM 계약 step에 최소 호출을 추가했다. 기존 baseline·허용치와
제품 renderer·Studio 소스·글꼴은 변경하지 않았다.

## 구현 단계 검증

기준 SHA는 `1805d8f08cb3a54bccda8dfa6647432b7799f074`이며 Windows PowerShell에서 실행했다.

| 검사 | 실제 결과 |
| --- | --- |
| classifier·policy·trusted reuse·새 artifact 계약 Node 검사 | 343/343 PASS, skip 0 |
| `test_*workflow.py` 전체 | 250/250 PASS |
| actionlint 1.7.12, 변경한 workflow 3개 | PASS |
| `git diff --check`, 변경 문서 링크 검사 | PASS |
| 표준 release WASM, 실제 wasm-opt 실행 확인 | PASS, 199.41초 |
| 새 공용 browser 실행기 | PASS, 123.14초 |
| #9077 글꼴 선택 | 12개 PASS |
| legacy/layer, 기존 기본 문서 3개 각 첫 쪽 | 3개 PASS; KTX 차이 0.00617%, 나머지 0% |
| CanvasKit readiness | 8/8 PASS, 누락 0 |

WASM SHA-256은 `16e3f571c57b82d666ec87967e2d0144db5c41d7f5bef04cbd8a30986da38f54`,
JS SHA-256은 `75eaf7070854c66d31b2223558a4128d05d00dff46b51051ec943cfe250c6365`다.
브라우저가 수신한 WASM도 동일했다. Windows Chrome 157과 공유 증분 cache를 사용했으므로
이 시간을 Ubuntu CI의 개선 시간으로 비교하지 않는다. CI는 기존 Render Diff와 같은 Chromium
revision `1660786`을 고정한다.

Windows의 `public/fonts`는 Git 심볼릭 링크가 일반 파일로 체크아웃된 상태였다. 실제 브라우저
검증 중에만 기존 대상 `assets/fonts`에 junction을 연결하고 종료 시 원래 파일을 동일 hash로
복원했다. `Cafe24Ssurround-v2.0.woff2`의 연결 경로와 원본 SHA-256은 모두
`2d726dbf8863d66e6d5cf56e2532c5d14d8a4b514461b565c513c1da6ae2f41a`였다.
`assets/fonts`, `ttfs`, `public/fonts`의 Git 변경은 없다. WASM 복사로 생성된 public 선언 파일도
검증 뒤 원복했다.

실행 로그와 중간 산출물은 ignored `output/ci-canvas-scope/`와
`output/frontend-canvas-gates/`에 보관했다. 이 절은 위 고정 기준의 구현 단계 결과다.
제출 직전 갱신된 `upstream/devel`을 반영하고 최종 후보를 재검증한다.

## 후속 절차

사용자 지시에 따라 옵션 2를 적용한다. 원 CI 코드 PR의 Full CI·검토·승인·병합 후 별도
기록 PR에 PR review·오늘할일을 반영한다. 원 코드 PR에 해당 문서-only trailing commit을
추가하지 않는다. 첫 정책 변경 PR은 trusted base의 기존 분류로 전체 CI가 실행될 수 있으므로,
그 실행을 frontend-only 선택 경로의 비용 개선 실측으로 보고하지 않는다.
