# #7700 최종 로컬 검증

검증 code head는 `fc378076f23aa60838a2871a7532d09830384216`,
고정 base는 `170b886f167b705b6e630c5b34bb638d92fae0a9`다.
Mac의 최소 글꼴 환경에서 Native 집중 64건, Native Skia lib 4,039건·PNG 2건·직접 PDF 22건,
전체 nextest 12,655건 및 범위 게이트를 별도 완료했다. 전체 실패 0, nextest skip 45, Skia lib ignore 12다.
실제 명령·종료 코드·검사 요약·로그 hash는 gates.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/gates.json`)에 기록했다.
WASM은 locked web/no-opt 로컬 대체 빌드이며 Docker 최적화 결과가 아니다.

## 전쪽 비교와 정식 회귀

Native/fresh WASM 각 8조건 212쪽의 새 전쪽 TSV에서 90% 미달 0·devel 대비 0.1%p 초과 하락 0이다.
원본 103쪽의 최저는 94.75432%/94.75587%(51쪽)다.
Native 대조 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/native-page-comparison.json`)와 WASM 대조 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/wasm-page-comparison.json`)는
쪽별 원값·PDF PNG hash·v5와의 실제 출력 동일성을 연결한다. TSV는 `tsv/{base,candidate}/{native,wasm}/`에 있다.

원문 일반 셀 14,166개와 컨트롤 보유 5셀의 텍스트 소속을 양쪽에서 새로 대조해 누락·중복 0을 확인했다.
19·30·76쪽의 독립 Print ±2px 보조 검사와 내용 관계 검사는 phase3 근거를 유지한다.
새 출력의 직접 판독과 재사용 범위는 direct-review.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/direct-review.json`)을 따른다.
51쪽에는 기존 병합 띠 높이·중앙 정렬 차이가 남는다. 내용 누락·중복과 새 후퇴는 없고
전쪽 90% 조건은 충족하지만, 100% 완료나 px 핀의 자격으로 보고하지 않는다.

## 실제 소비와 준비 한계

원본 19·30·76쪽의 실제 Studio Canvas2D screen/print 6건과 SVG·Native PNG를 확인했다.
원본 103쪽은 CanvasKit preflight 50,000 작업 한도에서 준비가 끝나지 않는다.
고정 base와 후보의 네 preflight (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/ck-preflight-base-comparison.json`)는 정확히 같으며 한도·guard를 변경하지 않았다.
지원되는 표 대조군에서는 CanvasKit compat/default screen/print 4건과 정상 Canvas2D 2건을 확인했다.
CK parity는 report-only이고 auto-readiness gate는 별도 미실행이다.
이를 원본 103쪽의 CK 직접 replay 통과로 쓰지 않는다.
실제 backend·surface·Typeface·DOM 그림·준비 진단은 actual-consumers.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/actual-consumers.json`)에 있다.

성공한 두 소유 포트 실행의 실제 초기화 WASM 응답 12건 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/actual-wasm-responses.json`)과
JS 응답 12건 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/actual-js-responses.json`)을 빌드와 연결했다.
JS는 Vite의 단일 WASM URL 치환과 inline source-map 때문에 raw 응답 hash가 원본과 다르다.
원문 대응 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase4/studio-js-response-identity.json`)에서 치환을 되돌린 원문이 locked JS와 정확히 같음을 확인했다.
처음 포트를 잘못 선택한 실행과 중단한 실행은 최종 성공 응답·시각 증거에서 제외했다.
기존 사용자 서버는 보존했다.

## 상태와 보존 범위

이 단계는 로컬 보정·검증 완료이며 PR·merge·issue close 완료가 아니다.
최종 제출 head의 편람 gate와 PR 승인은 별도다.
로그·실패 실행·임시 driver는 ignored output에만 보존하고 글꼴 바이너리는 포함하지 않았다.
