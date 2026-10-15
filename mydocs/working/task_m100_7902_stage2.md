# #7902 2단계 — #7877 문서 회귀 보정과 재발 방지

Issue: #7902. 이미 merge된 PR #7877의 원본 19문서·835쪽 범위를 유지한다.
1단계 도구 PR #7903과 기록 PR #7906은 실제 merge됐으며 이 이슈는 OPEN이다.

## 현재 확인 상태 — 2026-10-15

아래는 이전 진단과 구분한 현재 집계다. **아직 PR 생성·전체 검증 완료가 아니다.**
아래 전쪽 집계의 실행 소스는`f5284cd0b`, 기준은`a5e88ad68`다.
PR 직전 다시 fetch하며 기준 SHA가 바뀐 경우에만 커밋·리베이스 후 필요한 검증을 갱신한다.

| 항목 | 실제 확인 결과 | 남은 범위 |
| --- | --- | --- |
| Native 원본 전쪽 |19문서835쪽,미달/누락0·Print 쪽수 일치,최저91.71770%(scattered37쪽)|최종 제출 head와 증적 연결|
| Native devel 대조 |비교 가능한18문서787쪽,0.01%p 이상 하락0쪽|overfill base49/Print48로 전쪽 TSV 비교 불가|
| fresh WASM 전쪽 |19문서835쪽,미달/누락0·Print 쪽수 일치,최저90.09386%(chemical19쪽)|nested HWP의 Chrome 시작 timeout은 같은 producer로115쪽 재실행 완료|
| WASM devel 대조 |nested HWPX104쪽99.93539→99.87312%,0.06227%p 하락1쪽|재검토 상태 유지,기준·글꼴·마스크를 바꾸지 않음|
| 새 회귀7개 |정식 Cargo Native selector7PASS;공통6개·Native 진단1개|최종 source의 전체 gate와 연결|
| Rust 전체 nextest |10301실행,10300PASS·1FAIL·45skip|정책연구 HWPX 본문 넘침1→2건의 원인 보정·재검증|
| Native Skia lib |全lib PASS(exit0),컴파일·실행428.84초|Native integration·doctest는 순차 실행 중|
| Rust lint·정책 |fmt,일반/WASM/전체 target 및 Native Skia Clippy PASS,manifest/base·unit tier PASS|후속 검사 변경의 lint/정식 Cargo 검증|
| 브라우저·CI 계약 |관련 Node6PASS,CI 계약328PASS,Python workflow186PASS,actionlint PASS|없음|
| Studio |단위1840PASS·2skip,fresh 공용 pkg 갱신 후 tsc PASS|생성 public wrapper는 제품 변경으로 stage하지 않음|

검증 결과는`final-f528-head-dev-verified-{native,wasm}-result.json`에 있다.
WASM 원래 실패 로그와 동일 producer의 nested HWP115쪽 retry를 별도로 보존하고
완료 원장에는 해당 행만 대체했다. Native PNG835장은 직전 a37 producer와 바이트가 같다.
전체 회귀의 실제1FAIL은 SIGINT나 빌드 오류로 분류하지 않으며 PR 생성·통합을 보류한다.

## 기준과 승인된 진행

- 브랜치: `fix/pr7877-render-regressions-20261014`.
- 현재 기준 `upstream/devel`: `a5e88ad682c805e1d845502ddc9521b94ea8eccd`.
  아래 c0 측정은 리베이스 이전의 고정 기준 기록이다.
- 공용 Cargo target: `/home/tsjang/rhwp/target/pr-review`, jobs8, Cargo 순차 실행.
- 사용자 승인 범위: 원본 출력 보정, Native/fresh WASM 전쪽 대조, 회귀 보강 및 필수 검증.
  먼저 영향 경계를 확인하고 최종 소스에서 전체 범위를 검증한다. 기존 값·허용치를 완화하지 않는다.
- 원 PR TSV는 이전 결과의 증거로 읽으며 이전 소스를 전쪽 재실행하지 않는다. 새 기준 devel은
  같은 입력·독립 Print·글꼴·DPI·backend로 TSV를 산출한다. 한 쪽·한 경로라도0.01%p 이상
  하락하면 재검토하며 평균 상승으로 상쇄하지 않는다.

## 우선 분석 경로

1. 쪽번호: 원본 `Page Number` 글자모양 → resolved style → 폭 측정과 TextRun →
   정렬 원점. 현재 코드가 font family/size/color만 전달해 bold 등 속성을 누락한다.
   본문과 같은 스타일 변환 결과를 측정·paint에 전달하고 PDF 및 양쪽 backend에서 검증한다.
2. rowbreak15쪽: 그림 선언 상자가 아닌 바깥 host table의 물리 높이·예약 공간·뒤 본문의
   원점을 추적한다. 저장 줄·그림 상자·빈 줄의 실제 점유 계약을 확인한다.
3. overfill: 5쪽의 노트만6쪽으로 이월되고 이후 내용이 한 쪽씩 밀린다. 시작116/끝124에서
   다음 unit의 높이·빈 띠·중첩 예약·컷과 paint 소비를 대조한다. 전체 내용 보존과 종료를 확인한다.

위 내용은 보정 완료 판정이 아닌 기존 진단에서 확인한 분석 시작점이다.

## output 전체 정리

`output/pr-review` 전체를 점검한다. 종료된 #7903/#7906의 중복·중간 산출물은 공개 증적과
필요한 원시 기록의 보존을 확인한 뒤 제거한다. #7902가 실제 사용하는 과거 producer·글꼴·TSV와
미해결 재현 자료는 보존 사유를 기록한다. 공유 target·font raster cache·원본/PDF는 보존한다.
제거 경로·확보 용량·보존 이유와 후속 처리 결과는 이 기록에 이어 적는다.

## 검증 상태

- 도구 기반 PR #7903, 후속 기록 #7906: 실제 merge 완료.
- c0 기준 Native/fresh WASM19문서 전쪽 및 스타일·빈 셀 보정 후보 전쪽 결과는 확보했다.
  기존 pipeline의18문서787쪽 비교에서 양쪽 backend 모두0.01%p 이상 감소0건이다.
  overfill은 기준 쪽수49/Print48로 TSV가 없으므로 비교 불가·재검토 상태를 유지한다.
- 후속 진단에서 WASM font registration의 실제 선택 실패를 발견해 수정했다.
  기존 WASM 수치는 실제 공급 글꼴 선택 증거로 사용하지 않으며, 동일 수정 도구로 기준을 다시 산출한다.
- 현재 선택 쪽 결과는 진단 dev producer의 결과이며 최종 head 전쪽 판정이 아니다.
  overfill43–46쪽, nested HWP104쪽 보정, 현재 소스의 전쪽·회귀 보강·최종 lint/전체 검증은 미완료.

## 기준 출력과 후속 처리의 실제 결과

- 기준 Native Skia `release-test` 빌드:5분36초, 성공. immutable binary와 소스별 SHA를
  `output/pr-review/issue7902-render-repair-20261014/base-source-proof.json`에 기록했다.
  빌드 head의 Rust/Cargo/vendor 소스는 기준 devel `c0d330737`와 동일함을 확인했다.
- Native19문서 전쪽 비교 완료:16통과/3미달 또는 측정 불가. `rowbreak-hwpx`15쪽
  87.45557%, `nested-hwp`104쪽89.87550%, `overfill-hwpx`49쪽/Print48쪽이다.
  나머지16문서의 관련 전쪽은90% 이상이다. WASM은 별도 빌드·검증 중이다.
  이는 현재 기준 출력의 측정이며 후보 보정 완료 증거가 아니다.
- `overfill`116..124의 내용 합과 마지막 노트124의 높이·저장 되감김125를 실제 진단했다.
  단순 슬롯 합과 최종 paint/저장 vpos의 간격 소유를 대조 중이다. 수치 관용 확대나 컷 강제
  연장은 적용하지 않았다.
- #7903의 merge 후 comment를 API로 다시 확인했다:
  <https://github.com/edwardkim/rhwp/pull/7903#issuecomment-6170034389>.
  두 PR의 실제 MERGED·최종 head·merge SHA의 devel 포함·다른 OPEN PR/작업 사용 여부와
  worktree를 확인한 뒤 두 전용 remote branch를 exact SHA lease로 삭제했다. 동일 local branch도
  삭제했다. 다른 작업의 `rhwp-7105-verify` worktree는 보존했다.
