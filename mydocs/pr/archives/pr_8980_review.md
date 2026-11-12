# PR #8980 리뷰 — fix(equation): 문서 수식 괄호 경계를 입력 끝으로 보아 그룹 루프 무한 반복·수십 GB 할당 차단 (#8979)

## 최종 판정

**승인 — 명시한 통합 제품 code에서 원 PR 주장 범위의 로컬 검증을 완료했다.** 통합 PR의 최신 head CI·mergeability를 확인한 뒤 승인된 병합을 진행한다. 원 PR 직접 병합·원격 게시 완료를 뜻하지 않는다.

## 접수 정보

- 원 PR: https://github.com/edwardkim/rhwp/pull/8980, 작성자 `planet6897`, 원 head `602feeae50c6978ae0723563bf4a9536850f3d6c`.
- 통합 branch `review/integration-20261113-planet6897-clean20`, 제품 code `d0cc46573c7505633721c19062e504faff017d59`, 고정 devel `31a422575722f634cfc2d9eb97e708da58aea188`.
- 원 PR reviewer `jangster77`를 API 확인했다. 원 head와 적용 내역은 [source-plan.json](../assets/planet6897_clean20_20261113/source-plan.json)을 따른다. 원 contributor branch는 변경하지 않았다.

## 변경과 검토 범위

닫히지 않은 수식 그룹이 외부 괄호 경계를 입력 끝으로 인식해 무한 반복·대량 할당을 피한다.

수식 parser의 그룹 루프 종료 경계. 원래 과도 할당을 base에서 무제한 재실행하지 않고 기존 회귀의 malformed/balanced 두 경계를 검증한다.

소스: `src/renderer/equation/parser.rs`. 문서 조판·측정·paint 변경은 이 PR의 직접 범위가 아니므로 관련 없는 그림을 근거로 붙이지 않는다.

## 실제 회귀·제품 경로 검증

최소 글꼴 macOS 격리 환경(OS 기본528/nonOS0·허가된 저장소 fixture)에서 전체 nextest를 실제 실행해 **13,420 PASS / 0 FAIL / 45 skip**, 테스트 실행510.868초를 확인했다.
그중 이 PR의 관련 모듈 **2건 PASS**를 최신 전체 로그에서 다시 추출했다. 의존 PR과 중복될 수 있으므로 고유 신규 검사 수로 합산하지 않는다.

검사 소스: `tests/cases/issue_8979_equation_unbalanced_paren_group.rs`.
기록: [individual-test-execution.json](../assets/planet6897_clean20_20261113/final-d0cc4657/individual-test-execution.json) · [rust-validation.json](../assets/planet6897_clean20_20261113/final-d0cc4657/rust-validation.json).

fmt·Native/WASM Clippy·workspace build·all-target Clippy를 통과했다. Native Skia lib workspace 합계4,034 PASS/12 ignore, PNG2 PASS, PDF36 PASS를 확인했다.
후속 핵심 회귀1건을 추가한 뒤 관련 모듈4+15=19건과 lint·manifest를 별도로 통과했다. 새 검사1건이 위 전체13,420에 포함됐다고 표시하지 않는다.
fresh WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 통과가 아니다.

## 공통 조판 원칙 준수 판정

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 제출 전 로컬 검증과 PR 상태 | 충족 | 아래 전체·집중·lint·Skia·Mac fresh WASM 기록. 통합 PR은 로컬 필수 검증 완료 뒤 Open으로 제출하며 최신 head CI는 별도 병합 조건이다. |
| 기존 이슈·해결 PR의 선행 분석 | 충족 | 원 PR의 고정 head·고유 커밋·의존 순서와 현재 base 동작을 대조했다. 반례 보정은 별도 d0cc 원인 기록으로 연결한다. |
| 구현 근거와 일반성 | 충족 | 위 API/파서 입력 경계 계약과 실제 회귀를 검토했다. 조판 수치·문서 ID 예외는 추가하지 않았다. |
| 측정·배치 일관성 | 비해당 | 조회·파서 종료·파일명 정책 변경이며 측정·paint 값을 바꾸지 않는다. |
| 리베이스 반영 PR의 기능 보존 | 비해당 | 이번 검토 branch는 고정31a42257 위에서 누적 적용했고 이후 base 전진이 없었다. 제출 직전 live base를 다시 확인한다. |
| 최종 출력 소비 계약 | 충족 | 아래 공개 CLI/API의 실제 입력·결과 또는 malformed/balanced 파서 회귀로 입증했다. |
| 분할·이어받기 계약 | 비해당 | 이 PR은 분할 scanner·continuation 소비 규칙을 바꾸지 않는다. 해당 변경은 다른 원 PR에서 개별 검토한다. |
| 줄 소속과 점유 높이 | 비해당 | API 주소·안전 종료·출력명 계약이며 조판 줄 높이 변경은 없다. |
| 사례와 증거의 독립성 | 충족 | 실제 공개 CLI 결과 또는 malformed/balanced 입력의 별도 정식 검사를 사용했다. |
| 기준값 변경 | 비해당 | 이 PR의 기존 baseline·golden 문턱 변경 없음. |
| 회귀 추가·변경의 선행 조건 | 비해당 | API/안전 검사로서 실물 조판 px 고정 회귀가 아니다. |
| 90% 통과 후 확대 판독 | 비해당 | 문서 픽셀·미세 간격을 주장하는 PR이 아니다. |
| 주장과 검증 범위 | 충족 | 아래 source·test 모듈·실제 결과·입력과 출력에 한정한다. 통합의 전체 보호 결과를 개별 PR의 새 개선량으로 쓰지 않는다. |

