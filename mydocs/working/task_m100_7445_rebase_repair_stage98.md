# Stage98: 글꼴 선택 trace의 현재 레지스트리 근거 검증

Issue: #7445. 제품 코드 검증 HEAD는 `1f54c3df17747b71c8be9b47b3198add4fa4fd3b`,
고정 base는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 기존 문서의 보정 범위를
확대하지 않고, 제출 전 관련 도구 계약에서 드러난 실패를 바로잡았다.

## 원 해결과 현재 실패

원 이슈 #4961, 원 PR #5122와 초기 구현 `43a726449c4349949704cdcad64ff0b1eb2d058c`, 병합 `0bc05ef81107ac61ec38d622f71b44a44d1b4821`의 W1 후보 계보를
현재 직접 레지스트리 투영 `39ff2f03ed`와 대조했다. 기존 검증기는
`font_rule_candidates.json#candidate.*`만 근거로 인정했다. 현재 제품의 직접 선택은 실제 정본인
`assets/font-rules/font_rule_registry_v2.json#rule.*`를 반환하므로, 공개 HWPX의 정상적인
document-substitute trace가 `evidenceAnchor does not match candidateId`로 실패했다.
과거 W1 파일에 없는 근거를 있다고 가장하거나 제품의 글꼴 선택을 바꿔 해결하지 않았다.

## 변경과 검출 범위

- 활성 상태이고 direct projection을 갖는 실제 레지스트리 행만 인정한다. 앵커·rule ID·관계와
  source boundary, face, policy, conditions, order 및 소유자로 재구성한 candidate ID를 대조한다.
- 과거 W1 앵커도 정본 경로와 candidate ID가 정확히 일치해야 한다. 다른 JSON 파일의 같은
  suffix, 바꾼 소유자·관계·후보·rule ID, 비활성 규칙을 거절한다.
- schema는 현재 descriptive rule ID를 표현한다. 문자열 형식과 실제 규칙 소속 검증을 구분하며,
  기존 공개 WASM trace 네 문서를 strict JSON Schema와 의미 계약 양쪽으로 검사한다.
- 실제 glyph·paint 동일성이나 라이선스를 입증하는 검사가 아니다. 글꼴 선택·메트릭·렌더링
  소스와 공급 글꼴은 변경하지 않았다.

## 실제 실패와 재검증

ignored output의 `rebase-d9fd85ed-20261023/stage98-script-contracts/`에 원 로그를 보존한다.

| 검사 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 기존 공개 WASM trace E2E | 실제 1 FAIL | 공개 네 문서의 schema·계보·반복 결정성 PASS |
| 새 직접 규칙·변조 계약 두 건 | 1 PASS / 1 FAIL | 모두 PASS |
| 다른 JSON 앵커 위조 | 초기 보정에서도 실제 FAIL | 정본 W1 경로까지 대조한 뒤 PASS |
| 최종 trace·schema·coverage 묶음 | 위 실패 로그 보존 | 25 PASS / 0 FAIL / 선택 비공개 POC 1 skip |

`trace-focused-before.log`, `registry-anchor-before.log`, `registry-anchor-spoof-before.log`,
`final-registry-trace-validation.log`가 각각 실제 실행 증거다. 선택 비공개 POC 부재를 공개 네 문서의
성공으로 합산하지 않는다. Python 시각 계약 191 PASS, HFT 준비·보안 계약 32 PASS도 확인했다.

## CI 호출의 실제 누락 보완

기존 Studio 단위 수집과 Rust suite, 고정 base의 workflow를 확인했으며 이 두 trace 파일은
자동 수집되지 않았다. 기존 `frontend-package-gates`의 WASM bindings/editor embed 단계에만
두 파일의 호출을 덧붙였다. 이미 생성한 fresh WASM과 설치한 Studio 의존성을 재사용한다.
새 job·WASM 빌드·권한·trigger·required check를 추가하지 않았으며 과거 PR 시각 래칫의
자동 연결도 복원하지 않았다.

배선 검사는 기존 workflow에서 실제 1 FAIL, 변경 후 workflow·impact·promotion Python 묶음
69 PASS이며 Node routing/policy 95 PASS다. 실제 변경 경로를 classifier의 정식 입력 객체로
판정한 결과는 full/workflow-contract 및 frontend package 범위다. 최초 잘못된 API 인자로
얻은 file-list-empty fallback은 INVALID로 구분했다.

전체 actionlint에서 기존 CI 요약의 Markdown backtick 형식 문자열이 ShellCheck SC2016으로
해석되는 것도 확인했다. backtick을 인용한 인자로 옮겨 환경 변수를 평가하지 않는 기존 의미를
유지했다. 빈 배열·공백 경로·한글·backtick과 달러 문자가 있는 네 입력의 출력 바이트가 기존과
같으며, actionlint exit 0과 최종 workflow 계약 69 PASS를 확인했다. 단순히 ShellCheck를
끄거나 기존 안내를 삭제하지 않았다.

현재 workflow 명령의 로컬 실행·공개 trace 검증은 끝났으며, 새 code head의 GitHub Actions
실행 성공은 PR 생성 뒤 별도로 확인한다. 이 단계 기록은 전체 PR 준비 완료나 CI 성공 선언이 아니다.