- `output/pr-review` 전체8개 하위 경로와 과거 semanticist 검토의 하위 용량 목록을 점검했다.
  종료된 도구 검토의 중복·중간 출력11개 경로를 제거해 할당 공간1.95363GiB를 확보했다.
  삭제 전 live process의 cwd/exe/fd/명령 사용 여부와 요약·공개 증적 보존을 확인했다.
  경로·바이트 수는 `output/pr-review/issue7902-render-repair-20261014/cleanup-result.json`에 있다.
  필요한 원시 기록은 같은 경로의 `foundation-records/`에 보존했다.
- 보존: 현재 #7902 원시 출력, 이전 PR TSV·검증 글꼴, immutable producer·선택 쪽 진단,
  공유 font raster cache. `tests/`·`regression-temp/`는 소유·용도 확인이 안 되어 보존했다.
  과거 semanticist 자료는 현재 사용하는 producer/글꼴과 재현 기록을 포함하므로 보존하며
  불필요한 하위 경로는 용도 확인 뒤 추가로 정리한다. 공유 Cargo target·원본/PDF는 유지했다.

## 공통 결과의 누락·중복 보정

- 쪽번호: `layout.rs::build_page_number`가 기존 본문 helper
  `layout/text_measurement.rs::resolved_to_text_style`의 영문 슬롯 결과를 사용하도록 수정했다.
  그 `TextStyle`로 폭을 재고 같은 객체를 TextRun에 전달한다. 기본 스타일과10pt fallback은
  유지한다. 문서 ID·쪽 번호·글꼴 별 정렬 예외를 추가하지 않았다. 현재 후보 Native 빌드 중이며
  아직 최종 시각 개선 완료로 판정하지 않는다.
- 빈 셀 문단: `float_placement::stored_empty_successor_advance`는
  `현재 앞 간격 + (후속 저장 vpos - 현재 저장 vpos) - 후속 앞 간격`을 반환한다.
  온전한 셀 paint와 partial paint는 `para_y_before_compose + advance`를 소비한다.
  반면 `cell_units_uncached`의 paragraph/line 두 갈래는 다시 `spacing_before + advance`를
  더하고 있다. `overfill` 문단92의 앞 간격500HU(6.66667px)가 이 이중 계상에 해당한다.
  측정도 이미 완성된 동일 전진량을 소비하도록 paragraph/line 두 갈래를 수정했다. helper의 수용 조건이나
  예산 관용치를 완화하지 않는다. 앞 간격0인 대조군은 높이가 그대로여야 한다.
- `rowbreak`의 그림 자체 높이는 독립 PDF와 대응한다. 현재 끝 조각 뒤 흐름·빈 host 문단의
  앞 간격·표 바깥여백 소비와 이후 원점 선택을 추적 중이며 미검증 수정은 적용하지 않았다.

## 선택 쪽 선행 관측

쪽번호 스타일 보정만 포함한 Native producer로 원본 chemical29/100쪽을 재출력했다.
29쪽의2px 관용 실루엣은94.89796%→98.26087%,100쪽은97.51244%→98.68996%로 개선됐다.
원본·독립 PDF·2022 compat·96dpi·글꼴 공급은 동일하다. 이 값은 전체 fidelity가 아닌 실루엣
보조값이며 Native 선택2쪽의 결과다. 현재 후보의 빈 셀 간격 수정은 이 producer에 아직 포함되지 않는다.
producer와 정확한 patch/SHA는 `output/pr-review/issue7902-render-repair-20261014/page-number-only-proof.json`,
실측값과 review는 같은 경로의 `page-number-only/native/chemical-original/`에 있다.
WASM 보정 후 비교·전쪽 무회귀·확대 판독은 남아 있으며 회귀 검사 신규 추가를 완료했다고 쓰지 않는다.

이중 계상 보정까지 포함한 Native 빌드는6분03초로 성공했다. 실제 `overfill` 전체 쪽수는
49→48로 복구돼 Print48쪽과 일치했다. 문단92의 유닛121도38.933333→32.266667px로 바뀌어
helper가 반환하는 동일 전진량을 소비한다. 예산·컷·쪽수 기대값을 완화하지 않았다.
이는 쪽수 및 해당 높이 관측이며 전쪽 시각 완료 판정은 아니다. 현재 두 보정의 Native19문서
전쪽 비교와 새 WASM 빌드가 실행 중이다. 기준 WASM19문서 전쪽 비교는749.9초로 완료됐으며
13통과/6미달 또는 쪽수 불일치다. `nested-hwp`104쪽은 WASM90.61786%로 통과하지만
Native89.87550% 미달이므로 전체 통과 문서로 판정하지 않는다.

## 2026-10-15 `output` 전체 정리 결과

사용자 요청에 따라 `output/pr-review/<id>`에 한정하지 않고 `/home/tsjang/rhwp/output` 전체를
점검했다. 종료된 #7877·#7882·#7903의 실제 MERGED 상태와 현재 검증의 종료를 확인한 뒤
프로세스 cwd/exe/fd 및 mapped file 사용 여부를 검사했다. 불필요한 파일 109,476개와
빈 디렉터리 19,376개를 제거해 할당 공간 약 30.288GiB를 정리했다.
삭제 대상은 과거 실행 파일·WASM·browser scratch, 전체 sweep의 중간 raster 및
재생성 가능한 embedded-font 복사본, 옛 render tree·중복 compare PNG·소스 복사본,
종료된 테스트의 진단용 출력이다. 원본 `samples/`·`pdf/`와 공유 `target/pr-review`는 보존했다.

현재 #7902의 기준·후보 immutable producer, 소스 SHA/patch, 원 PR TSV, 현재 전쪽 TSV74개와
manifest, 로그 및 대표 review/overlay PNG를 보존했다. 검증 글꼴과 producer 등을 포함한31개
보존 파일의 SHA-256은 정리 전후 동일하다. 공유 font raster cache도 유지했다.
`output/poc/hwpx2hwp/task903`은 다른 작업의 ignored 진단 테스트가 입력으로 참조하므로 보존했다.
`output` 루트에 남아 있던 generated 회귀 출력도 제거했으며 루트 파일은0개다.

정확한 삭제 경로·용량·사용 확인은 ignored
`output/pr-review/issue7902-render-repair-20261014/cleanup-output-result-20261015.json`과
`cleanup-output-final-pass-20261015.json`, 보존 SHA는 `cleanup-protected-hashes-20261015.json`에 있다.
정리는 렌더러 소스나 테스트 기대값을 바꾸지 않았으며 전체 회귀를 재실행하지 않았다.
#7902 출력 보정 및 최종 검증은 별도로 계속 진행할 미완료 작업이다.

## 실제 Bold 선택과 글꼴 등록의 후속 보정

1. SVG의 기존 actual Bold 별칭을 `renderer/font_face_aliases.rs`로 옮겼다. Canvas는
   같은 후보의 full face 이름에서 CSS base family를 얻고 요청 weight를 적용한다.
   Regular 체인과 첨자·장평·CSS 크기 계산은 유지한다. positioned provider/paint 및
   회전·글자겹침·boxed PUA의 family 소비도 같은 helper로 연결했다. 후자의 최종 빌드·시각 범위는 남아 있다.
2. 최초 full-name 후보 시도는 chemical29쪽85.61151%를 개선하지 못했다. 실제
   Canvas descriptor/measureText를 캡처해 family + weight 선택으로 바꿨다.
   초기 인용 registration 경로의100% 관측은 아래 공급 결함이 있으므로 최종 증거로 사용하지 않는다.
3. `FontFace(JSON.stringify(family), …)`는 글꼴이 loaded여도 Canvas에서 원 family로
   선택되지 않았다. 실제 독립 프로그램의16px 숫자 advance8.8과 인용 alias의9.44가
   다름을 통제 실험으로 확인했다. raw family 등록은8.8을 재현한다.
   `scripts/visual-sweep-font-registration.mjs`를 실제 공급 경로에서 공유한다.
4. 기존 MIT 합성 glyph fixture(1000 UPM·advance900·300×700 직사각형)로 실제
   브라우저 회귀를 추가했다. 영문 공백 family와 한글 family 모두 advance36px 및
   ink12×28px을 검사한다. 인용 등록의 수정 전 FAIL(status loaded, advance24.31998,
   ink25×29), raw 등록의 수정 후 PASS를 확인했다. 이것은 글꼴 공급 계약이며 한컴 문서
   일치 증거로 대신하지 않는다. Render Diff의 Chromium 준비 후 이 검사를 배선했다.
5. Node 실제 glyph/HTTP 공급5개, Python PNG·workflow19개, actionlint 통과.
   source rendering 회귀의 신규 추가 및 전체 Rust 검증은 아직 완료하지 않았다.

