# 확장 E2E 선택 실행 도구 결과보고

Issue: #3512, #3513, #3515 · PR: #7283

## 현재 범위 — 2026-09-27

[메인테이너 동의](https://github.com/edwardkim/rhwp/pull/7283#issuecomment-5844951296)에 따라
반복 CI 게이트에서 배포 전 선택 실행 보조 도구로 변경했다. 구현은 `fcdef5f57`, 실행 환경 기록·출력 경계 보완은 `a2df40172`다.

- 이번 PR의 workflow·CI 영향 분류·Frontend 승격·필수 집계·browser cache 변경을 모두 제거했다.
- 기존 Chrome smoke/download 검사에 후보 경로 전달을 추가하고, 설정 수명주기 10개와 진단을 유지했다.
- 실행기는 명시적 `--dist` 폴더를 검사하고 후보를 재빌드하지 않는다. 모든 suite가 같은 후보를 사용한다.
- 결과는 새 폴더에 저장한다. 후보 내용 SHA-256, 도구/브라우저 환경, suite별 pass/fail/not-run,
  실패 로그·진단을 남긴다. 후보 변경이나 부분 선택을 전체 검증 통과로 처리하지 않는다.
- Node 계약 177개, Python 정책 계약 130개 통과. 실제 후보 검증 결과는 아래와 같다.

## 검증

최종 code SHA `a2df40172`에서 실제 후보 전체 검사와 실패 대조를 실행했다.

| 검사 | 결과 |
| --- | --- |
| CI 정책 + E2E Node 계약 | 177 통과; 마지막 경계/메타데이터 보완 뒤 영향받는 E2E 계약 19개 재통과 |
| 기존 workflow/promotion Python 계약 | 130 통과 |
| 외부 후보 smoke/download/lifecycle | 모두 통과; 각각 1.778 / 14.755 / 35.261초, lifecycle 10개, retry 0 |
| 외부 후보에서 print.html만 제거 | exit 1, smoke fail, 뒤 2개 not-run, JSON/LOG/PNG 생성 |
| 후보 내용 변경 검사 | 정상/실패 실행 모두 검사 전후 동일; 별도 계약에서 검사 중 변경은 error 확인 |
| 보고서 환경 | Node 24.15.0, macOS arm64, Puppeteer 25.11.0, Chrome 153.0.8010.36 |

실제 결과에서 필요한 필드와 fixture hash를 추출한 [검증 JSON](3512-extension-e2e/opt-in-validation.json)에
검토 source와 후보 SHA-256을 기록했다. 실행 환경의 브라우저 경로는 문자열로 기록됨을 확인했다.
검증에 사용한 HWP/HWPX 3개는 `a2df40172`의 Git blob과 실행 파일이 동일함을 대조했다.

```bash
CARGO_TARGET_DIR=/Users/melee/Documents/projects/forks/rhwp/target/pr-review \
  scripts/wasm-pack-locked.sh --target web --out-dir /private/tmp/rhwp-7283-review/pkg --dev
npm --prefix rhwp-chrome run build
# dist를 /private/tmp/rhwp-7283-optin-candidate로 복사한 뒤:
npm --prefix rhwp-chrome run test:e2e -- \
  --dist /private/tmp/rhwp-7283-optin-candidate \
  --output /private/tmp/rhwp-7283-optin-e2e-final
```

이 빌드는 도구 실행 검증용 dev WASM 후보이며 실제 스토어 배포본이라고 주장하지 않는다.
실행기는 빌드를 호출하지 않는다. 실패 대조에서도 저장소의 정상 dist로 대체하지 않았고
누락된 print.html 요청이 실제 실패 원인임을 로그로 확인했다.


## 범위와 한계

확장의 제품 동작·Rust/Studio 렌더러를 바꾸지 않는다. 조판 원칙/Visual Sweep은 비해당이며
문서 표시의 정확성을 주장하지 않는다. 테스트는 독립 프로필의 unpacked Chrome for Testing에서만 실행한다.
스토어 설치·업데이트, 사용자 환경, Edge/Firefox 및 실제 문서 표시·인쇄는 별도 수동 검증이다.
도구 checkout SHA와 후보 빌드 SHA는 같다고 추정하지 않는다. 후보 해시와 출처를 함께 기록한다.

기본 CI의 Chrome 필수 게이트 도입을 철회했으므로 원래 #3515의 CI 완료 조건을 충족했다고
표시하지 않는다. #3512 전체 종료와 외부 E2E 증적 보고서 연동 이슈 생성도 이번 갱신에 포함하지 않는다.

## 사용법과 이력

현재 명령·결과 계약은 [확장 매뉴얼](../manual/chrome_edge_extension_build_deploy.md#39-배포-후보-선택-실행과-결과-보고-3515)을 따른다.
[이전 CI 구현·실행 기록](https://github.com/edwardkim/rhwp/blob/0b3da1cbd2ca6881bd760303312966f03cfabbbe/mydocs/report/task_m100_3512_report.md)과
[당시 반복 검증 JSON](https://github.com/edwardkim/rhwp/blob/0b3da1cbd2ca6881bd760303312966f03cfabbbe/mydocs/report/3512-extension-e2e/validation.json)은 과거 head에 고정해 보존한다.
이전 CI 성공을 현재 도구의 실행 증거로 재사용하지 않는다.
