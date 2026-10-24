# 보고서 가독성과 위임 수동 판정 보정

Issue: #7449 · PR: #8178 · 보정 전 code head: `e32f63d2aac839190d2b8085f0ec5346fa4e0479`

## 요청과 판단

2026-10-24 사용자에게 실제 실행 시도의 공유 report.html/report.json과 비공개 원본 인덱스를
구분해 제시했다. 사용자는 가독성에 대해 목적에 맞는지 판단을 요청했고, 정보 제외의 적절성도
에이전트에게 위임했다. 사용자 본인이 직접 열어 통과 판정을 내렸다고 기록하지 않는다.

기존 HTML은 긴 메타데이터 JSON을 검사 결과보다 먼저 보여 줬다. 실제 실행 시도에서는 실패
단계·미실행 검사·PNG 누락 사유가 아래에 묻혀 목적에 충분하지 않다고 판단했다. 공유 범위는
추적용 SHA·버전·시각만 남기고 원본 경로·로그·진단·다운로드 정보를 제외하는 구성이 적절했다.

## 보정 동작

- 원본 상태와 보고서 판정, 확인할 문제, 검사별 결과를 먼저 보여 준다. 오류 코드는 한국어 설명을 함께 표시한다.
- 후보·도구 SHA와 환경 기록은 상세를 펼쳐 확인한다. 모바일에서는 검사 표를 세로로 읽는다.
- 파일의 검증됨과 검사의 통과를 구분한다. Chrome 버전은 설치 대상 기록으로 표시하며 실제 실행 증거로 쓰지 않는다.
- 수동 입력의 선택 필드 `reviewMode`는 direct/delegated만 허용한다. 기존 생략 기록은 unspecified로 유지하고,
  사용자 위임 판정을 직접 판정과 구분한다. 이름·메모·자유 텍스트는 공유 요약에 추가하지 않는다.
- 보고서 생성기는 기존 원본을 그대로 읽는다. 수동 통과도 실제 fail·incomplete·not-run을 바꾸지 않는다.

## 실제 실행 시도와 환경 제한

실행기는 실제 빌드 후보 `rhwp-chrome/dist`를 대상으로 실행했다. 합성 runner를 주입하지 않았다.
2026-10-24 12:31 KST 실행의 runId는 `b51dfe3c-599c-4c3b-aa88-d6e4aaff3143`이다.

| 항목 | 실제 결과 |
| --- | --- |
| 후보 버전 / inventory SHA-256 | 0.8.7 / `8ccccb9e9ee3a802f6436b7fe0615bc1af93ada2d0039c38c58f2d0ffe600295` |
| 실행 도구 SHA / 변경 여부 | `e32f63d2aac839190d2b8085f0ec5346fa4e0479` / false |
| 원본 result.json SHA-256 | `fabdb4c1b313eed53cf752dd583b91ae022dd0dab3cce5e966ea8c6d15d23609` |
| 원본 실행 / CLI | fail / exit 1 |
| extension-smoke | launch 단계에서 Chrome for Testing 미설치로 실패, exit 1 |
| download-interceptor / extension-lifecycle | not-run / not-run |
| 보고서 / CLI | incomplete / exit 1, failure-png-missing |
| 원본 로그·진단 JSON | 해시와 runId 확인, private.html에서 연결 |
| 실패 PNG | 브라우저가 실행되지 않아 없음. 대체 이미지를 만들지 않음 |

확장 JS/HTML/CSS는 현재 checkout에서 빌드했으나 WASM은 준비된 pkg 산출물이다. 이번에 fresh
WASM을 빌드하지 않았고 그 빌드 SHA는 미관측이다. 원본 environment.chrome은 Puppeteer의
설치 대상 버전이며 실행 관측 버전이 아니다. 후보 해시와 도구 SHA를 후보 빌드 SHA로 대체하지 않는다.

현재 관리형 환경의 지정 프록시로 공식 Chrome for Testing 다운로드 URL에 HEAD를 보내자
`Tunnel connection failed: 403 Forbidden`이었다. 허용 목록에 storage.googleapis.com이 없다.
시스템 Chromium의 확장 설치 금지 정책은 변경하지 않았다. 실제 브라우저의 확장 동작·다운로드·
수명주기 전체는 미검증이며 합성 계약이나 HTML DOM 검사로 이 보류를 해제하지 않는다.