올바른 raw family 등록 이후 같은 입력·Print·96dpi·8개 글꼴 경로의 선택 쪽:

| key | 쪽 | fresh WASM |
| --- | --- | --- |
| chemical-original |29 /100 |98.05825% /98.00000% |
|6761-control |29 /100 |98.05825% /98.00000% |
|6761-shifted |29 /100 |98.05825% /98.00000% |
|6761-later |29 /101 |98.05825% /96.41026% |

later101쪽은98% px 보호 대상으로 판정하지 않는다. 이 값은 전체 문서 완료 판정이 아니다.
원시 결과는 `actual-font-registration-dev` 및 `actual-font-registration-variants-dev`에 있다.
rowbreak15쪽의 이전 dev Native92.37322%·fresh WASM91.90583%는 terminal accepted flow end
소비의 선행 관측이다. 새 registration 도구 및 최종 소스에서 다시 확인한다.

c0 immutable WASM/CLI를 새 도구로 실행한 `base-canvas-family-registration` 전쪽 비교가
671.8초로 완료됐다.19문서 중13통과/6미달 또는 쪽수 불일치이며, rowbreak15쪽87.54603%,
chemical 계열29쪽77.77778%, overfill49/Print48쪽이다. nested HWP104쪽은 WASM91.01640%다. 과거 tool의 TSV와 혼합해 동일 조건 무회귀를 주장하지 않는다.

## overfill 컷의 후속 원인 범위

26행 자식 표의24행 조건 셀은 원본 LineSeg의27번째 unit에서40960→0HU로 재시작한다.
canonical child ledger의91번째 unit도 row24/start-cut27을 그대로 갖고 있다.
실제45쪽은 부모 cut1278→1311, child cut83→98을 받아 row24/start-cut19로 시작한다.
따라서 현재 증거는 커서 필드가 사라진 것이 아니라 예산이 원본 재시작보다8유닛 이르게
끝난 것을 보여준다. scan의 물리 예약·반복 헤더·원본 프레임 빈 밴드와 paint 점유를
연결해 추적한다. 뒤 빈 줄 삭제나 임의8줄 컷 보정은 적용하지 않았다.
`typed-nested-cut-diagnosis.log`와 `nested-frame-diagnosis-render.log`에 실제 값이 있다.
진단 필드 추가를 포함한 dev Native 빌드1분35초는 성공했다.

## 반복 제목행의 예약 중복 제거 — 선택 쪽 관측

`mixed_nested_flow_extra_from_cut`의 마지막 원자 행 예약은 typed child fragment에도
제목행을 더했다. 같은 컷의 `row_cut_content_height`와 `nested_continuation_header_height`가
제목행을 예약하므로 동일 물리 띠가 두 번 계상됐다. `atomic_nested_header_reservation`을
실제 원자 유닛에 한정해 typed fragment의 기존 예약과 분리했다. 일반 원자는 유지한다.
Source ID·쪽 번호·예산 관용·컷 개수의 강제 보정을 추가하지 않았다.

같은 원본·2024 Print·2024 compat·96dpi·글꼴8경로의 dev Native:

| overfill 쪽 | 앞 후보 | 중복 제거 후 |
| --- | --- | --- |
|43 |86.55231% |93.05352% |
|44 |57.17436% |81.35610% |
|45 |56.39796% |99.95610% |
|46 |11.82210% |99.97079% |

쪽수48/Print48을 유지하며45쪽의 끝 행 및46쪽 제목 시작이 복구됐다.
44쪽은 표의 행 소속·순서가 복구됐지만 긴 PCB 조건 셀의 글줄 원점·세로 정렬 차이가 남아 미완료다.
이 결과는 선택4쪽의 선행 확인이다. 최종 전쪽·fresh WASM·관계형 회귀·lint 통과로 보고하지 않는다.
producer/patch/hash는 `single-header-reservation-dev-proof.json`, review와 실측은
`single-header-reservation-dev/native/overfill-hwpx/`에 있다. 직접 SVG를 만들지 않았다.

## 원본 문단 흐름과 자식 표 바깥여백 — 중간 커밋

원본 LineSeg가 같은 문단 내부에서 재시작하는 분할 셀은 선택 조각을 독립적으로 가운데
정렬하지 않고 저장된 문단 흐름의 시작을 소비한다. 변경 조건은 편집되지 않은 저장 줄과
canonical unit의 원본 재시작이며, 문서 ID나 쪽 번호로 분기하지 않는다.
overfill 44쪽의 실제 정렬 경로에서 발견한 불필요한 가운데 정렬 간격을 제거했다.

typed 자식 표 이어받기는 반복 제목행이 없어도 비인라인 표의 바깥여백을 예약·배치한다.
자식 표 다음 문단은 그 조각의 canonical flow height를 소비하도록 연결했다.
중첩 표 HWP 104쪽의 자식 표에는 위·아래 각각140HU 바깥여백이 있으며 기존 경로에서
이를 빠뜨린 것을 원본 모델과 실제 호출 경로로 확인했다.

동일 입력·독립 Print·2024 compat·96dpi의 dev Native 선택 쪽 결과:

| 대상 | 쪽 | 보정 후 |
| --- | --- | --- |
| overfill-hwpx |43 /44 /45 /46 |93.05352% /85.14566% /99.95610% /99.97079% |
| nested-hwp |104 |93.84310% |

Native 빌드는1분13초에 성공했다. overfill 44쪽은 여전히90% 미만이므로 미완료다.
이는 선택 쪽의 선행 관측이며 전쪽·fresh WASM·전체 회귀·lint 완료 판정이 아니다.
producer와 소스 patch/hash는 `source-flow-and-outer-frame-dev-proof.json`, 원시 측정은
`source-flow-and-outer-frame-dev-native-result.json` 및
`nested-outer-frame-dev-native-result.json`에 보존했다. 사용자 요청으로 현재 변경을
중간 커밋한 뒤 upstream 변경 여부를 확인하고 필요한 리베이스 및 보정을 이어간다.

## upstream 동기화와 원본 정렬 계약 재검토

사용자 요청으로 중간 소스·증거를 `b6d6820e5`에 커밋한 뒤 upstream을 fetch했다.
기준이c0에서 `a5e88ad68`로 변경되어 충돌 없이 리베이스했으며, 후보 커밋은
`870ca3e29347b053bed25bc27382e1099839ec4d`다. 새 기준에는 #7907 TTC SFNT 보존과
#7908 줄 구성 변경이 포함된다. 이후 비교는 새 기준의 producer/TSV로 갱신한다.
진단용 runner도 실행 시 `--base-ref`를 SHA로 고정하게 했고 과거 결과를 덮어쓰지 않는다.

리베이스 후 Native dev 빌드는2분14초에 성공했다. overfill43–46쪽 점수는 위의
source-flow 후보와 각각 동일했다. 44쪽의 추가 직접 판독에서 글줄 원점과 함께
하단 외곽도 약10px 늘어난 것을 확인했다. 따라서 원본 CENTER를 TOP으로 대체한
앞 후보의 가정을 제거하고 예약 높이 생산자를 재검토한다.

원본 row24/col6 마지막 앞쪽 줄은40960HU, 줄높이1000HU, 뒤 간격600HU이며
셀 상하 여백은566HU다. 합43126HU는 선언 셀높이43992HU와 일치하지 않는다.
`stored_plain_restart_cut(..., false)`는 이러한 재시작에서 **내용 컷**만 허용한다.
`stored_plain_cut_frame_minimum`이 그 판정으로 선언 셀의 빈 띠까지 예약한 것이
상위 `stored_child_cut_frame_extra` → 조각 flow/row override → 실제 셀 높이를 늘렸다.

빈 띠는 기존 원본 상자 닫힘 판정(`source_owned=true`)이 인증한 경우에만 예약하고,
선택한 실제 셀 상자의 원본 세로 정렬은 내용 컷의 계약으로 유지하도록 분리한다.
원점 clamp·쪽 번호 분기·허용치 변경을 추가하지 않는다. 이 후속 후보의 결과는
아직 미검증이며 dev 빌드와 영향 쪽 직접 비교를 먼저 진행한다.

첫 후속 후보 Native에서 row24/col6의 실제 셀 높이는586.5→575.0px로 바뀌었고,
44쪽 점수는85.14566→87.97724%였다. 나머지 선택43·45·46쪽은 각각 동일했다.
명시적 `end_row_height_override`가 없는 재귀 자식도 선택 컷 높이로 실제 상자를
만들지만, 정렬 소유의 조건은 override만 요구해 원본 CENTER를 소비하지 않았다.
실제 종료 컷에서 원본 재시작과 일치하는지로 판정하도록 연결했다. 뒤 꼬리 내용을
정렬 높이에 포함하지 않고 선택된 글줄만 소비한다. 이 최종 연결은 검증 대기다.

