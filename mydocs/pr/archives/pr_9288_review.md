# PR #9288 재검토

메인테이너 보완 소스 `10f5bca362`에서 F11 확인 항목 46개가 통과했다. 첫 글자 뒤 화면이 그려지기 전에 둘째 키가 도착할 때도 선택을 유지한다. 실제 브라우저 입력 12조합의 HWP·HWPX 재열기와 저장한 HWPX XML의 필드별 값이 일치한다. 배포용 WASM 선언과 JavaScript도 함께 보완했다. [보완 검증과 전쪽 증거](../assets/semanticist21-9288-9314-20261124/corrected/README.md)

2026-11-24 KST에 semanticist21의 보완을 확인했다. 원 보류 대상은 `fb23a086a1`, 재검토 대상은 `3fc45e7057300b21c49ced254cb476fdab69ac39`이며 edwardkim이 리뷰어로 지정되어 있다. 기존 Rust/WASM 누름틀 조회 기능에 Studio의 실제 F11 선택·활성화 경로와 브라우저 회귀 검사가 추가됐다. 각주·머리말 편집 시 입력을 해당 누름틀에 넣는 별도 기여는 #9354에서 검토할 수 있다. [PR](https://github.com/edwardkim/rhwp/pull/9288) · [이전 보류](https://github.com/edwardkim/rhwp/pull/9288#issuecomment-7088992176) · [기여자의 수정 설명](https://github.com/edwardkim/rhwp/pull/9288#issuecomment-7106024208)

현재 `devel` `9ee9a5b0b1`에 #9288을 병합한 소스는 `c028304f32b54bc2699c78a995e62f1e0c04de57`이다. 도형 전단을 다루는 #9314와 함께 실행한 후보 소스는 `38b38f2924dfa6a863e6dcef3bab27f14e479248`이다. #9288만 같은 base에 병합한 tree는 `f231c1cfdcd548b28448ea1699307c4134bbb1d5`이다.

## 시험 조건

본문 또는 1×1 표의 셀에 ‘앞 뒤끝’을 넣고 글자 위치 2에 빈 누름틀 ‘가’, ‘나’를 차례로 삽입했다. 문단 끝에서 실제 브라우저 키 F11 한 번은 ‘나’, 두 번은 ‘가’를 선택해야 한다. 각 선택에서 영문 키로 `xy`를 입력하거나 Chrome 입력기의 조합으로 ‘한’을 확정하고, 입력 직후와 새로 연 HWP·HWPX의 두 필드 값을 각각 대조했다. 정답은 고른 누름틀에만 입력값이 있고 다른 누름틀은 비어 있는 상태이다.

보완 전 대조군은 현재 `devel`의 Studio 소스를 직접 구동했다. Native·WASM 실행 파일은 `devel`과 제품 및 빌드 입력이 동일한 `e4ce75728a`에서 빌드한 것을 썼다. 8개 입력 조합 모두 이 조건을 충족하지 못했다. 예를 들어 본문에서 F11 한 번 뒤 `xy`를 입력하면 ‘가’에 `x`, ‘나’에 `y`가 들어갔고 두 파일 형식에서도 같은 값이 나왔다.