smoke 서버가 읽은 기존 입력 samples/hwp3-pagedef-1915.hwp는 HEAD의 바이트와 동일하며 SHA-256은
`b272fdd218b4e91355167e63438a1605ef6902d75970c4ff5b8bae67087122d0`이다. 브라우저 실행 전
중단되어 문서 표시·인쇄 품질을 검사하지 못했다. 원문을 보고서에 복사하지 않았다.

## 검증과 수동 판정

- Node 24.19.0 계약 40 PASS / 0 FAIL / 0 SKIP. Node 22.23.4에서도 같은 40개 통과.
- Chromium 151 HTML 검사 1 PASS / 0 FAIL / 0 SKIP. 기존 CSP 비의존 DOM/escaping과 PNG 검사,
  5가지 합성 상태의 결과 우선 표시, 접힌 상세, 1200px·390px에서 상세를 접고 펼쳤을 때 가로 넘침 없음을 확인했다.
- 실제 실패 원본의 보정 전후 HTML을 1200px·390px 화면으로 캡처해 직접 판독했다. 상태·문제·검사 결과를
  먼저 읽을 수 있고 긴 해시도 상세에서 확인할 수 있어 **가독성 통과**로 판단했다.
- 실제 공유 HTML/JSON에서 후보·브라우저 절대 경로, 원본 stack·로그·진단 내용이 없는지 원본과 대조했다.
  합성 입력의 비공개 경로·다운로드 ID·원문·태그 제외와 DOM 보안 계약도 유지돼 **정보 제외 통과**로 판단했다.
- 수동 기록은 위 원본 SHA에 연결하고 reviewMode=delegated로 입력했다. 검토 시각은
  `2026-10-24T03:40:22.789Z`다. 두 통과는 사용자 위임에 따른 에이전트의 수동 판단이며 자동 인증이 아니다.
- 실제 실행 보고서는 위 수동 통과 뒤에도 원본 fail, 보고서 incomplete, 후속 검사 not-run을 유지했다.
  합성 예제에는 이 판정을 자동 적용하지 않았다.

산출물은 ignored `output/pr-review/7449/user-review-20261024/`에 보존한다. 공유 ZIP은 두 요약만,
비공개 ZIP은 원본 결과·로그·진단·private.html을 상대 위치 그대로 담는다. 이 세션에는 attach_artifact
도구가 없어 파일 링크와 화면 캡처로 제시했다. 사용자 측 링크 열기 성공은 별도 확인되지 않았다.

## 재현 명령

```bash
npm --prefix rhwp-chrome run test:e2e -- \
  --dist /workspace/rhwp/rhwp-chrome/dist \
  --output /workspace/rhwp/output/pr-review/7449/user-review-20261024/private/run

node rhwp-chrome/e2e/report.mjs \
  --input output/pr-review/7449/user-review-20261024/private/run \
  --output output/pr-review/7449/user-review-20261024/reviewed/report \
  --expect-candidate-sha256 8ccccb9e9ee3a802f6436b7fe0615bc1af93ada2d0039c38c58f2d0ffe600295 \
  --expect-tool-sha e32f63d2aac839190d2b8085f0ec5346fa4e0479 \
  --manual-review output/pr-review/7449/user-review-20261024/delegated-review.json

node --test rhwp-chrome/e2e/report.test.mjs rhwp-chrome/e2e/png-integrity.test.mjs \
  rhwp-chrome/e2e/run.test.mjs rhwp-chrome/e2e/tab-monitor.test.mjs \
  rhwp-chrome/e2e/failure-diagnostics.test.mjs rhwp-chrome/e2e/page-budget.test.mjs
RHWP_REPORT_BROWSER=/usr/bin/chromium node --test rhwp-chrome/e2e/report-browser.test.mjs
```

출력 폴더는 새로 생성할 때만 사용하며, 재현 시에는 각 --output 값을 새 경로로 바꾼다.