리베이스 후 도구 계약 검사: Python PNG·workflow24개 PASS, HTTP 공급4개 PASS,
실제 브라우저 glyph1개 PASS(영문 공백/한글 family 모두). 이는 도구 계약 결과이며
19문서835쪽 렌더링 완료 증거가 아니다. 새 기준 Native/fresh WASM producer를
만들기 위한 읽기 전용 detached sparse worktree는 `output/.../base-a5-source`다.
Cargo 대상은 기존 `/home/tsjang/rhwp/target/pr-review`를 순차 재사용한다.

## 최종 CI와 기록 PR 경로 — 사용자 지정

모든 필수 회귀·lint·Native/fresh WASM 전쪽 검증 및 직접 판독 완료 후, PR 생성 직전에
upstream/devel을 다시 fetch해 검증 기준 HEAD를 대조한다. 사용자는 HEAD가 변경된
경우에만 현재 변경을 커밋하고 리베이스한 뒤 새 통합 소스에서 재검증하도록 지정했다.
그 검증까지 완료한 뒤 코드 PR을 생성하고 실제 code head CI를 모니터링한다.
사용자는2026-10-15에 PR review·오늘할일을
옵션2의 **별도 문서 PR**로 처리하도록 지정했다. 코드 PR에 trailing 기록 커밋을
추가하지 않는다. 코드 PR 통합 후 최신 devel을 기준으로 새 브랜치를 만들어 기록만
담는 PR을 생성하고 review-only required checks를 실제 확인해 통합·후속 처리한다.
검증 완료 전 PR 준비 완료·승인·이슈 해결로 보고하지 않는다.

## 48쪽 Native 통과와 마지막 줄간격의 독립 판독

`source-cut-alignment-dev` Native 전쪽은48/Print48쪽, 누락·90% 미달0쪽,
최저43쪽90.86416%였다.44쪽99.92289%,45쪽99.95610%,46쪽99.97079%다.
원시 TSV·manifest는 해당 phase에, immutable producer/hash/patch는
`source-cut-alignment-dev-proof.json`에 있다.43–46쪽 review PNG는 같은 전쪽
실행의 PNG를 canonical 합성 함수로 재사용해 `source-cut-alignment-reviews-native`에
만들었다. export를 다시 실행하거나 SVG를 생성하지 않았다.

44쪽 PCB 조건의 시작·줄 간격·중앙 정렬·하단 괘선이 독립 PDF와 대응한다.
43쪽은 앞 후보93.05352%보다 낮아졌으며 직접 판독에서 끝 행의 비고 셀 글줄이
4px 내려간 것을 확인했다.90% 통과로 면제하지 않고 후속 원인을 보정한다.
원본 row21/col8의 앞쪽 마지막 잉크 끝은11200+1000HU, 상하 여백566HU다.
합12766HU는 선언 셀12476HU보다 크다. 실제 선택 유닛의 잉크 끝도 이 원본과
일치하지만 예약은 다음 프레임의 전진 간격600HU까지 포함해 셀을178.2px로 늘렸다.
원본 잉크·여백에 맞게 확장한 상자를 쓰고 종료 줄간격은 제외해야 한다.
반례인 row24/col6은 같은 잉크·여백 합42526HU가 선언43992HU 안에 있으므로
이 조건을 적용하지 않는다.44쪽의 이미 확인한 정렬 공간을 없애지 않는다.

이 후속 분기는 원본 재시작·선택 유닛 누적 잉크 일치·선언보다 큰 잉크를 모두 확인해
마지막 줄간격만 기존 공통 컷 높이 helper에서 제외한다. clamp로 글줄을 옮기거나
내용을 숨기지 않는다. Native 재빌드·전쪽 확인 및 fresh WASM은 검증 대기다.

공유 target에서 다른 worktree를 빌드한 뒤 main 빌드가0.42초에 완료되면서
기준과 같은 실행 파일 SHA를 남긴 것도 확인했다. 이 산출물은 후보 증거로 사용하지
않았다. 소스 내용을 바꾸지 않고 main의 lib/bin 입력 mtime을 갱신해 실제 재컴파일을
확인했다(3분11초). 기준/후보 producer를 각각 복사하고 파일 SHA가 다른지 확인한
뒤 위 전쪽 sweep을 실행했다. 공유 target을 삭제·초기화하지 않았다.

마지막 줄간격 보정의 Native 빌드는1분08초에 성공했다. 새 producer
`source-frame-tail-dev-rhwp`의 overfill 전쪽48/Print48은 미달·누락0쪽이며
**최저37쪽97.27881%**,43쪽99.00924%,44쪽99.92289%,45쪽99.95610%,
46쪽99.97079%다. 앞 후보의43쪽 하락을 복구했고44쪽 개선도 유지했다.
같은 source/patch의19문서 전쪽 Native는 `candidate-a5-frame-tail-dev/native`에서
계속 진행 중이다. fresh WASM dev 빌드도 실제 재컴파일3분27초에 성공했으며
같은19문서 전쪽 비교를 시작했다. 완료된 전쪽 결과와 진행 중 범위를 구분한다.

최신a5 기준 Native19문서도 완료됐다.16통과/3미달 또는 쪽수 불일치로,
rowbreak HWPX15쪽87.45557%, nested HWP104쪽89.87550%, overfill49/Print48이다.
기준 fresh WASM 전쪽과 후보 양쪽의 완료·비교·직접 판독·관계형 회귀 추가·최종
Rust 전체 회귀 및 lint는 아직 남아 있다.

후보 Native19문서835쪽도 완료돼 모두90% 이상·쪽수 일치·미달/누락0쪽이다.
하지만 최신a5와18문서787쪽 Decimal 비교에서는 nested HWPX104쪽이
99.39010→93.84310%로5.54700%p 하락했다. overfill은 기준49쪽 때문에 비교
불가로 별도 기록했다.90% 통과만으로 무회귀·PR 준비 완료라고 판정하지 않는다.

추적 결과 이어받기 helper가 위·아래 여백을 모두 더하는 동안, 생산자는 이미
마지막 유닛의 `terminal_band = om_bot + spacing_after`에 아래 여백을 담고 있었다.
`nested_table_split_from_cut_units`의 합 → 컷 요구 높이 → `split.flow_height` →
`para_y`의 소비 경로에서 아래 여백이 중복되어104쪽 후속 그림이512.9→516.7px로
밀렸다. HWPX의 기존 표 원점83.1px 자체는 바뀌지 않았다. 형식별 예외를 추가하는
대신 시작 띠에서는 위 여백만 예약하고 종료 띠의 아래 여백 소유는 생산자에 둔다.
이 후속 수정은 Native/fresh WASM과 같은 기준의 전쪽 비교를 다시 확인한다.

시작 띠의 중복 제거 후 Native104쪽은 HWP/HWPX 모두99.66092%로 올랐다.
이어 실행한 `candidate-a5-leading-band-dev` Native19문서835쪽도 모두90% 이상·
쪽수 일치·누락0쪽으로 완료했다. 비교 가능한18문서787쪽의0.01%p 이상 하락은
0쪽이다. overfill은 여전히 기준 쪽수 불일치로 별도 기록하며 자동 무회귀 통과로
바꾸지 않는다. 최종 Rust 검증 및 현재 family 후속 후보 검증은 아직 미완료다.

한편 앞 후보 `candidate-a5-frame-tail-dev` fresh WASM19문서835쪽은 모두90% 이상이지만
기준 대비0.01%p 이상 하락77쪽을 검출했다. 여기에는 위104쪽 여백 결함 외에 본문
글꼴 변경이 있었다. chemical3쪽의 PDF 본문 face는 휴먼명조이고, base/candidate PNG
차이는 꼬리말이 아닌 본문 y243..899에 분포했다. 도입한 Bold 별칭 우선 선택이
실제 설치한 휴먼명조를 HCR Batang으로 교체하고 있었다. SVG의 파일 탐색용 fallback
후보를 Canvas의 실제 글꼴보다 앞세울 수 있다는 가정을 제거한다.

원래 Canvas family 순서와 동일한 실측/paint 경로로 복원하고 FontFace raw-family
등록 수정은 유지했다. 불필요해진 Bold 별칭 이동도 되돌렸다. 이 후보의 fresh WASM
빌드는2분10초 재컴파일로 성공했다. 실제 글꼴 등록을 고치기 전의 관측을 근거로
본문 family를 교체했던 보정이 필요했는지는 chemical3·29·100쪽에서 먼저 확인한다.
다른 font 이름 조건을 붙여 대상을 좁히거나 점수 허용치를 완화하지 않는다.