새 API 두 개를 호출하는 TypeScript 소비 예제를 별도로 컴파일했다. wasm-pack이 생성한 `pkg/rhwp.d.ts`를 사용하면 성공한다. Studio의 `public/rhwp.d.ts`와 실제 Vite 배포 빌드의 `dist/rhwp.d.ts`에서는 `findNearestControlAt`과 `setActiveFieldAt`이 모두 누락돼 컴파일이 실패한다. Studio 자체 타입 검사에서는 `../pkg`를 사용한다. [소비 예제의 컴파일 결과](../assets/semanticist21-9288-9314-20261124/public-api-types.json) · [Studio 타입 설정](../../../rhwp-studio/tsconfig.json) · [동일한 공개 바인딩을 맞춘 이전 커밋](https://github.com/edwardkim/rhwp/commit/53058f69c4e9fdef8aecfb7e47fed08a6b7d6d12)

## 실제 입력과 저장 결과

Chrome의 실제 F11은 기여자의 수정 후 ‘나 → 가 → 선택 없음’ 순서가 된다. 기여자가 추가한 E2E의 본문·셀 시나리오도 통과했다. 그 검사에서는 ‘가’를 선택한 뒤 한 글자 `z`를 입력한다. 두 글자를 연속으로 입력하면 결과가 달랐다.

| 같은 글자 위치의 두 누름틀 | 입력 | 입력 직후 | HWP 재열기 | HWPX 재열기 |
| --- | --- | --- | --- | --- |
| 본문, F11 한 번으로 ‘나’ 선택 | `xy` | 나 = xy | 나 = xy | 나 = xy |
| 본문, F11 두 번으로 ‘가’ 선택 | `xy` | 가 = x, 나 = y | 가 = x, 나 = y | 가 = x, 나 = y |
| 표 셀, F11 한 번으로 ‘나’ 선택 | `xy` | 나 = xy | 나 = xy | 나 = xy |
| 표 셀, F11 두 번으로 ‘가’ 선택 | `xy` | 가 = x, 나 = y | 가 = x, 나 = y | 가 = x, 나 = y |

한 음절 ‘한’을 조합 확정하는 네 경우는 모두 고른 누름틀에 들어갔다. 두 음절 ‘한글’을 차례로 조합 확정하면 본문과 표 셀 모두 ‘나’ 선택 시 ‘나’에 ‘한글’이 남았다. ‘가’ 선택 시에는 ‘가’에 ‘한’, ‘나’에 ‘글’이 들어갔다. HWP와 HWPX를 각각 새로 열어 확인했고, HWPX의 본문 XML을 독립적으로 읽어 필드별 값을 대조했다. [입력별·형식별 결과](../assets/semanticist21-9288-9314-20261124/field-keyboard.tsv) · [HWPX XML의 필드별 값](../assets/semanticist21-9288-9314-20261124/serialized-fields.json) · [두 음절을 입력한 본문](../assets/semanticist21-9288-9314-20261124/F11-body-korean-split.hwpx) · [표 셀](../assets/semanticist21-9288-9314-20261124/F11-cell-korean-split.hwpx)

‘가’를 고르면 필드 변경 이벤트는 실제로 ID 1을 보고한다. 첫 글자를 친 뒤 캐럿 위치가 달라지면서 ID 2로 전환되고 다음 글자가 ‘나’에 들어간다. `activateFieldAt`은 최초 선택 시의 문단 주소와 글자 위치가 모두 같은 동안만 고른 컨트롤 번호를 재사용한다. 같은 필드 안에서 진행한 입력과 사용자가 의도적으로 캐럿이나 필드를 옮긴 동작을 구분해야 한다. [선택 기록](../../../rhwp-studio/src/engine/input-handler-keyboard.ts#L2434) · [활성화 조건](../../../rhwp-studio/src/engine/input-handler.ts#L5506)

동일한 fresh WASM으로 F11, 누름틀 저장, embed 필드 이벤트의 Studio E2E를 실행해 모두 통과했다. Studio 전체 테스트는 2,834개 통과, 2개 건너뜀, 0개 실패였다. Studio 타입 검사와 배포용 빌드도 통과했다. 공개 글꼴 5개만 공급한 환경에서 같은 위치의 누름틀, 중첩 컨트롤 탐색, 책갈피·개체 삭제, 인라인 컨트롤과 필드 범위의 Rust 회귀 25개를 실행해 모두 통과했다. 원 head의 GitHub 검사 상태는 성공 35개, 건너뜀 6개이며 최신 devel과 충돌이 없다. [PR 검사](https://github.com/edwardkim/rhwp/pull/9288/checks)

원 기여만 적용한 시점에는 **머지 보류**로 판정했다. 본문과 표 셀에서 ‘가’와 ‘나’를 각각 골라 영문과 한글을 연속 입력하고, HWP와 HWPX로 저장해도 선택한 필드에만 전체 문자열이 남아야 한다. 배포 산출물의 공개 선언으로도 두 새 메서드를 호출할 수 있어야 한다.

## 메인테이너 보완 범위

작업지시자가 2026-11-24에 로컬 보완과 재검증을 승인했다. F11로 고른 컨트롤의 현재 문단 주소와 실제 필드 범위를 사용해 연속 입력과 필드 이탈을 구별한다. 본문·표 셀에서 한글 조합과 영문 연속 입력, 다른 필드 선택, HWP·HWPX 재열기를 비교하고 배포용 타입으로 소비 예제를 컴파일한다.

## 최종 로컬 검증과 통합 판정

공개 글꼴 5개 환경에서 최종 Rust 회귀 14,074개, 집중 36개가 통과했다. Native 공식 검사 4,072개, Studio 2,835개가 통과했다. Studio는 2개를 건너뛰었다. Native·WASM·워크스페이스 Clippy와 배포 빌드도 통과했다. [검사별 명령과 결과](../assets/semanticist21-9288-9314-20261124/corrected/local-gates.json)

**메인테이너 보완 후 수용 가능.** 최신 devel과 후보에서 보호 문서 4,046행의 PNG가 모두 같았고, Native 하한 미달은 0쪽이다. WASM은 양쪽에서 같은 14쪽이 공식 100.00%에 대해 99.87~99.99%였다. 2026년 11월 24일 작업지시자는 같은 픽셀인 기존 14쪽에 한정한 예외와 통합 PR의 CI 성공 후 셀프 리뷰·머지를 승인했다. 두 원 기여 SHA와 공식 보호값을 보존했다. [전쪽 비교](../assets/semanticist21-9288-9314-20261124/corrected/protected-pages.tsv) · [기존 차이 14쪽](../assets/semanticist21-9288-9314-20261124/corrected/protected-below.tsv)

## Merge 후 contributor PR comment 계획

기여자가 연결한 F11 선택·활성화와 중첩 영역 탐색에 감사드린다. 화면 갱신 전 영문 입력과 한글 두 음절, 본문·셀의 같은 위치 누름틀, 양 형식 재열기와 XML 판독 12건, 실행 취소·다시 실행, 배포용 WASM 소비를 설명한다. 기능 판정 근거인 필드별 저장값 TSV·브라우저 검사와 통합 PR 링크를 병합 SHA에서 확인한 뒤 게시한다. 각주·머리말의 입력은 별도 [#9354](https://github.com/edwardkim/rhwp/pull/9354)에 연결한다.
