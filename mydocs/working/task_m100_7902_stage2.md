# #7902 2단계 — #7877 문서 회귀 보정과 재발 방지

Issue: #7902. 이미 merge된 PR #7877의 원본 19문서·835쪽 범위를 유지한다.
1단계 도구 PR #7903과 기록 PR #7906은 실제 merge됐으며 이 이슈는 OPEN이다.

## 기준과 승인된 진행

- 브랜치: `fix/pr7877-render-regressions-20261014`.
- 기준 `upstream/devel`: `c0d330737fabae77ca2bf1706cba8514d0e8f1df`.
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