원 family만 사용한 선택 쪽에서는 chemical3쪽98.13497%로 기준을 그대로 재현했지만,
29쪽85.61151%·100쪽91.02041%여서 전체 해결로 채택하지 않았다. 실제 요청한
family가 있을 때는 보존하되, 없을 때 실제 Bold 후보를 찾는 fallback은 필요하다.
별칭을 원 family보다 앞세운 공통 가정을 제거해 **요청 family → Bold 대체 후보 →
기존 나머지 체인** 순서로 실측·paint를 연결했다. 이름별 예외는 추가하지 않았다.

이 후보의 첫 WASM 빌드는 코드 오류가 아니라 rlib archive의 디스크 부족(error28)으로
실패했다. 실행 중인 Cargo/Rust와 sweep이 없음을 확인하고 종료된 `font_resources`
복사본을 SHA-256으로 중복 정리했다. 실제 바이트가 다른1313개 프로그램은
`normalized-font-programs`에 hardlink로 보존하고,4272개의 임시 경로를 정리해
할당 공간8.75443GiB를 확보했다. 원본8개 글꼴 경로·TSV·JSON catalog·로그·PNG·
producer package와 공유 target은 삭제하지 않았다. 이전 catalog의 경로와 보존
프로그램 SHA/경로 대응은 `deduplicated-font-resources-20261015.jsonl`에 있다.
공간 확보 후 동일 후보 빌드를 다시 실행하며 이 환경 실패를 회귀 재현으로 세지 않는다.

## 2026-10-15 후속 검증: 글꼴 교체 가정 철회와 정식 회귀 추가

PR 직전에 `upstream/devel` head를 다시 확인한다. 변경됐을 때만 현재 단계를 커밋한 뒤
rebase하고 새 통합 소스를 재검증한다. 모든 필수 검증이 완료되면 code PR을 생성하고 실제
CI를 모니터링한다. code PR의 CI 완료·merge 뒤 새 branch에서 **옵션2의 별도 docs-only PR**로
개별 PR review·오늘할일을 처리하고, 그 PR의 CI·merge·후속 정리까지 진행한다.
이 단계에는 아직 remote push·PR 생성·승인이 없다.

### 원 face·별칭을 앞서는 Bold 대체는 채택하지 않음

`primary-before-bold-fallback-dev` 선택 쪽은 chemical3쪽98.13485%,29쪽98.05825%,
100쪽98.00000%였다. 그러나 원 이름 뒤에도 동일 face의 설치 별칭보다 HCR Bold를
앞세웠으므로 rowbreak HWPX의9개 하락은 그대로였다. 예:2쪽99.71263→99.55553%,
13쪽94.52919→93.96166%. 단순히 primary 이름 하나만 앞세워도 원 face 보존이 된다는
가정을 철회했다. `canvas_text_font.rs`, `web_canvas.rs`의 family 체인과 SVG의 별칭
정의 위치를 a5 기준과 동일하게 복원했고, 잘못 도입한 공통 별칭 모듈을 제거했다.
FontFace의 실제 family 등록 수정과 real-browser 검사는 유지한다.

원본 두 문서에서 `한양중고딕`은 HFT 선언이었다. 독립 PDF Type3 프로그램의 공통
Unicode115개 중114개는 실제 CharProc 바이트가 같았다. 글꼴 이름만으로 다른 Bold face를
선택하는 보정은 이 근거를 대신하지 못한다. 기존 SVG와 같은0.02em 합성 획의 진단용
브라우저 실험도 수행했다. rowbreak2쪽99.71234%,13쪽94.55224%였지만 chemical29쪽
80.35714%,100쪽88.78505%여서 채택하지 않았다. 이 실험은 ignored probe에만 있으며
제품 코드나 최종 시각 통과로 보고하지 않는다.

원 family·별칭 순서를 보존한 현재 소스로 fresh WASM dev/no-opt 빌드를 완료했다
(실제 컴파일1분19초, wrapper2분15초). producer·source patch·새 검사 SHA는
`preserved-original-aliases-dev-wasm-proof.json`에 있다. 같은 기준19문서 전쪽 WASM을
`preserved-original-aliases-dev`로 다시 측정 중이다. 이는 진단 빌드이며 최종 release 검증과
구분한다. 기존 chemical의 쪽번호 미달 및 nested HWPX104쪽의 기준 대비 미세 하락은
아직 해결됐다고 판정하지 않는다.

### `tests/cases/issue_7902_original_frame_and_content.rs`

독립 Print와 `candidate-a5-leading-band-dev`의 전쪽 Native/fresh WASM 시각 증거가 있는
overfill·nested 두 형식에4개 검사를 추가했다. 화면 위치는98% 이상을 확인한43·104쪽에만
독립 PDF 좌표와±2px를 사용하고, 별도로 원본 내용·소유·순서·포함·가운데 정렬을 검사한다.

| 검사 | 독립 기대값·보호 의미 | 실제 수정 전 | 실제 수정 후 |
| --- | --- | --- | --- |
| overfill 내용 컷 | Print48쪽,43→44쪽 `기인/한다.`,44→45쪽 `있/다.`, 두 조각의 합=원본 셀 전체 | 최종 전체 회귀에서 base 대조 추가 예정 | PASS |
| overfill 닫힌 프레임 | Print43쪽 마지막 괘선1053.886637px±2, 셀21/6·21/8 및 정상 대조24/6의 위/아래 여유 대칭·줄 포함 | `source-cut-alignment-dev-rhwp`: 실제1063.5px로 의도한 assertion FAIL | `continuation-leading-band-dev-rhwp`: PASS |
| nested HWP104쪽 | Print115쪽,14×2표→그림 순서, 표 위82.949300px·그림 위514.157333px±2 | `source-frame-tail-dev-rhwp`: 그림516.7px로 의도한 assertion FAIL | PASS |
| nested HWPX104쪽 | 같은 원본 대응과 별도 Print PDF의 동일 배치 계약 | 같은 이전 producer: 그림516.7px로 FAIL | PASS |

같은 정식 Rust 검사 소스를 기존 rlib·serde_json에 연결해 focused test executable로 실행했다.
수정 후4/4 PASS(116.49초), 수정 전 마지막 줄간격1/1 FAIL(28.31초), 중복 아래 여백2/2
FAIL(37.64초)이다. source 차이 대신 `CARGO_BIN_EXE_rhwp`로 SHA가 보존된 이전/이후 CLI를
선택했으며 실패는 빌드·환경 문제가 아닌 위 배치 assertion이다. 이 실행은 Cargo generated
suite 및 최종 전체 nextest·세 Clippy lane의 완료를 뜻하지 않는다. 로그는 각각
`frame-content-after.log`, `frame-content-before-trailing-space.log`,
`frame-content-before-double-bottom.log`다.

104쪽 Native/fresh WASM의 표 끝·후속 문단·그림·그림 뒤 문단을 독립 PDF와 직접 읽었다.
각 backend에서 HWP/HWPX PNG SHA가 각각 같아 같은 출력임도 확인했다. 현재 review·overlay는
`nested-current-reviews/{native,wasm}/{hwp,hwpx}/`에 있으며,0기반 overlay API를 잘못 호출해
처음 패널에105쪽으로 적힌 진단 이미지는104쪽으로 다시 생성했다. 측정·원본 쪽은104쪽이다.

종료된 임시 `font_resources`144개 경로도 같은 SHA의 보존 프로그램으로 중복 정리해
추가0.42714GiB를 확보했다. 첫 ledger를 덮어쓰지 않았으며 v2 ledger·결과에 분리했다.
원본 문서/PDF·8개 공급 font 경로·TSV·PNG·producer와 공유 target은 보존했다.

### HFT 자동 번호 기준선과 Native의 공통 글리프 폭 소비

원 face를 바꾸지 않고 chemical29쪽의 자동 번호를 진단했다. 독립 Print의 HFT Type3
프로그램은 FontBBox `[0 -200 1000 1000]`이며12pt 번호의 실제 원점은
`(294.359985,797.039978)pt`다.100쪽도 같은 y다. 원본 영문 슬롯은 HFT이고,
기존 `footer_page_number_y`는 프로그램 종류와 무관하게 크기의1/3을 기준선 여유로 뺐다.
0.2em 하강 메트릭으로 예상한 기준선은1062.613333px이고 독립 PDF는1062.719971px다.
TTF 대조군 rowbreak/7336의 HCRDotum 기준선은1061.559896px이며 기존 경로를 유지한다.

