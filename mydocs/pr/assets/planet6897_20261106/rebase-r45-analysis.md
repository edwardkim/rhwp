# R45 리베이스와 충돌 해소 기록

## 완료한 Git 작업과 검증 범위

이전 base `5077b216cbc51d2046da07350de562f6f521a7c1`에서 새 devel
`7f76cee60e38fceb63feacaff706cf984b19c427`로 142개 커밋을 전용 작업 트리에서
리베이스했다. 마지막 `git rebase --continue`의 실제 종료 코드는 0이며,
결과는 `0e8e2a6059130a19e37398545e9da8433e74b670`이다.

리베이스 전 head는 `741a9a524e374da83c2ec0b6917317518007e0bb`다.
142개 전체의 저자·이메일·저자 시각·제목·순서를 Git에서 대조해 모두 보존했음을 확인했다.
원 기여자 27개 PR의 56개 커밋과 메인터너 보정·기록 커밋을 합친 수다.
원 PR을 다시 체리픽하거나 원격 head를 수정하지 않았다.

기본 브랜치에도 `5884be349194176fae39298c4084efc71ca45bf4`로 반영했다.
이는 위 제품 head 위에 완료된 이전 검증과 리베이스 기록을 보존한 문서 커밋을 얹은 결과다.
실제 반영 명령의 종료 코드는 0이며, 새 base가 조상임을 확인했다.
제품·검사·공급 도구 4,691개 파일의 SHA를 고정하고 새 후보의 검증을 시작했다.
Git 충돌 해소와 format 성공은 기능 보존의 증거가 아니다.

## 충돌 처리

| 위치 | 충돌 파일 | 보존한 계약과 실제 확인할 경계 |
| --- | --- | --- |
| 글꼴 registry | `assets/font-rules/font_rule_registry_v2.json`, `assets/font-rules/font_rule_projection_manifest.json`, `scripts/tests/font_rule_projection_gen.test.mjs`, `tests/cases/issue_4966_font_rule_projection.rs` | 새 base의 한겨레 웹폰트·CanvasKit 규칙과 원 PR의 윤고딕250 메트릭을 정본 reducer로 합쳤다. change set 순서를 21로 정하고 parent hash·projection을 공식 도구로 생성했다. active 855·retired 8, Node 계약 16건 통과. sealed v1은 보존했다. |
| 본문 넘침 원장 | `tests/fixtures/body_overflow_baseline.tsv` | 새 base에서 복원한 `issue5679/10857_delegation_rules.hwp`의 1과 원 PR의 `issue8596/policy_committee_agenda.hwp`의 1을 함께 보존했다. 허용값을 늘리지 않았다. |
| 글자 겹침 원장 | `tests/fixtures/text_overlap_baseline.tsv` | 같은 전결규정의 1과 `issue8641/field_table_after_page_box.hwpx`의 6을 함께 보존했다. 서로 다른 입력의 추가 행이며 기존 수치를 바꾸지 않았다. |
| 공백 줄바꿈 | `src/renderer/composer/line_breaking.rs` | 새 base의 `first_interword_separator` 제외 조건을 유지하면서, 해당하지 않는 줄 끝 공백의 원 PR #8639 `cut`·`hung_extra` 소비를 보존했다. 첫 낱말 구분 공백·문단 끝 공백·#7418 경계의 실제 결과를 함께 확인한다. |
| 표 host 이월 | `src/renderer/typeset/table/block/prepare.rs` | #8652의 보이는 되감긴 host는 방금 낸 줄을 거두고 표와 함께 다음 쪽에서 다시 준비한다. 그 밖의 경로에서는 새 base의 계산된 빈 host 선행 채움을 유지했다. 빈 host와 보이는 host의 소유 차이를 같은 조건으로 합치지 않았다. |
| 구형 수식 | `src/renderer/equation/layout/script_geometry.rs`, `src/renderer/equation/layout/source_glyph_runs.rs` | 원본의 font/version 문자열이 실제 생략된 수식과 HYhwpEQ를 명시한 구형 표시를 모델 속성으로 구분했다. 전자의 한글·수학 기준선과 후자의 아래첨자 0.28em 계약을 각각 유지하고, 새 base의 DPI 전달을 보존했다. 두 표시 세대의 독립 Print와 회귀를 대조한다. |