원래 무제한 할당 입력을 base에서 제한 없이 다시 실행하지 않았다. malformed 그룹의 외부 닫는 괄호 종료와 balanced 정상식의 기존 회귀를 실제 통과했다. 문서 픽셀 일치율은 이 parser 종료 계약에 해당하지 않는다.

## 통합 보호·후속 조건

Native 정본 보호18조건1,771쪽은 공식 두 자리 표시값 하락0이며 별도 `--handbook-gate`383쪽도 통과했다. fresh WASM 정본 보호18조건1,771쪽도 공식 두 자리 표시값 하락0이다. 같은 code의 전쪽 비교는 [최종 보호·주장 검증](../assets/planet6897_clean20_20261113/final-d0cc4657/whole-visual-qualification.json)으로 연결한다. 일반 기여자에게 비공개 OTF 설치를 요구하지 않는다.

## 실제 backend 선택과 초기화 출처

일반48건과 실제 주장 페이지 추가12건을 분리해 캡처했다. 일반 검사의 실제 Canvas2D 요청16건, 지원되는 실제 CanvasKit16건, 안전한 Canvas2D 대체16건을 확인했다. 추가 검사에서는 #8999의2쪽에 실제 CK4건·Canvas2D2건, #9001의14쪽에 Canvas2D2건·기존 preflight 미완료 대체4건을 확인했다.
일반·추가 observer의 원본 종료코드는 각각1이며 각 실행의 #9001 CK 미지원 요청에 대한8개 hard violation을 그대로 보존했다. base/후보 모두 기본 작업 한도50,000에서19쪽 중18쪽을 검사한 뒤 미완료로 판정해 CK를 시작하지 않았다. 한도·글꼴 guard를 우회하거나 CK 성공으로 계산하지 않았다. 그 대체 화면은 실제 Canvas2D와 픽셀이 같았다. 나머지 글꼴 미준비 대체12건은 기존 fontFallbackGate12/12를 통과했다. backend 간 RGB parity는 report-only이며8/16 통과·8/16 차이를 원본대로 남겼다.
실제 렌더48회·독립 초기화6회와 추가 주장 페이지12회의 WASM 수신 해시를 확인했다. `c8596bc0...`은 새 제품 빌드와 일치한다. Vite가 변환한 JS는 원본 파일 SHA와 직접 같지 않으므로, 설치 Vite의 실제 변환·응답 바이트를 재현해 초기화 응답6/6을 검증했다.
[실제 backend·54개 WASM 수신 근거](../assets/planet6897_clean20_20261113/final-d0cc4657/actual-backend-qualification.json) · [추가 주장 페이지12건](../assets/planet6897_clean20_20261113/final-d0cc4657/actual-claim-pages-qualification.json). 전자는54개, 후자는12개 영수증을 보존한다.

## 검증 입력 커밋 확인

**충족.** HWP/HWPX/HML·PDF85개를 실제 실행 파일과 검토 commit의 Git blob SHA-256으로 대조했다. 대용량 sample10 PDF(96,185,197바이트)는 기존 파일을 로컬 재사용했으며 이 PR에 새 PDF·사본을 추가하지 않았다. [입력 감사](../assets/planet6897_clean20_20261113/final-d0cc4657/committed-input-audit.json)를 따른다. 비공개 글꼴 바이너리는 Git에 넣지 않았다.

## 통합 PR 최신 code CI

[통합 PR #9033](https://github.com/edwardkim/rhwp/pull/9033)의 정확한 code head `257b6fabeefabbe7588b428d84b1078ea5be7299`에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/43307698017), CodeQL·Render Diff·Adapter·Proptest와 최종 CI 정책 검사가 모두 성공했다. Archive A–E 실제 합계는 **13,236 PASS / 0 FAIL / 45 skip**이며 macOS 로컬 전체 수와 합산하지 않는다. 실제 Canvas font binding도 CI 19 PASS/0 FAIL로 확인했다.

[정확한 head·실제 CI 실행과 archive 요약](../assets/planet6897_clean20_20261113/final-d0cc4657/pr9033-code-ci.json)에 실행 신원과 결과를 보존한다. 이 문서-only trailing head의 required checks·mergeability와 실제 병합은 후속 조건으로 구분한다. 통합 PR 번호만을 위한 추가 review 문서는 만들지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR의 최신 head CI 성공과 실제 병합 뒤에만 원 PR에 한국어 존댓말로 결과·통합 PR·merge SHA를 안내하고 close한다. 이 문서의 판정과 원격 조치를 구분한다. 원 PR 주장 범위와 검증한 실제 동작을 설명한다.
Visual Sweep 정본: [GitHub merge comment](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
이 API·안전 검사는 관계없는 문서 그림을 붙이지 않고 실제 명령·JSON·검사 결과를 안내한다.
여러 단락은 UTF-8 본문 파일로 게시하고 API로 실제 본문·이미지 URL을 재조회한다. 원 contributor fork branch는 삭제하지 않는다.