원본 정상 저장본의 font type → `ResolvedStyleSet.page_number_font_is_hft` → 자동 번호의
공통 `footer_page_number_y` → 테두리 유무 분기의 최종 TextRun 기준선으로 연결했다.
문서 ID·글꼴 이름·쪽 번호로 분기하지 않으며, 대체 family를 원본 HFT 프로그램 종류의
대용으로 쓰지 않는다. 머리말 번호의 기존 배치와 명시적 저장 LineSeg 경로는 이 변경에
포함되지 않는다. 일반 꼬리말·테두리 분기는 같은 기준선을 소비한다.

WASM 실측은 chemical3쪽98.13497%로 기준 본문을 유지했고,29·100쪽은 각각100%였다.
그러나 기준선만 고친 첫 Native는29쪽90.64748%로 낮아졌다. Native가 설치 별칭을
건너 generic을 선택하고, Canvas의 글리프 폭 맞춤도 소비하지 않는 두 경로를 확인했다.
Native도 기존 `installed_render_font_aliases`의 원 face 순서로 프로그램을 찾게 했고,
저장/측정 전진 위치는 유지하면서 `glyph_fit_positions`와 `canvas_cluster_fit_scale`의
커닝 전 폭 계약을 소비하도록 연결했다. 별칭만 고친 중간 후보는29쪽87.95181%여서
완료로 채택하지 않았다. 글리프 실측 폭을 바꾸지 않은 채 숫자·구두점을 그대로 그리는
Native의 가정을 제거한 후 새 Native는3쪽97.72264%,29쪽98.47328%,100쪽99.10314%다.
전쪽 무회귀를 뜻하지 않으며 새 원본19문서 Native/fresh WASM 측정을 시작했다.

현재 진단 producer는 `source-hft-baseline-aliases-and-fit-dev-rhwp`와
`source-hft-baseline-dev-wasm`이다. WASM은 HFT 기준선 수정 뒤 실제 재빌드했고 이후
Skia 전용 변경은 WASM cfg에 들어가지 않는다. 최종 제출용 정확한 code head의 release
빌드·WASM과 필수 lint/전체 nextest는 별도 완료해야 한다. 현재 전쪽 cohort 이름은
`candidate-source-hft-and-shared-glyph-dev`다.

원 별칭 복원만 적용한 이전 전쪽 진단은4개 chemical 변형의29쪽85.61151%를 실제 확인한
뒤 HFT 기준선 수정으로 대체했다. 남은 이전 source의 비교 작업·하위 프로세스만 정확히
종료했고 결과를 `superseded_by_source_hft_baseline_change`로 보존했다. 전체 완료나
통과로 바꾸지 않았으며 Cargo·새 source 검증은 종료하지 않았다.

nested HWPX104쪽 WASM은 여전히99.87312%로 기준99.93539%보다0.06227%p 낮다.
비슷한 점수만으로 해소하지 않는다. 추가 불일치35픽셀 중25픽셀은 y880대의 **그림 뒤
문단**에 있어 그림 resampling만의 문제로 단정할 수 없다. 진단용 high quality는
99.87261%, nearest는99.86552%여서 두 후보 모두 제품에 적용하지 않았다. 현재 source의
표→그림→뒤 문단 흐름과 독립 PDF의 실제 기준선을 더 대조한다.

추가 기존 검사에서도 a5 기준의49쪽/Print48쪽 때문에 원본 내용 컷 검사가 실제 FAIL
했다(`frame-content-before-original-count.log`,38.05초). 위 표의 수정 전 칸은 이제
미실행이 아니라 **쪽수 assertion49≠48의 실제 실패**로 보완됐다.

공간 확보도 현재 작업과 분리해 기록했다. 사용 중이 아닌 폐기된 진단 CLI7개만 삭제해
2.83658GiB를 확보했다. 기준·실제 before/after 회귀용 producer와 모든 proof/TSV/PNG는
유지했다. 또139개의 완료된 WASM export의 실제 적재 프로그램194개 SHA를 감사해,
한 번도 적재되지 않은 정규화 중간 프로그램1244개만 제거해3.57598GiB를 확보했다.
실제로 적재한 프로그램, 원본 공급 글꼴, catalog/ledger·이미지·공유 target은 보존했다.
기록은 `obsolete-diagnostic-cli-cleanup-20261015.json`,
`normalized-font-program-use-audit.json`,
`never-loaded-normalized-font-program-cleanup-20261015.json`에 있다.


### 전쪽 대조에서 폐기한 후보와 자동 번호 한정 재검증

`candidate-source-hft-and-shared-glyph-dev`는 두 backend 모두19문서835쪽의90%·쪽수
조건을 통과했다. 그러나 a5와 비교 가능한18문서787쪽에서 Native191쪽이0.01%p 이상
하락했다. Native 전체 본문에 설치 별칭과 글리프 폭 맞춤을 적용한 후보는 채택하지 않는다.
WASM도 nested HWPX104쪽99.93539→99.87312%가 남아 있어 PR 승인 조건을 충족하지 않는다.
overfill은 base49쪽/Print48쪽이므로 같은 전체 페이지 집합으로 비교 가능하다고 보고하지 않는다.

Native의 본문 face 선택·자연 글리프 폭은 a5 경로로 복원했다. 저장 글줄이 없이 생성되는
HFT 자동 번호에만 원 face의 설치 별칭과 원본 전진폭을 적용하는 후보를 별도 검증한다.
첫 제한 후보는 별칭을 누락해 chemical29쪽90.00000%,100쪽93.91304%였다.
일반 본문까지 다시 바꾸지 않고 생성 번호의 원 face 탐색 누락을 보완한다.
현재 결과를 최종 제출 head의 전체 검증으로 보고하지 않는다.

### Snap Chromium 프로필 누수 수정·실행 증거

Puppeteer가 host `/tmp`에서 만든 프로필 경로와 Snap Chromium의 private `/tmp`가 달라,
브라우저가 실제로 쓴 프로필이 종료 뒤 남는 것을 확인했다. 출력 폴더 아래에 실행별 전용
프로필을 만들고 Chrome 종료 뒤 제거하도록 수정했다. 시작 실패 경로도 실제 exporter CLI에
존재하지만 Chrome이 아닌 Node 실행 파일을 주입해 검증했다. 원본 문서 파싱은 이 fault
검사에서 실행하지 않는다. 실제 Canvas 글꼴 검사와 시작 실패 검사2개 PASS(4.744초),
HTTP 서버 검사4개 PASS(0.637초)다.

동일 입력·WASM·글꼴의 nested HWPX104쪽을 실제 새 프로필 경로로 캡처했다.
수정 전후 PNG SHA256이 모두
`5314795c95d164342e706849b8ac9db1b98613272c11347dfb6cb8a1b19676ae`로 같고,
작업 후 전용 프로필이 남지 않았다. 캡처 방식·픽셀·폰트 선택은 바꾸지 않는다.

Snap private 임시 경로에서 동일 UID·미사용·loopback RHWP 리소스 캐시가 확인된 오래된
프로필1105개만 감사 후 제거해96,984,805,376바이트(약90.3GiB)를 확보했다.
사용 중이거나 RHWP 소유 증거가 없는5개 프로필은 보존했다. 원본 HWP/HWPX/PDF·글꼴·
공용 target은 삭제하지 않았다. 감사·두 단계 정리 JSONL/결과는
`snap-rhwp-browser-profile-*-20261015*`와 v2 ledger에 남긴다.


HFT 쪽번호의 정식 검사2개도 같은 소스로 실행했다. 이전 producer
`continuation-leading-band-dev-rhwp`는 HWP/HWPX 모두 실제 기준선1060.48px로 FAIL
(2/2,8.54초), 제한 후보는 기존4개 검사와 합쳐6/6 PASS(67.57초)였다.
독립 Print29쪽의 실제 번호·원본12pt·진하게 속성과797.039978pt 기준선을 검사한다.
Native29쪽98.47328%,fresh WASM100%와 대표 review 직접 판독을 선행했다.
이 focused rustc 실행은 최종 Cargo 전체 suite 완료로 보고하지 않는다.

제한 후보의 전쪽 비교에서도 chemical Native 일부 쪽이0.01%p 이상 낮아졌다.
원본 H2GTRM 프로그램·같은 glyph 원점·폭의 작은 Skia 진단으로 실제29쪽 PNG를
바이트 단위로 재현했다. 기본 `Font`의 subpixel=false가 소수 x 원점을 정수로
반올림하는 것을 확인했다. 이 플래그만 true로 그린 진단은 독립 PDF 대비100%였다.
실제 소스에서는 원본 폭을 소비하는 생성 HFT 번호에만 소수 원점 보존을 적용한다.
본문·글꼴 프로그램·비교 기준은 바꾸지 않는다. 증거는 `number-raster-probe-result.json`과
각 PNG다. 새 Native producer·전쪽 비교로 실제 효과를 다시 검증한다.