충돌 파일 10개의 6차 중단을 해소했다. 조판 구현 파일 네 개는 `rustfmt --check
--edition 2021` 종료 코드 0을 확인했다. 이를 전체 Cargo format·컴파일·회귀 통과로 쓰지 않는다.

## 새 base에서 보존할 기능

새 base는 [PR #8754](https://github.com/edwardkim/rhwp/pull/8754)의 squash 결과다.
그 PR의 최종 review·실제 diff·전쪽 원장을 먼저 확인했다. 새 체리픽 대상에 추가하는 작업이
아니며, 현재 PR과 공유하는 조판·글꼴·그리기 경로의 리베이스 보존 검사다.

- 셀 도형 편집의 실제 선택·이동·삭제·Undo 및 저장 재열기.
- 명시 구형·현대 수식의 글리프·글줄·앞뒤 문항 보존과 생략 속성 수식의 별도 계약.
- 표 첫 조각·분할·이어받기·종료와 후속 본문의 소유·점유.
- 원본 TTF → TTC의 일치 face → OTF 순서와 실제 family·style·coverage에 따른 공급.
- 실제 Native Skia·Canvas2D·지원 CanvasKit 소비와 현재 원 PR의 변경 영역.

새 devel과 후보는 같은 입력·독립 Print·글꼴 공급·DPI·compat에서 대조한다. 기존 PR
#8389·#8397의 표시값 하락 금지 및 현재 PR 전쪽 90% 조건을 그대로 유지한다.
과거 producer나 Mac의 점수를 현재 Linux 후보의 새 실행 결과로 옮겨 쓰지 않는다.

## 로컬 로그와 관측 오류

아래 로그는 ignored `output/pr-review/planet6897-integration-20261106/` 아래에 있다.

- `logs/rebase-r45.log`: 최초 실제 리베이스와 첫 충돌, 종료 코드 1.
- `logs/rebase-r45-font-registry-resolution.log`: 정본 projection 생성·Node 16건, 종료 코드 0.
- `logs/rebase-r45-continue-01.log`: 첫 충돌 해소 뒤 본문 넘침 원장 충돌까지의 실제 출력.
  이 호출은 부모가 종료 코드를 수집하지 못했으므로 코드를 추정하지 않는다.
- `logs/rebase-r45-continue-02.log`부터 `05.log`: 다음 충돌에서 각각 실제 종료 코드 1.
- `logs/rebase-r45-continue-06.log`: 142/142 완료, 실제 종료 코드 0.
- `logs/rebase-r45-conflict-format.log`: Rust 충돌 파일 format, 종료 코드 0.
- `rebase-r45-all-commit-map.tsv`, `rebase-r45-commit-preservation.json`: 실제 Git 대응표와 저자·순서 보존 확인.
- `logs/r45-adopt-rebase.log`: 기본 브랜치 반영, 실제 종료 코드 0.

sparse 작업 트리에서 기존 정본·Print가 보이지 않아 projection 명령이 중단됐을 때는
해당 HEAD 또는 stage된 새 파일의 정확한 Git blob만 materialize해 재실행했다.
입력을 새로 만들거나 검증 조건을 우회하지 않았다. 실패 로그도 구분해 보존했다.

WASM 실행 준비의 `PermissionError`는 검토용 셸 스크립트의 실행 비트 누락이었다.
같은 스크립트를 `bash`로 명시해 재개했으며 제품 코드를 바꾸거나 이미 통과한 58건을
반복하지 않았다. 재개 전 실패를 최적화 WASM 빌드 성공으로 세지 않는다.

## 리베이스 직전 보정의 실제 결과

`741a9a524e374da83c2ec0b6917317518007e0bb`의 최적화 WASM 빌드와 실제 브라우저
3건은 통과했다. 정책 문서의 1쪽은 2020/2024 조건에서 각각 99.95%/99.99%로,
보고서 838의 49쪽은 99.99%로 복원됐다. Native의 같은 보고서 49쪽도
94.99% → 99.99%이며 표 뒤 제목의 위치를 review·overlay에서 직접 확인했다.

반면 법령 보고서의 WASM 5·10쪽은 여전히 99.85%/99.54%다. 합성 CFF 누락
글리프 검사의 통과를 이 실물의 개선 증거로 승격하지 않는다. 새 base에 들어온
TTF/TTC/OTF 공급 순서와 실제 프로그램 선택까지 포함해 다시 대조한다.
두 쪽의 이전 100.00% 대비 하락을 없앴다는 판정은 아직 하지 않았다.

새 base가 정해져 아직 진행 중이던 이전 `5077` base·`026168` 후보의 WASM 보호
측정은 중단했다. 이미 완료한 원문 TSV와 부분 산출물은 보존했으며, 전쪽 완료나 새
후보 통과로 표시하지 않는다. `r43-render-superseded-for-r45.json`에 해당 프로세스와
당시 상태를 기록했다. 새 후보와 새 base의 실제 결과를 별도 경로에서 산출한다.

## R45 후보의 실제 실행 결과와 추가 보정

후보 `5884be349194176fae39298c4084efc71ca45bf4`의 충돌 경계 집중 회귀는
110건 모두 통과했다. 실제 종료 코드 0이며 `logs/r45-conflict-boundaries.log`에
기록했다. Native 필수 편람 gate는 전 383쪽, 최저 99.96%, 기존 기준 대비 하락 0쪽으로
통과했다. TSV 표시값이 100.00%인 쪽은 382쪽이다. 이전 후보의 382쪽 수치를 재사용하지 않는다.

표시용 글꼴 별칭을 자동 공급하면 정책연구 검토 요청 문서의 원본 `substFont`가 체인에서
사라지는 실제 회귀가 있다. 원본의 `KoPub돋움체 Light,한컴바탕`이
`KoPub돋움체 Light,KoPubDotum Light`로 바뀌고 U+2024의 최종 전진이 164 HU에서
480 HU로 늘어난다. 글꼴을 다시 설치하거나 문서별 위치를 보정할 문제가 아니다.
`lookup_font_name_in_environment`에서 별칭을 적용할 때 원본 대체 공급자를 제거하는 것이
원인이다. `preserveSourceMetrics`인 표시용 세션의 기존 공급 계약을 보존하되, 사용자가
새 자연 프로그램의 메트릭을 선택하는 세션은 기존 계약대로 구분해야 한다.
현재 실제 1쪽 raw는 Native 99.73%, fresh WASM 99.75%이며, 별칭 전후의 동일 원본
글자 전진을 비교하는 관계형 검사를 추가해 검출한다. 시스템 글꼴 설치를 전제하지 않는다.

또한 CFF 누락 글리프의 윤곽 선택 검사는 Print만 확인하고 있었다. 같은 공개 CFF·OFL TTF
입력을 Screen에도 적용한 실제 브라우저 검사에서 1건 실패·1건 통과를 관측했다.
`faux_bold_glyph`의 `allow_hft_outline` 조건 때문에 Screen에서 분리 CFF의 결측을
Canvas에 맡기지 않고 뒤 family의 윤곽으로 바꾸는 것이 원인이다. 등록된 primary의 결측
소유 판단과 그 primary 윤곽을 직접 그릴 수 있는 profile 조건을 구분해 보정한다.
이 검출 결과를 법령 보고서 실물의 개선 완료로 쓰지 않고 새 producer로 재측정한다.

알려진 결함이 있는 후보에서 새 전체 회귀를 시작하지 않도록 전체 검사 시작을 종료 코드 75로
보류했다. 실제 실행한 110건과 전체 미실행을 구분한다. 보정 후 최종 head의 필수 검증을
다시 수행하며 기대값이나 baseline 허용폭을 낮추지 않는다.

### 보정 단계 고정

새 관계형 검사 `display_alias_declared_glyph_fallback`는 수정 전 실제 최소 글꼴 환경에서
1건 실패(종료 코드 100)했다. 같은 원본의 비교 전진은 `[0.0, 2.186666…]`인데 별칭
적용 뒤 `[0.0, 6.4]`가 되어, 좌표 기대값을 낮추지 않고 결함을 검출했다.
메인터너 보정은 표시용 세션에서 기존 대체 체인을 보존하고, CFF 등록 primary에 글리프가
없으면 윤곽 재생 profile과 관계없이 Canvas의 실제 동반 face 선택에 맡긴다. 이 단계의
집중 검사·새 producer·실물 재측정은 진행 중이며 통과를 미리 기록하지 않는다.

### 표시용 체인 보정의 검증과 실제 윤곽 선택의 추가 경계

`9cb885e18e`의 보정 뒤 최소 글꼴 환경 집중 회귀 13건이 모두 통과했다.
새 별칭 검사는 수정 전 FAIL → 수정 후 PASS이며, PR #8754의 사용자가 직접 선택한
프로그램·슬롯별 선택·자동 HFT 신원 보존 검사도 함께 통과했다. 실제 로그는
`logs/r46-policy-before-test.log`(100), `logs/r46-policy-after-focus.log`(0)다.

법령 보고서는 공급 catalog에 같은 family의 TTF·CFF가 함께 있어 coverage 등록을 생략한
경로였다. `TextFontCoverage::select`는 미등록 family를 건너뛰므로, 뒤의 Haansoft Batang
TTF만 등록돼 있으면 그 윤곽을 실제 Canvas 선택으로 오인했다. 원 해결 PR #8430의
`20a8ee978f`와 CFF 연결 보정 `406cc8c157`, 새 base의 실제 공급 경로를 대조했다.

공개 글꼴을 같은 주 family에 FontFace로 적재하되 coverage는 생략하고 뒤 family만
등록한 실제 브라우저 반례가 수정 전 실패했다(종료 코드 1,
`logs/r46-unknown-primary-before.log`). 공통 선택 결과에 앞 공급자의 확인 상태를 함께
반환해, 등록된 원 CFF나 확인된 동일 체인의 실제 프로그램만 윤곽으로 재생한다.
앞 family가 미확인이면 Canvas에 맡기며 문서별 글꼴·기호·좌표 조건을 추가하지 않는다.
실물의 전쪽 개선과 새 producer의 최종 회귀는 별도 실제 결과로 확인한다.

### 실물 보정 결과와 제출 전 lint 정리

`cbe1f41507cf783f433acbbf62cefb8a9b0fe669`의 실제 최적화 WASM 브라우저 검사는
3건 모두 통과했다. CFF 누락·미확인 주 글꼴의 수정 전 실패를 실제 수정 후 PASS로 확인했다.
정책 문서의 2020/2024 기준 1쪽은 Native 99.91%/99.99%, WASM 99.95%/99.99%로
복원됐다. 법령 보고서 5·10쪽은 Native·WASM 모두 100.00%다. 보고서 838의 49쪽은
Native 99.95%, WASM 99.99%다. 각 TSV는 실제 출력의 1·1·2·1쪽 범위이며 이를
전쪽 완료라고 표시하지 않는다. `r46-critical-{native,wasm}-progress.json`과 대응 로그에
producer·범위·실제 종료 코드를 보존했다.

제출 전 확인 중 표시용 체인 선택식의 `obfuscated_if_else` 가독성 경고를 발견해
명시적 `if/else`로 정리했다. 조건·평가 순서·true일 때의 체인 복사·false일 때의 빈 Vec는
같으며 동작이나 기대값을 바꾸지 않는다. 위 결과는 정리 전의 실제 기능 검증으로 보존한다.
최종 전체 Rust·lint 및 Native/fresh WASM producer와 전쪽 TSV는 정리 후 소스로 실행한다.

### 100.00% 뒤 확대 판독에서 확인한 합성 굵기

법령 보고서 WASM 10쪽의 실제 review·overlay와 같은 영역 확대를 대조했다.
괄호 모양은 복원됐지만, 주 프로그램 정보가 미확인인 경우의 일반 Canvas 글자 경로가
인쇄용 최소 장치 선 굵기를 적용하지 않아 제목이 얇았다. 2px 실루엣 100.00%를
모양·굵기 일치로 대신하지 않았다. 진단 영역은 실제 PNG의 동일 좌표이며 공식 overlay와
함께 `r46-legislation-restored-wasm-panels`에 보존했다.

공개 FontFace의 등록·미등록 경로가 같은 Print stroke를 소비해야 한다는 실제 브라우저
반례를 추가했다. 수정 전 실제 값은 0.640, 독립 Print 장치 규칙을 소비하는 윤곽 경로는
1.000이었다. `logs/r46-fallback-weight-before.log`의 종료 코드는 1이다. 실물 원점의 px
핀을 추가한 검사가 아니라 공개 글꼴과 Canvas profile의 선 굵기 계약이다.

`faux_bold_outline::width_for_profile`을 공통 계산으로 두고 윤곽 경로와 일반 글자·효과
경로가 같은 규칙을 소비하도록 보정한다. 저장 폭·줄 구성·글자 원점은 바꾸지 않는다.
실제 outline을 모르는 주 글꼴을 뒤 family로 바꾸지 않으며 Screen의 원래 0.02em도
보존한다. 새 최적화 WASM에서 브라우저 반례와 실물 제목의 굵기를 다시 확인한다.


두 경로 모두 같은 인쇄 장치 폭을 쓰므로 예전의 `등록 뒤 > 미등록` 잉크 순서는 더 이상
계약이 아니다. 원 FontFace/윤곽의 래스터 방식 차이로 순서를 정하지 않는다. 같은 공개 글꼴과
같은 원문에서 bold만 끈 독립 대조군을 각 경로에 추가해 `합성 굵기 > 보통 굵기`를 양쪽
경로·Print/Screen에서 각각 검사한다. 수정 전 실패한 공통 장치 폭 검사는 그대로 유지한다.


## R46 최종 전체 회귀의 유일한 실패 분석

제품 source `65195e9730f35bdbd7ea0c9a393e07e66125202e`의 최소 공개 글꼴 5개 전체
Rust 실행은 13,043건 중 13,042건 통과·1건 실패·45건 skip으로 실제 종료했다.
실패 함수는 `issue_7445_document_substitute_lookup::document_substitute_preserves_the_print_heading_owner_and_width`다.
원 API 검사의 도입 커밋 `6097253468`과 PR #8191, 실물 좌표 검사의 반영 PR #8754 및
`mydocs/pr/assets/issue7445/completed-20261009/stage35-20261021/README.md`의 독립 Print 근거를 먼저 대조했다.

원 HWP의 대체 필드는 한컴바탕이며 HWPX는 HCR Batang이다. 동일 mock 가용 프로그램으로
최적화 WASM의 공개 API를 실행하면 두 원본 모두 46쪽과 제목의 동일 셀 소유·내용·순서를
유지하고 시작 58.853333px·끝 284.826667px로 독립 Print의 58.88px·285.04px에 해당한다.
실제 문자열은 HWP `Haansoft Batang,한컴바탕`, HWPX `Haansoft Batang`이다.
첫 프로그램은 같으며 HWP가 원래 선언한 후순위 glyph fallback을 보존한 것이다.
전체 CSS 후보 목록이 단일 문자열과 같아야 한다는 assertion은 선택 프로그램의 검사가 아니다.
선두 family를 정확히 비교하도록 고치고 독립 좌표·셀 소유·겹침·원본 직렬화 검사는 유지한다.

이 조사에는 새 온새미로 조판 개선을 추가하지 않는다. 실패한 기존 검사의 환경과 실물 선행
조건만 같은 원본·이미 보존된 독립 Print·제품 producer로 확인 중이다. 전체 실행의 exit 100과
원 로그는 보존하며 보정 후 집중 실행 결과와 분리해 보고한다. 원래 통과한 13,042건을 다시
실행하거나 새 source에서 단일 전체 실행이 통과했다고 표시하지 않는다.


## R46 최종 실행 계획의 호환 모드 정정

이전 실제 R43 체크포인트와 R45 manifest를 대조해 `trailing-space-2024`, `sample16`,
`policy-track2024`, `policy-flags0`, `overflow-space-2024`의 compat가 기존 2024와 달리
2022로 기록된 실행 계획 오류를 발견했다. 제품·기준 PDF·관용 거리·판정값은 바꾸지 않고
기존에 실제 검증한 compat2024로 복구했다. 첫 잘못 실행된 결과는 진단 이력으로만 보존한다.
작업 중이던 WASM 문서가 끝나고 렌더 lane이 비었는지 확인한 뒤 계획을 정정했으며,
원본·PDF·호환 모드·DPI·producer가 모두 같은 완료 건만 바이트 검증 후 재사용했다.
Native 원 PR 전쪽 실행은 아직 시작 전이어서 처음부터 올바른 계획을 사용한다.


실제 최종 전쪽 재검증은 HWP Native 최저94.65%·fresh94.55%, HWPX Native94.62%·fresh94.43%로 각각46쪽이다.
46쪽은 HWP Native99.78%·fresh100.00%, HWPX 양쪽100.00%이며 review·overlay를 직접 판독했다.
[현재 실측 원장과 PNG](substitute-lookup-r46/README.md)에 연결한다. 기존 제목의 ±2px·소유·내용·직렬화 assertion은 유지했다.
이 단계는 테스트 전용 변경이며 제품 source·Native/WASM producer·Studio 결과는 바이트 확인 뒤 재사용한다.
집중 재검증과 남은 lint는 별도 실제 로그로 확인하며 전체 실행의 실패 이력은 덮어쓰지 않는다.