`candidate-generated-number-dev` 전쪽 진단은 양쪽 모두19문서835쪽, 미달/누락0,
독립 PDF 쪽수 일치로 끝났다. Native 최저91.71770%(scattered37쪽),664.39초,
WASM 최저90.09386%(chemical 계열19쪽),963.22초다. 단, 이 Native는 소수 원점 보존
직전 후보다. 같은 a5 기준18문서787쪽 대조에서 Native chemical4종 각9쪽(총36쪽),
WASM nested HWPX104쪽1쪽의0.01%p 이상 하락을 실제 검출했다. 앞의90% 통과로
무회귀를 대신하지 않는다. 해당 비교 JSON·TSV는 모두 보존하며 PR은 생성하지 않았다.

일반 Clippy3분04초, WASM Clippy2분15초, workspace build3분47초가 통과했다.
전체 target·Native Skia Clippy와 전체 회귀는 아직 완료 전이다. 브라우저·HTTP의
최종 관련 Node 검사6/6 PASS(5.586초), source unit tier의 고정 base 비교도 PASS다.
추가한6개 회귀의 Cargo 정식 suite 실행은 남아 있다. 사용자 요청에 따른 커밋·upstream
확인 후 head가 바뀐 경우에만 리베이스하며, 최종 PR 직전에도 이를 다시 확인한다.


체크포인트`59a6c9d6c` 후 fetch한`upstream/devel`은 a5 그대로여서 리베이스하지 않았다.

최종 all-target Clippy의 lib-test 경로에서 `ResolvedStyleSet`의 완전 초기화6곳에
새 HFT 메타데이터가 누락된 것을 검출했다. 기존 테스트의 비 HFT 입력에는false를
명시해 컴파일 계약을 보완했다. source unit tier 및 세 Clippy lane을 새 소스에서
재확인한다. 이 실패를 런타임 회귀로 분류하거나 통과로 기록하지 않는다.

Roman 번호도 별도 진단했다. Native 기본 `Font`는 subpixel=false,baseline_snap=true였다.
원본의 소수 원점을 보존하려면 두 축 모두 반올림하지 않아야 한다.5쪽 번호만 바꿔 본
진단은 x만 보존했을 때99.17727%,기준선도 보존하면99.19634%(a5는99.19277%)였다.
이 합성 교체 이미지는 진단 근거일 뿐 정식 전쪽 측정으로 세지 않는다.
실제 Native는 생성 HFT 번호에 subpixel=true,baseline_snap=false를 적용해 재빌드했고
1분42초에 성공했다. 이 producer로 원본19문서 전체와 a5 비교를 다시 실행한다.


### 생성 번호의 Native 진단과 paint 경로 일치

코드 대조에서 생성 HFT 번호의 Native paint에만 설치 별칭이 추가되고 진단 observer는
원래 family만 받은 것을 발견했다. a37의 실제 Native 라이브러리를 분리 보존해 같은
정식 테스트로 재현했다. 실제 원본29쪽의 후보2번째가`Noto Sans KR`로 나와
`HY중고딕`을 기대한 검사1개가 의도한 원인으로 FAIL(4.89초)했다.

paint의 기존 문자열 생산식을`native_font_family_for_style`로 옮겨 Native paint와
읽기 전용 진단이 같은 결과를 사용하게 했다. 일반 본문은 원 family를 그대로 빌린다.
새 검사는 HWP/HWPX 두 원본의 생성 번호를 실제 Native renderer-bound API로 조회한다.
새 fixture·사유 불명의 좌표·baseline 완화는 추가하지 않는다. Native CI의 두 profile에
이 source를 추가했고 classifier의 Native 입력 목록도 맞췄다. 수정 후 workflow186개,
CI Node328개·actionlint는 통과했다. Native 실제 after 검증은 이어서 수행한다.

직전 전체 nextest는723.57초 후 중단됐다. 마지막 컴파일 관측 뒤 실행 단계가 시작되어
최종 로그는2425/10301실행·2417PASS·8SIGINT·7876미실행·45skip이었다.8건은
실제 구현 실패로 분류하지 않는다. 중단 사유와 프로세스 소유 확인은
`a37-full-build-superseded-for-font-trace.json`에 남겼다. 새 소스의 전체 검증으로 대체한다.

before worktree가 공용 target의 해시 없는`librhwp.rlib`를 덮은 뒤 main에서 Cargo가
Fresh로 판정한 캐시 불일치도 실제 확인했다. 첫 after 시도는 before와 같은 RLIB 바이트로
실행되어 완료 증거로 사용하지 않는다. 공유 target을 삭제하지 않고 main의 entrypoint
mtime만 갱신해 실제 현재 라이브러리를 재빌드하고 해시·실제 API 결과를 확인한다.


현재 라이브러리의 SHA가 before와 다름을 확인한 실제 after 실행은 HWP/HWPX 두 입력
조회 모두 통과했다(정식 새 테스트1PASS,9.46초). 로그는
`font-trace-after-refreshed-test.log`이며 이전1FAIL은`font-trace-before-test.log`다.
새 정식 테스트는 Native feature에서 동작하므로 CI의 Native lane에도 명시했다.
이제7개 검사 중6개는 기본 전체 회귀,새 진단1개는 Native lane에서 보호한다.
이 source 보완은 실제 paint의 기존 후보 생산식을 공통 helper로 옮긴 것이며
글리프·좌표·폭을 새로 변경하지 않는다. 새 source에서 필요한 검증을 이어간다.


### f5284cd0b의 실제 공용 패키지와 정식 Cargo 검사

진단 보완·Native CI 배선을`f5284cd0b`에 커밋했다. 저장소 루트 wrapper와 공용
`target/pr-review`,jobs8로 fresh dev/no-opt WASM을 재생성했다(2분41초).
`pkg`와`rhwp-studio/public`의 JS/WASM SHA가 같음을 확인했고,오래된 타입 선언 때문에
실패했던 표준 Studio tsc를 다시 실행해 PASS했다. 생성된 public wrapper는 검증 산출물이며
이 변경에 임의로 stage하지 않는다. 불변 producer와 source/해시는
`final-f528-producers-proof.json`에 보존했다.

Native feature의 정식 Cargo source selector로 새 회귀7개가 모두 PASS했다.
실행95.92초,debug test/CLI 컴파일8분27초이며 Native 글꼴 진단 검사도 포함한다.
로그는`final-f528-native-formal-seven-dev.log`다. 첫 selector 호출의 harness 인자 배치는
잘못되어 전체 target 선택으로 해석됐으므로 중단·보존했고,올바른 target selector와
`RUST_TEST_THREADS=4`로 다시 실행한 위7PASS만 정식 결과로 사용한다.
현재 f528의 Native/fresh WASM19문서 전쪽 측정과 최종 lint/전체 회귀를 이어간다.


### 최종 전체 회귀에서 검출한 실제 실패와 기존 검사 감사

f528 전체 nextest는 컴파일·실행1414.57초, 실제 테스트772.154초에 종료됐다.
10301개 중10300PASS·1FAIL·45skip이다. 실패는
`body_overflow_baseline::body_overflow_does_not_grow_partition_14`의
정책연구 HWPX이며, 본문 바닥2px를 넘는 건수가1→2로 증가했다.
`final-f528-full-nextest.log`와`final-f528-gates-result.json`을 보존한다.
종료된 runner 대신 남은 Native feature·doctest만 순차 실행하는 별도 runner를 시작했다.
Native lib 전체는 exit0이며 나머지는 완료 전이다.

같은 원본의 기준a5와 현재f528을 같은2024 Print·96dpi·글꼴로 직접 비교했다.
기준162/214쪽은12.29880%/59.36872%,현재30.21663%/34.15727%다.
기준과 현재 모두215쪽이다. 기존 #7445 하위 #7753에서 같은 원본 전체 피델리티를
추적 중인 것도 실제 API로 확인했다. 그러나 이번 흐름 끝 전달이 아래 바깥여백283HU
(3.77333px)를 paint에 반영하면서214쪽 후속 표의 넘침이1.78667→5.56px로
늘었다. 기존 이슈 등록만으로 새 영향을 숨기거나1→2로 baseline을 완화하지 않는다.
직접 비교 PNG·render tree·fit/paint trace는`liver-failure-review/` 및
`liver-{flow,pagination}-audit.log`에 보존하며 구현 원인 검토를 계속한다.

현재 보정 대상의 기존 검사도 경로로 따라 감사했다. 원본HU·셀 padding·글자크기에서
계산한 상대 계약과 이미 포함/순서 검사였던 항목은 보존했다. 실제 교체·보완은 다음과 같다.

- #7336:97%대인5쪽의 bbox 상단1048..1049.5 핀을 독립 Print에서 같은
  기준선인2·5쪽 자동 번호의 정렬 관계로 교체했다.95%대인1쪽은 독립 Print의
  꼬리말 하단 여유/원본 글자 em 비율0.322±0.045로 검사한다(기존 약0.6px 정밀도).
- chemical16쪽:97%대의974.72/1020.32 절대 baseline을 원본 문단149/150의
  저장LineSeg 원점·baseline과 실제 본문 원점의 관계로 교체했다. 원본 크기·줄 순서를 보호한다.
  검토 범위 밖 reduced 입력의 기존 기대값은 유지한다.
- chemical57쪽:Native99.08734%·WASM99.09650%,표 하단·꼬리말과 원본 내용을
  직접 확대 대조했다. 기존 독립 Print1006.6±1px 핀을 유지하고 본문 포함·꼬리말 비침범을 보완한다.
- chemical77쪽:Native99.97569%·WASM99.97568%,같은 영역4배 확대에서 CCC 이미지와
  셀 경계를 직접 대조했다. 기존 원본 소유·참조 검사를 유지하며 독립 Print235.903px의
  공차를3→2px로 좁혔다. 미확인 과거2.8px 오차 주석은 제거했다.
- rowbreak HWPX:첫 본문의 임의90px 영역과10쪽 표의120px 영역을 원본HWP와의
  본문 상대 원점·제목→표 간격 관계로 교체한다.10쪽 제목 한 줄 높이로 간격을
  추정한 첫 진단은 실제 빈 줄 점유를 빠뜨려 FAIL했으므로 채택하지 않았다.

같은f528 debug 라이브러리에 현재 테스트 source를 직접 컴파일한 선행 확인은
#1133 대상1PASS,#7336 대상1PASS,#7048전체6PASS,#6782전체2PASS,
rowbreak전체16PASS다. 이는 정식 Cargo/lint gate를 대체하지 않는다.
대표 확대는`ccc-audit/ccc-p77-4x.png`와`chemical-p57-bottom.png`다.
수용 근거인 전쪽 TSV는 위 f528 최종 원장을 재사용하며 생산 코드는 이 감사로 변경하지 않았다.

### 추가 Native feature 결과와 사용자 기준 재확인

Native lib는3859+15+165+2개 PASS,기존ignore12개다. 이후 Native CLI/그림/
직접 PDF/배경/차트/SIXEL 선택 source는 각각 exit0으로 끝났다.
`issue_4961_font_decision_trace`는8PASS·1FAIL이었다. 실패 검사가 원본`task-001`의
face를 시스템에서 찾지 못해 custom Noto fallback을 쓸 것이라고 가정하고 있었다.
이 호스트는 원본 face를 공급하므로 실제 선택은 system이었으며 custom inventory의
손실로 판정할 증거가 아니었다. paint와 observer의 기존 우선순위는 변경하지 않는다.

해당 검사는 배치 정답지가 아닌 renderer snapshot 계약이므로 입력 font table만
저장소의 실제 custom face`Noto Sans KR`를 요청하는 합성 대조군으로 설정했다.
`set_document`로 파생 상태도 재구축하고,custom 기록의 실제 resolved face와
`nativeGlyphCoverageObserved`를 함께 검사한다. 기존 입력·그림·시스템 font 설정은
수정하지 않았다. 보존한 debug 라이브러리로 수정 source를 직접 컴파일한 선행 검사는
1PASS(0.96초)다. Native 정식 Cargo 재실행 및 중단 뒤 남은 새7902/doctest는 대기다.

사용자는 쪽수 복구 예외 없이 **현행 전체 페이지 일대일 무회귀를 입증할 때까지 PR 보류**를
선택했다. overfill의 기준49/Print48을 승인 예외로 처리하지 않으며,.06227%p 하락도
해제되지 않았다. 머지·PR 생성·지침 완화는 수행하지 않는다.

요청에 따라 다시 fetch한`upstream/devel`은`a5e88ad682c805e1d845502ddc9521b94ea8eccd`로
변경되지 않았다. 따라서 불필요한 리베이스는 수행하지 않았다.

본문 넘침의 원인을 분리하기 위해 새로 추가했던 종료 흐름 전달9줄만 뺀 진단 producer를
공용 target에서 빌드했다.원본 source를 즉시 바이트 복원했고 진단 CLI를 별도 보존했다.
이는 채택한 구현이나 제출 head가 아니며 같은 조건의 rowbreak15쪽/nested104쪽/정책연구
본문 넘침과 비교해 해당 일반화가 필요한지 확인하는 실험이다. 명령·source diff·해시는
`flow-end-ablation-result.json`,실행 상태는`flow-end-ablation-run.log`에 있다.

### 조각 물리 잔여의 잘못된 생산 근거

9줄 제거 진단은 정책연구의 본문 넘침을 기존1건으로 돌렸지만,rowbreak HWPX15쪽도
기준의87.45557%로 되돌렸다. nested HWPX104쪽 Native99.66092%는 변하지 않았다.
그러므로 올바른 종료 흐름 전달을 삭제해 검사를 통과시키는 안은 채택하지 않는다.

정책연구 원본 표 id1937554103은 독립적인 원본 첫 프레임을 갖는다.
마지막 원본 줄의10000+1000HU와상하padding282HU가표 선언11282HU를 닫는다.
이어받는6..13유닛은 실제 합15000HU(200px),원본 마지막 줄간격은 유닛에서
이미 제외되어 있다. 따라서 종료 상자의 내용+padding은15282HU(203.76px)다.
독립2024 Print214쪽의 첫 표 높이는203.457px(595×841pt PDF)로 대응한다.

그러나 스캐너는213.333px를 수용했다. 종료 조각에 전달된 물리 하한이 내용 요구보다
9.57333px 컸다. 잔여 생산의 마지막 fallback이`cut_row_h`(측정으로 생긴 전체 내용 높이)
에서 앞의 독립 프레임 높이를 빼서,첫 프레임의 줄간격을 미소비 물리 공간처럼 넘겼다.
`cellSz`의 원본 최소 높이3948HU는 앞 프레임보다 작아 그러한 빈 공간을 입증하지 않는다.

수정 경로는`emit_table_fragment`의 잔여 생산→cursor.start_row_height_override→
`RowScanQuery::whole_row_height`의 내용과 물리 하한 max→scan.consumed→
`PageItem::PartialTable` 시작 행 override와실제 배치다. 이전에 실제 수용한 물리 잔여와
걸침 셀·인증된 프레임의 특수 생산은 보존한다. 마지막 일반 fallback은 측정한 내용 합 대신
원본 행의 선언 최소 높이에서만 빈 공간을 생산하도록 보정한다. 문서ID·쪽·임의 수치로 분기하지 않는다.

물리 잔여 보정 후보의 Native 빌드는2분02초에 성공했다.
기존 물리 잔여·가운데 정렬·중첩 이어받기22개와 글꼴 진단9개를 같은 정식 nextest
선택 바이너리로 실행해 **31PASS/0FAIL**했다(실행46.147초,dev).
Native CI의 두profile에 기존 #4961 진단 source도 명시했고 classifier 입력과 검사를
맞췄다. 관련Node45PASS,workflow35PASS,actionlint exit0이다.

선택 쪽 Native:rowbreak15쪽92.37322%,overfill43/44/45/46쪽
99.00924/99.92289/99.95610/99.97079%,nested HWPX104쪽99.66092%로
직전 보정 수치를 유지했다. 정책연구는215쪽을 보존하며 전체 본문2px 초과 넘침이
**2→0건**으로 줄었다(기준a5의 기존1건도 함께 해소).
`physical-tail-minimum-{producer,checks}.json`과`physical-tail-minimum-native-contracts.log`,
`physical-tail-minimum-liver-anomaly.log`가 근거다. 원본·PDF·baseline 기대값은 바꾸지 않았다.
새 producer의 정책연구162/214쪽 직접 비교와 현재 최종 head의 전체 Native/fresh WASM
19문서,전체 Rust 회귀·lint는 계속 확인한다. 현행0.01%p·일대일 전체 비교 gate는 유지한다.
