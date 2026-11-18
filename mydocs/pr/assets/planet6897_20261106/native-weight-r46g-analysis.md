# Native 굵기 기능 보존 분석

## 최종 프로그램·윤곽 검증 (2026-11-09)

현재 검증 code head는 `eb876d1416fb558c874f9efec65c5722a6769751`입니다. sample16 세 원본 192쪽과 nested 17쪽을 포함한 Native/fresh WASM 전쪽을 다시 측정했으며 정본 보호 TSV 공식 표시값 하락은 없습니다. nested 8쪽은 두 경로 모두 99.56%입니다. 원래의 윤곽·자연 전진·장식 소유를 분리한 공통 규칙을 실제 소비와 회귀로 확인했습니다. [최종 검토·측정 원문](final-b768bee1-20261109/integration-final-review.md)과 개별 PR의 대응 페이지 증적을 기준으로 판정합니다.

아래 R 단계는 보정 과정의 과거 관측입니다. 예전 99.48%·미완료·선택 실행을 현재 결과로 재인용하지 않습니다.


## R55 후속: 실제 전쪽 결과와 Native 프로그램 선택 경계

제품 source `96805fee45853296fc4d6f3689f98e7bd125014f`의 Native 실행 파일
`4d27641f7133d13f40d62231801e72c274a2613e79c921bdf967baa66829690a`로
sample16 3종 각각64쪽을 다시 출력했다. HWP2022는64쪽 모두100.00%, HWP2024·HWPX는
각 최저99.99%다. PR #8389·PR #8397의 표시값192행보다 낮아진 쪽은0쪽이다.
정식 원문은 [Native TSV·생산자와 범위](program-ink-r55/validation.json)에 연결한다.
기존 집중66건은 모두 PASS지만, 새 legacy 검사는 suite 준비 전이라 그 실행에는 수집되지 않았다.
생성기를 실행해 정식 수집에 편입했으며, 실제 실행 확인은 다음 집중 검사에 포함한다.
이를66건 안에 이미 통과한 것으로 계산하지 않는다.

nested의 이전 R43 producer를 지금 다시 실행해 같은 기존 환경과 승인된 Gulim만 적용한 환경
모두에서8쪽99.49%를 재현했다. 현재 R54는같은 기존 환경에서99.48%다. 이전 재측정 설정은
새 자동 목록으로 바꾸면서 승인된 휴먼명조 대체 외에 휴먼고딕까지TTF로 바꿨다.
기존 PR의 다른 원본 프로그램 선택을 보존한 설정을 별도로 만들었고, 변경 픽셀은60,410개에서
536개로 줄었으나 제목의 `< >`에 새 미대응13개가 남았다. 설정 수정만으로 완료하지 않는다.

실제 Native의 같은 `<`에서 전진8.693333·자연 폭12는 같지만 잉크 폭은 이전8에서 현재7로
바뀌었다. 원 PR이 선택한 `MalgunGothic-Regular`와 다른 같은 family의 TTF 버전이 새 목록
열거로 선행하는 경계를 확인했다. 같은 형식의 선택은 OS의 원래 결과를 보존하고, 형식 우선순위
TTF→TTC→OTF는 형식이 달라질 때만 적용한다. 같은 family의 결측 글리프 공급 후보는 계속 남긴다.

시스템 face 재개방도 원 프로그램과 같은 glyph ID·무힌팅 윤곽·자연 전진이 실제로 일치하는
정적 단색 TTF에 한해 원래 face의 raster 설정을 유지한다. CFF·TTC·가변·색상 프로그램과
불일치 glyph는 기존 재개방을 유지한다. 문서 ID·목표 좌표·특정 글자·점수에 따른 분기는 없다.
이 후속 제품 변경의 컴파일·실행 및 실물 재측정은 아직 수행 전이다.

Native 실제 프로그램 선택의 새 합성 검사는 독립 FontMgr가 선택한 MIT fixture를 명시 공급한
출력과 시스템 출력을 비교해 수정 전 의도한 단언FAIL을 확인했다. 검사는 기존 CI가 호출하는
`native_fractional_glyph_origin`의 내부 모듈로 구현해 새 workflow 호출을 늘리지 않는다.
외부 helper의 path 속성은 별도 integration target을 추가해 수집 예산을 초과하므로,
기존 CI 진입점 내부로 옮겨 자동 수집과 실행 경로를 함께 보존했다. `RHWPHostFixture-Preferred.ttf`는 원래 MIT 합성 glyph의 단위·버전·이름만 바꾼
자체 자료다. 기존 생성기의 `preferred_face()`로 바이트 동일 재생성을 확인했다.

실행 로그와 원인 분리 원본은 ignored output의 `r55-nested-original-producer-replay.json`,
`logs/r55-nested-fit-r43.log`, `logs/r55-nested-fit-r54.log`,
`logs/r55-system-preference-before-and-ink-after.log`에 남긴다.


## R55: sample16 원 해결의 윤곽 계약 복원과 nested 분리 조사

원 해결 [PR #8188](https://github.com/edwardkim/rhwp/pull/8188)의 실제 보정
`0862655314f7d141052c7183c3bea1c6f9358473`과
`mydocs/working/task_m100_8160_stage2.md`·`stage3.md`를 대조했다.
38쪽 HCI Poppy는 다음 글자까지의 전진과 실제 윤곽 폭이 다르며, 일반 글자의 윤곽을
전진 칸 안에 축소하지 않는 계약이다. 후속 [PR #8754](https://github.com/edwardkim/rhwp/pull/8754)의
실제 통합 `7f76cee60e38fceb63feacaff706cf984b19c427`은 생략된 `preserveSourceMetrics`와
명시 false를 구분하는 과정에서 자연 전진 선택뿐 아니라 기존 `preserve_glyph_ink`도
함께 껐다. 실제 후보의 전진·글자 위치를 보존한 paint 진단에서 sample16 HWPX 38쪽이
raw 99.99464%, 미대응4 → 100.00000%, 미대응0으로 복원됐다. 기준 PDF·96 DPI·임계값232·2px는
바꾸지 않았다. 이는 수정 전 코드의 진단 결과이며 정식 제품 완료는 아니다.

공통 생산자 `ResolvedCharStyle`에서 자연 전진 선택을 `font_families_natural_program_metrics`로
분리하고 `TextStyle.natural_program_metrics`로 전달한다. 측정의 HFT 굵기·leader 분기만 이 값을
소비한다. paint의 `preserve_glyph_ink`는 기존 명시 공급의 윤곽 계약을 계속 소비한다.
명시적인 자연 프로그램 선택은 두 계약을 함께 선택하고, 원본 메트릭 보존은 둘 다 바꾸지 않는다.
문서명·쪽 번호·좌표·특정 글자 분기를 제품에 추가하지 않는다.

사설 글꼴 없는 공개 API 합성 검사 `legacy_program_ink_metric_separation`은 수정 전 고정
Native library에서 의도한 윤곽 단언 FAIL(0 PASS·1 FAIL, exit101)을 확인했다. 처음 진단 코드의
소유권 컴파일 오류는 별도 기록이며 테스트 실패나 통과로 합산하지 않는다. 기존 자연 전진 검사는
검사 목적에 맞는 새 전용 값을 확인하며 기존 leader·꾸밈 보호를 유지한다. 수정 후 실제 집중 검사와
Native/fresh WASM 측정은 아직 수행 전이다.

nested의 원 입력은 [PR #2052](https://github.com/edwardkim/rhwp/pull/2052), 원 반영
`eac1ac0cb73e905b5a32523d427f43b88e84c481`이며 1×1 중첩 셀의 실제 분할을 도입했다.
이후 [PR #8397](https://github.com/edwardkim/rhwp/pull/8397)의 기존 회귀 입력 재측정이
17쪽을 검증했다. 현재도 17쪽·내용 소유는 유지되며 보호 기준8쪽99.49% 대비99.48%를
복구해야 한다. 윤곽 보존만 바꾼 진단은 raw99.48284→99.48285%로 표시값을 복구하지 못했다.
따라서 sample16 수정으로 해결됐다고 표시하지 않으며 실제 선택 프로그램·같은 원본 글줄을
별도로 대조한다. 원 프로그램 재개방·물리 스타일 정렬의 단독 및 결합 진단도 아직 이 차이를
복구하지 못했다. 미해결 항목은 삭제·기준 완화하지 않는다.

실행 로그·중간 JSON은 ignored `output/pr-review/planet6897-integration-20261106/`의
`r55-paint-ink-only-metrics.json`, `r55-native-glyph-frame-diagnostic.json`,
`r55-combined-glyph-frame-diagnostic.json`, `logs/r55-legacy-program-ink-before-test.log`에 있다.

## R52 추가 전쪽 대조에서 발견한 제목 전진 차이

대기 중이던 추가 전쪽 검증을 실제로 진행해 `report-628` 195쪽 중 4·6·8·11·13·21·27·28쪽의
이전 R43 대비 0.1%p 초과 하락을 확인했다. 4쪽은 99.99346→96.24296%, 13쪽은
99.98966→97.32621%다. 기존 선택 쪽 보정 완료를 전쪽 완료로 확장하지 않는다. 원본·PDF·
compat 2022·96 DPI·글꼴 환경 SHA는 두 실행에서 같으며, 기준을 다시 출력하지 않았다.

원 PR #8754의 통합 `7f76cee60e38fceb63feacaff706cf984b19c427`과 그 작업 기록의
「새 devel 기능 보존 중 확인한 HFT 굵기 전진」을 대조했다. 해당 보정은 #8418의 `신명 태명조`
원본을 표시용 대체 프로그램으로 공급한 뒤 기존 HFT 꾸밈 전진을 잃는 경계를 보호한다.
기존 #8389·#8397 및 `legacy_font_advances`의 HWP3 원 프로그램 꾸밈 계약도 별도로 유지해야 한다.

반면 `report-628` 4쪽의 독립 Print는 실제 TrueType 휴먼명조(Type0, embedded TTF)를
사용한다. 제목의 `제` 원점은 99.18624pt, 뒤 숫자 원점은 113.57845pt로 차이는 약 19.18961 CSSpx다.
R43의 해당 첫 run 폭은 19.2px, R51은 20.2px다. 실제 Native 준비를 마친 추적에서도 원본의
-10% 자간은 그대로이며, 공급 프로그램의 1em 전진에 HFT fallback의 0.05em을 추가한 차이를
확인했다. 자간 수치·독립 PDF·기준값을 바꾸어 맞추지 않는다.

보정 경로는 `실제 주 프로그램 선택의 원본 쿼리·프로그램 신원 → 해소된 언어 슬롯 → 실제 글리프
공급 프로그램 → 공통 문자 전진 → 줄/paint 원점`이다. 대체 후보·선택된 주 프로그램·그 프로그램에
없는 글자의 실제 fallback을 구분한다. 같은 주 프로그램이 해당 글리프를 실제로 공급하는 경우에는
HFT의 추정 꾸밈을 새로 붙이지 않고 원본 자간과 자연 전진을 소비하도록 보정한다. 독립적으로
확인된 HWP3/HFT 꾸밈과 #8418의 대체 프로그램 소유는 보존한다. 공개 CFF를 이용한 합성 계약과
실제 미달 8쪽·정상 대조군으로 수정 전후를 확인한 뒤 완료를 판정한다. 현재는 원인 분석 및
수정 전 재현 단계이며, 보정 후 통과를 선기록하지 않는다.

### R54 집중 회귀와 실제 프로그램 신원

제품 source `e2ca9b0c52ce705cddaf34dd67bbf5c787f9e445`에서 최소 공개 글꼴 환경의 집중 회귀
**55 passed / 0 failed**를 확인했다. 1,880건은 필터 밖 검사이며 전체 회귀의 skipped에 합산하지
않는다. 새 주 프로그램 계약 5건은 -10/0/+10 자간, 실제 주 공급자·문서 지정 대체·다른 프로그램·
기존 HWP3 꾸밈·브라우저 독립 SFNT를 구분한다. 기존 #8418 공개 CFF 및 원 프로그램 소유·
문장부호·굵은 제목의 관계 검사도 유지했다.

원 Native 프로그램 신원은 파일 SHA만이 아니라 `SHA-256(SFNT bytes || face index big endian)`
이다. 실제 공급자가 같은 face인지 이 규칙으로 확인하며, 브라우저 `sweep-` 신원은 독립 SFNT
자원의 실제 바이트 해시와 index 0을 확인한다. 서로 다른 face나 결측 글리프의 뒤 프로그램에
신원을 넘기지 않는다. 확인한 공급자의 배열 인덱스를 캐시해 글자마다 프로그램 바이트를 다시
해시하지 않는다. 원본 자간·장평·자연 glyph-fit과 HWP3/HFT의 독립 꾸밈 계약은 유지했다.

[동일 수정 전 실행 파일의 정식 Native 신원 재현](primary-program-decoration-r52/before-native-identity-test.txt),
[최소 글꼴 수정 전 실패](primary-program-decoration-r52/before-minimal-fonts-test.txt),
[수정 후 집중 회귀](primary-program-decoration-r52/after-focused-test.txt),
[실제 source·명령·실행 파일 SHA](primary-program-decoration-r52/after-focused-provenance.json).
중간 R53은 새 검사의 SHA 문자열 표현 컴파일 오류로 실행되지 않았다. 해당 오류를 수정하고
실제 신원 계약을 반영한 위 R54 실행만 최종 집중 결과로 사용한다. 전쪽 Native/fresh WASM과
실물 8쪽·정상 대조군의 출력 복구는 별도로 확인 중이며, 이 통과로 대체하지 않는다.

### R54 실제 Native 제목 8쪽 복구

같은 원본·독립 Print·96 DPI·compat 2022·기존 글꼴 환경에서 하락 8쪽을 다시 출력했다.
모두 이전 R43의 두 자리 표시값을 복구했다. 이 표는 선택 8쪽이며 전195쪽 검증을 뜻하지 않는다.

| PDF 쪽 | 보정 전 R51 Native | 보정 후 R54 Native | 이전 R43 표시값 |
| --- | ---: | ---: | ---: |
| 4 | 96.24% | 99.99% | 99.99% |
| 6 | 99.47% | 99.91% | 99.91% |
| 8 | 98.90% | 100.00% | 100.00% |
| 11 | 98.71% | 99.96% | 99.96% |
| 13 | 97.33% | 99.99% | 99.99% |
| 21 | 99.39% | 99.91% | 99.91% |
| 27 | 98.93% | 100.00% | 100.00% |
| 28 | 99.78% | 99.93% | 99.93% |

[보정 전 선택 TSV](primary-program-decoration-r52/before-native-selected.tsv) ·
[보정 후 선택 TSV](primary-program-decoration-r52/after-native-selected.tsv) ·
[이전 검증 선택 TSV](primary-program-decoration-r52/reference-native-selected.tsv) ·
[producer·원본·Print·글꼴·정밀 수치·판독](primary-program-decoration-r52/after-native-selected-provenance.json) ·
[4쪽 review](primary-program-decoration-r52/after-native-p004-review.png) ·
[4쪽 overlay](primary-program-decoration-r52/after-native-p004-overlay.png).

정상 대조군은 childcare 13/24/25쪽100.00/99.94/99.92%, sample10의1/2/37/52쪽
99.96/100.00/100.00/100.00%, 기존 #8418의2쪽100.00%로 유지됐다. 앞 문단의 선택 쪽을
전체 문서 또는 fresh WASM 통과로 확대하지 않는다. 추가 보호 기준에는 sample16 저장본3개의
38쪽100.00→99.99%, nested8쪽99.49→99.48%가 남아 있다. 이 네 행은 이번 제목 8쪽 복구로
상쇄하지 않고 다음 보정·최종 전쪽 검증에서 해결한다. PR 준비 완료 판정은 아직 보류한다.

## 관측한 차이

같은 `pr8642/report-838-original.hwp`와 기존 독립 한컴 Print PDF의 24쪽을 비교했다.
원 PR의 각주·표·캡션 소유 검토 중 제목과 캡션의 굵기가 이전 Native 출력보다 가늘어진 것을
확인했다. raw 99.94%만으로 직접 판독을 대신하지 않는다. 글꼴 프로그램을 배포물에 추가하거나
기준 PDF를 다시 인쇄해 이 차이를 가리지 않는다.

- 이전 Native `0261687265f3a10a64e51809d006b2305a36d950`과 현재 Native
  `65195e9730f35bdbd7ea0c9a393e07e66125202e`에 같은 글꼴 환경·경로·compat 2022·96 DPI를 공급했다.
- 이전/현재 글꼴 환경을 교차 적용해도 굵기 차이는 실행 파일을 따라간다. 새 base
  `7f76cee60e38fceb63feacaff706cf984b19c427`에서도 같은 Native 제목 차이를 재현했다.
  따라서 글꼴 환경 JSON 교체만으로 해결됐다고 판정하지 않는다.
- 원본의 제목·표 캡션·표 머리말은 공개 레이어 getter에서 모두 명시 `bold=true`다.
  family 이름만으로 합성 굵기 여부를 추측하지 않는다.
- 독립 PDF의 같은 제목과 캡션에는 fill 및 stroke 텍스트가 함께 있다. 원본 문서의 bold와
  실제 프로그램의 weight, OS가 덧붙인 합성 weight, 최종 paint의 획을 구분해 추적한다.
- 이전/현재/Print 확대 영상의 검은 잉크 합계는 굵기 차이를 찾는 진단값이다. 정식 일치율이나
  새로운 통과 임계값으로 사용하지 않는다.

## 보정 범위와 검증

PR #8754의 실제 프로그램·cmap·collection face 선택과 20조건/562쪽 보호는 유지한다.
그 변경 전체를 되돌리지 않으며, Native의 실제 선택 face에서 최종 굵기 paint까지 공통 경로를
확인한 뒤 필요한 부분을 보정한다. WASM의 완료된 제품 source와 실제 바이트가 영향을 받지 않으면
그 증적을 재사용하고, Native는 새 실행 파일로 영향 영역과 필수 게이트를 다시 검증한다.

## 확인한 원인과 보정

호스트의 두 Fontconfig pattern 규칙은 `한컴바탕` 요청에 HCR 대체를 넣고 실제
Haansoft face를 다시 우선한다. OS의 한 번의 face 매칭과 스타일 목록 열거는 같은 결과를
보장하지 않는다. 실제 `family_order=0` 후보에 HCR Batang과 Haansoft Batang이 함께
들어왔으며, 새 TTF 우선 처리에서 다른 계열의 Bold TTF가 원 계열을 대신했다.
물리 프로그램 재개방 자체나 명시 `bold`가 사라진 문제가 아니다.

- `matching_system_programs`는 같은 요청 이름의 스타일 목록을 실제 family 소유로
  간주했다. 목록의 각 프로그램이 SFNT family를 선언했는지 확인하도록 보정한다.
- `match_system_family_style`의 같은 스타일 TTF 우선 선택도 실제 family 소유를
  확인한다. 같은 계열 안의 TTF → TTC → OTF 정책과 물리 프로그램의 cmap·paint 일치는 유지한다.
- 공개 GUST CFF와 OFL TTF 두 개만 격리 공급하고 같은 pattern 규칙을 공개 이름으로
  구성한 대조군을 만들었다. 원본 글꼴의 name 테이블이 독립 기대값이다.
  수정 전 요청 `Latin Modern Math` 대신 `Rhwp Family Contract`가 선택되어 exit 101로 실패했다.
- 새 정식 검사는 `native_system_font_family_ownership.rs`이며 새 프로세스의 전용
  Fontconfig에서만 두 프로그램을 노출한다. 보통/굵게, 원 CFF 계열/대조 TTF 계열의
  양방향 소유를 검사한다. 설치된 한컴·OTF나 다른 검사에서 초기화한 FontMgr에 의존하지 않는다.
  합성 이름 선택 계약이며 실문서의 px·쪽수·이미지 기대값을 만들지 않는다.

[수정 전 정식 검사 실패](native-family-ownership-r46g/before-test.txt) ·
[실패 실행 출처](native-family-ownership-r46g/before-provenance.json) ·
[사전 진단](native-family-ownership-r46g/before-probe.txt) ·
[실제 후보 묶음](native-family-ownership-r46g/actual-chain-before.json).

보정 직후에는 재검증 완료 전이었다. 아래 R47의 실제 선행·정식 검사 결과와 최종 전쪽 대조를 구분한다.


## R47 실제 출력 선행 확인

제품 source `0275f9c3bb69b216c96146e98506271eead9a601`의 새 Native 실행 파일에서
같은 원본·Print·글꼴 환경으로 PDF 24쪽 raw 100.00%, 49쪽 99.99%를 확인했다.
24쪽의 제목·캡션은 이전 정상 R43과 같은 검은 잉크 합계를 복원했다. 잉크 합계는 진단값이며
통과 임계값으로 사용하지 않았다. 실제 review·overlay에서 제목의 굵기, 본문·각주 62~66,
표 캡션·4개 열·마지막 행·쪽 번호 소유를 함께 판독했다.

[선택 쪽 TSV](native-family-ownership-r46g/after-selected-native.tsv) ·
[관측·출처](native-family-ownership-r46g/after-selected-provenance.json) ·
[review](native-family-ownership-r46g/review-after-p024.png) ·
[overlay](native-family-ownership-r46g/overlay-after-p024.png).

이 절은 선택 쪽의 실제 선행 확인이다. 새 회귀·전체 필수 검사와 전쪽 Native/fresh WASM
검증 완료는 아직 선언하지 않는다.

## R47 정식 이름 소유 회귀 검증

같은 제품 source에서 최소 글꼴 wrapper로 새 회귀를 실행해 **1 passed / 0 failed**로 통과했다. 선택 밖 308건은 전체 회귀의 skipped로 합산하지 않는다. Native feature의 라이브러리·해당 suite Clippy도 `-D warnings`로 exit 0을 확인했다. 수정 전 실패와 같은 assertion·입력을 유지한 검사이며 원 계열의 프로그램 소유를 확인한다.

[수정 후 정식 검사](native-family-ownership-r46g/after-test.txt) · [source·파일·로그 SHA](native-family-ownership-r46g/after-test-provenance.json). 전체 Rust·fresh WASM·전쪽 시각 검증은 별도 진행 중이다.

## R47 필수 기본 검증 완료

같은 제품 source에서 전체 Rust **13,069 passed / 0 failed / 45 skipped**(exit 0), Native lib 4개 crate 합계 **4,035 passed / 0 failed / 12 ignored**, Native 집중 **86 passed**를 확인했다. 같은 최소 글꼴 환경의 format·Native/WASM32/workspace lint·workspace build·정책 검사·문서 예제까지 후보 실행 17단계가 모두 exit 0이다. 실제 source와 출력 경로를 바꾸지 않았다.

최적화 WASM은 7분 55초에 성공했다. 공식 Native 편람 HWPX gate는 전 383쪽에서 기준 하락 0건이며, 382쪽 100.00%·독립 PDF 130쪽 raw 99.96%다. 알려진 WMF 차이를 마스킹하거나 100%로 고치지 않았다. 나머지 전쪽 Native/fresh WASM·최신 base·원 PR 기준 대조와 개별 최종 판독은 계속 진행한다.

## R48 전쪽 대조에서 확인한 추가 하락과 보정 계획

R47의 완료된 실제 결과를 이전 R43의 같은 원본·Print·compat·DPI와 대조해 report-628의
16쪽 96.88→94.21%, 18쪽 99.72→97.38%, 120쪽 96.55→94.81%,
122쪽 99.58→97.22% 하락을 확인했다. 중첩 표·보육 문서의 작은 하락도 함께 추적한다.
이는 조사 결과이며 준비 완료 판정이 아니다. 새 base 대비 검사와 별개로 이전 개선의 보존을 확인한다.

수식의 공통 원인 가설은 `legacy_math_face`를 원본 HFT 표시와 HWP5 선택 문자열 생략의
두 계약에서 함께 사용하는데 PR #8754의 원본 그룹·첨자 여백까지 생략 계약에 적용하는 것이다.
생략 계약의 독립 10/20pt Print와 기존 `implicit_legacy`의 공통 줄 프레임을 보존해야 한다.
`original_group_unit → original_group_frame/original_script_base → layout_node/layout_script_base`
호출 경로를 대조했다. 명시적인 원본 HFT 표시와 생략된 선택 문자열을 구분해 보정하고,
같은 수식 4쪽과 PR #8754의 원본 그룹·첨자 대조군을 함께 측정한다. 문서 ID·좌표 보정으로
수치를 맞추지 않는다. 코드 보정 전 관측 원장은 ignored output의
`r47-last-validated-live-diagnostic.json`이다. 독립 출력 대조가 가설을 뒷받침하지 않으면 채택하지 않는다.

### 보육 문서의 가운데점 전진 하락

독립 Print 13쪽의 `일회적·단편적인`에서 가운데점은 H2hdrM 15pt이며,
96 DPI로 환산한 점 원점175.52와 뒤 글자195.52의 차이는20px다. 원본 15pt의 1em이다.
R43은 같은 원본 TTF의 검증된 표를 신뢰해1500 HU를 사용했으나 R47은448 HU로 줄였다.
현재 공개 FontDecisionTrace는 `HY헤드라인M`이 물리 프로그램의 논리 family `HYHeadLine M`으로
표시 인증된 뒤 `metricNarrowPunctuationOverlay`로 잘못 내려가는 것을 보여 준다.
원본 쿼리와 같은 선택 프로그램의 신원 인증을 다른 글꼴로의 대체와 혼동했다.

`selected_source_programs → 검증된 원본 TTF의 metric 신뢰 → 공통 문자 폭 결정 → 뒤 run의 원점`
경로를 보정한다. 같은 원본 쿼리의 실제 선택임이 확인된 경우만 기존 검증된 메트릭 신뢰를 보존하고,
임의 대체 프로그램에 신뢰를 새로 부여하지 않는다. HFT와 자연 프로그램 선택의 기존 경계도 유지한다.
해당 관측은 ignored output의 `r48-childcare-dot-independent-print.json`과
`r48-childcare-dot-trace.json`이다. 이 절은 원인·보정 계획이며 아직 재검증 통과가 아니다.

### R48의 실제 선행 보정 결과와 남은 조건

공용 target·최소 글꼴·8 jobs Native 빌드는 exit0, 23분55초로 완료했다.
같은 원본·기존 Print·96 DPI·compat2022에서 그룹 계약 분리 후 16쪽96.27%,
18쪽98.79%, 120쪽96.67%, 122쪽98.84%를 실제 산출했다. 보정 전 R47의
94.21/97.38/94.81/97.22%보다 네 쪽 모두 상승했다. 이 값은 선택4쪽의 선행 확인이며
전쪽/fresh WASM 완료로 표현하지 않는다. 이전 R43 기준에는 일부 하락이 남아 있다.

[실제 선택 쪽 TSV](implicit-frame-r48/native-selected.tsv) ·
[관측·producer·변경 파일 SHA](implicit-frame-r48/native-selected-provenance.json) ·
[진단 소스 차이](implicit-frame-r48/source.json) · [빌드 로그](implicit-frame-r48/native-build.txt).

이 단계에서 사용한 것은 R43의 자동 생성 글꼴 환경이었다. 이후 R50에서 확인한 실제 승인
Print 대체 설정과는 다르다. 이 대조만으로 글꼴 환경을 원인에서 제외할 수 없다.
기준 PDF·raw 알고리즘·기준값은 바꾸지 않는다.


## R49 원 해결 커밋 대조와 두 선택 경로의 보정

원 PR의 실제 코드와 독립 증거를 다시 추적했다. 다음 두 수정은 아직 아래 집중 재검증을
진행하는 단계이며, 새 head의 전쪽 일치율 복구 완료를 뜻하지 않는다.

| 원 해결 | 보존할 계약 | 현재 경로의 차이와 보정 |
| --- | --- | --- |
| PR #8389·PR #8397의 실제 후보 순서, PR #8754 통합 `7f76cee60e38fceb63feacaff706cf984b19c427` | 같은 원본 계열의 프로그램·실제 스타일, TTF → TTC → OTF와 실제 cmap/paint 일치 | 후보 열거는 `unemboldened_typeface`로 물리 Regular를 확인하지만 마지막 정렬은 OS 가상 Bold를 실제 Bold로 취급했다. 마지막 정렬도 동일한 물리 스타일을 소비하게 했다. 실제 Bold·가변 wght 판정과 family 소유 검사는 유지한다. |
| 이슈 #7092의 원 해결 `01e2c9905a8a9ae5767d20ead9e264229c327e38`, 자동 프로그램 공급 PR #8565 `5551091085` | 실제 주 TTF·문서 선언 대체 TTF와 HFT의 서로 다른 문장부호 폭 | 원본 `HY헤드라인M`을 같은 프로그램의 family `HYHeadLine M`으로 인증한 뒤 기존 메트릭 신뢰를 잃었다. 원본 TTF·기존 신뢰·원본 조회 쿼리·실제 측정이 소비하는 원본 표를 모두 확인한 별칭만 신뢰를 보존한다. 별칭에 독립 메트릭 키가 없어도 원본 표를 그대로 쓰는 경우를 포함하며, 다른 메트릭 표를 선택하면 제외한다. 다른 프로그램·HFT에 신뢰를 새로 부여하지 않는다. |

`matching_system_programs → chain 최종 정렬 → select_typeface_for_character → candidate_typeface → paint`
경로에서 실제 스타일 소유를 공유한다. `원본 슬롯 신뢰 → 환경 프로그램 인증 → metric_trusted →
공통 문자 전진 → 후속 run 원점` 경로에서 같은 표의 별칭이 신뢰를 덮어쓰지 않게 한다.

프로그램 재개방 자체를 원인으로 보았던 가설은 배제했다. 정식 실행 파일을 바꾸지 않은
별도 진단 사본에서 기존 clone 경로만 실행해도 중첩 표 1쪽은 95.68127%로 같았다.
따라서 이슈 #7640의 실제 프로그램/cmap 일치 수정은 되돌리지 않았다. 이 사본은 원인 분리용이며
정식 후보·통과 증거로 제출하지 않는다. 원장과 로그는 ignored output의
`r49-native-program-diagnostic-receipt.json` 및 `logs/r49-original-program-native-diagnostic.log`다.

검증은 기존 독립 Print·96 DPI·기존 승인 대체 설정을 유지한 중첩 표의 하락 9쪽,
보육 13·24·25쪽, 수식 보고서의 하락 및 보정 대조 15쪽, NARS2021 1쪽부터 수행한다. 전쪽 검증과 PR #8754의
실제 프로그램 일치·원본 수식 대조군 및 새 base의 clip 경계 보존은 별도로 확인한다.
새 관계형 검사는 설치 글꼴 없이 공개 `DocumentCore`·`FontEnvironment`를 통해 같은 별칭의
메트릭 보존과 다른 프로그램·HFT 반례를 확인한다. 기대값·독립 PDF·측정 알고리즘은 바꾸지 않는다.

## R50 실제 실행 결과와 가설 정정

R49 제품 커밋은 `40c90ca187201527f5b7776f86f0fe6e493cef52`다. 최소 공개 글꼴 환경의
집중 검사 29건은 모두 통과했다. 같은 TTF 별칭의 메트릭 신뢰 보정 후 보육 문서 13쪽의
실제 Native raw는 99.21%에서 100.00%로 회복됐다. 그러나 물리 스타일의 마지막 정렬을
바꾼 실험은 중첩 표의 하락을 해결하지 못했고 일부 다른 쪽의 raw를 낮췄다. 따라서 위 표의
정렬 변경은 원인 해결로 채택하지 않고 종전 정렬로 복원한다. 그와 별개로 입증한 family
소유 검사와 같은 TTF 메트릭 신뢰 보정은 유지한다.

### 독립 Print의 글꼴과 승인 설정 복원

중첩 표 PDF의 제목·표 머리글은 실제 `Gulim`인데, 자동 공급은 설치된 `휴먼명조` CFF를
선택했다. 원 PR 검증의 정식 자산
`mydocs/pr/assets/planet6897_integration_final/pr8389_8397/hwp3-2020-font-environment.json`과
작업지시자가 승인한 `휴먼명조 → Gulim` 계약을 재확인했다. R43 자동 환경을 이 승인 설정으로
부른 것은 잘못이므로 정정한다. 전체 시스템 글꼴 공급은 유지하고, 해당 원본의 자동 환경에
승인된 대체와 `Gulim → Haansoft Batang` 글리프 fallback만 명시한다. 같은 원본의 scoped
프로그램 선택이 이 대체를 다시 덮어쓰지 않는지도 확인했다. 다른 PDF에는 일괄 적용하지 않는다.

독립 Print·입력·96 DPI·raw 알고리즘을 유지한 `40c90ca187` Native 실제 결과는 다음과 같다.

| 중첩 표 쪽 | R43 기준 | 승인 Print 대체 복원 후 |
| --- | ---: | ---: |
| 1 | 96.58% | 98.20% |
| 6 | 98.11% | 100.00% |
| 7 | 97.74% | 99.10% |
| 16 | 96.00% | 97.72% |

이는 선택 4쪽의 실제 결과이며 전17쪽·fresh WASM 완료를 뜻하지 않는다. 실행 원장은
ignored output의 `r50-print-gulim-nested-selected.json`, 환경은
`r50-nested-print-gulim-environment.json`이다. 최종 제출에는 전쪽 증거를 별도로 고정한다.

### 원본 CHAR와 선택 문자열 생략 수식의 전진 소유

PR #8754 통합 `7f76cee60e38fceb63feacaff706cf984b19c427`의 전후 Git 소스를 대조했다.
원본 CHAR의 장치 격자 반올림과 이탤릭 전진표는 원본 CHAR에 필요한 계약이다. 그러나
같은 표시 face를 사용하는 선택 문자열 생략 HWP5 경로까지 이 계약을 적용하면서, 기존
연속 전진과 정체 전진 위의 별도 shear를 바꿨다. 이전 코드는
`legacy_glyphs::measure`에서 `units * fs / 1000`을 누적하고 이탤릭도 같은 정체 전진을 쓴다.

`implicit_legacy_font → source_glyph_runs → legacy_glyphs::measure → LayoutBox.width →
후속 수식 원점`에서 출처를 전달하도록 `AdvanceFrame`을 구분한다. 원본 CHAR의 반올림·
이탤릭 전진은 유지하고, 선택 문자열 생략에만 이전 연속 전진·정체 폭과 shear를 복원한다.
원 CHAR에서 추가 수용한 소수점·가운데점도 생략 경로의 기존 fallback을 바꾸지 않는다.
분수·제곱근은 생략 표시 지원 AST가 아니므로 그 경로에 조건을 더하는 가설은 채택하지 않았다.

새 합성 관계 검사는 실제 `DocumentCore` 삽입·배치 경로에서 크기를 두 배로 바꾼 전진과
정체/기울임의 폭 관계를 검사한다. 설치 글꼴·실물 좌표 핀을 쓰지 않는다. 수정 전 실제
`40c90ca187` 라이브러리에 연결한 실행은 `a: 9.96 → 20.92`로 실패했다(종료101).
수정 후 집중 검사와 수식 보고서의 독립 Print 대조는 계속 수행하며, 아직 통과로 기록하지 않는다.
수정 전 원장은 ignored output의 `r50-implicit-advance-before.json`과
`logs/r50-implicit-advance-before.log`다.

## R51 원 PR의 비적용 범위와 남은 문장부호 전진

[원 PR #8605](https://github.com/edwardkim/rhwp/pull/8605)의 실제 head
`5d37286267136c9978d7ac2f804913730ab6becf`와 코드 `4f8570309e`를 Git에서 다시 확인했다.
이 변경은 WASM 합성 굵기 전용이며 Native를 바꾸지 않는다. 기여자의 공개 바탕 대체 측정과
현재 설치 프로그램을 공급한 측정을 혼동하지 않는다. 이후 통합한
[PR #8754](https://github.com/edwardkim/rhwp/pull/8754)의 프로그램 선택과 현재 보정도 함께 대조했다.

R50의 실제 집중 회귀는 최소 글꼴 환경에서 11/11 통과했다. 생략 수식 보정으로 보고서
16·18·120·122쪽은 각각 기준과 같은 96.88·99.72·96.55·99.58%로 회복됐다. 중첩 표는
승인 Print 대체를 적용한 전17쪽이 90% 이상이며 최저95.69%다. 알려진 Native 하락27행 중
0.1%p 초과 하락은 보육13·24·25쪽만 남았다. 따라서 fresh WASM 빌드는 아직 시작하지 않았다.

보육24쪽은 이전 출력과 현재 PNG의 실제 차이가 법조문 첫 줄에 한정된다. 독립 PDF의
`H2gtrE`에서 `직 → · → 파` 원점은 108.72 → 118.44 → 128.04pt다. 가운데점은 약9.60pt를
점유하며 전각 계약이다. 이전 Native는 같은 run을 약12.9 CSSpx로 측정했지만 현재는3.8px로
줄여 뒤 문장을 당겼다. 글자 크기나 좌표를 맞추는 임의 보정이 아니라 가운데점 전진의 소유 문제다.

실제 자동 환경은 원본 `HY견고딕`을 같은 조회 쿼리의 TTF family `HYgtrE`로 인증한다.
원래 측정 키 `HYGothic-Extra`와 별칭 키 `HYgtrE`는 em·문자 전진이 같지만 서로 다른
정적 객체다. BMP 전체 전진 대조에서 차이는0개였다. R49 helper가 포인터 동일성만 확인해
이 동일한 전진을 다른 표로 판정하고 기존 신뢰를 잃어 좁은 문장부호 overlay로 떨어졌다.
이는 글꼴 전체 교체나 합성 굵기 차이로 해결할 문제가 아니다. 보육 문서에 중첩 표의 Gulim
설정을 확장한 진단도 해결하지 못했으므로 최종 설정으로 채택하지 않는다.

원본 TTF·기존 신뢰·같은 원본 조회·원본 CSS 선두 소유 조건은 유지한다. 표의 같은 객체는
즉시 수용하고, 객체가 다를 때는 스타일·em·문자 범위와 모든 전진·한글 조합 매핑 및 폭이
동일한지 비교한다. 이름만 다른 같은 전진의 표를 다른 프로그램의 폭으로 오인하지 않는다.
Arial처럼 전진이 다른 표와 HFT·다른 조회 경계는 계속 거절한다. 설치 글꼴 없이 실제
`DocumentCore` 경로에서 `HY견고딕/HYgtrE`를 기존 별칭 관계 검사에 추가했고 수정 전 실제
R50 라이브러리에서 실패했다. 이 실패를 허용하거나 기대 전진을 줄이지 않는다.

수정 전 실제 로그는 ignored output의 `logs/r51-original-truetype-alias-before.log`,
원장은 `r51-alias-before.json`이다. `r51-source-metric-identity-unit-provenance.json`의
작은 대조는 함수의 구조 동등성만 확인하며 실제 집중 회귀·Native/fresh WASM·전쪽 판독을
대체하지 않는다. 보정 후 같은 원본·Print로 세 쪽과 대조군을 다시 확인한다.

## Mac 이관: CoreText 원본 프로그램 식별과 OS별 검사

2026-11-08 Mac 이관 후보는 `1b159d1f855c82a28de8b909b6f15a9b1d65c7b7` 위의
로컬 보정이다. 원 해결 PR #2052의 셀 내부 분할, #8397의 HFT 윤곽·전진 공급,
#8389의 보호 TSV를 각각 대조했다. 중첩 표 8쪽의 하락은 분할 정책 변경으로 해결하지 않았다.

Mac과 Ubuntu의 `HMKMG.otf` 원본 SHA-256은
`62514ec65a65c0bd41f33d2c2d699ec68518c43d32f5c399df33f01733473dc9`로 같다.
그런데 CoreText의 선택 face를 SFNT stream으로 얻으면 테이블 내용은 같아도 컨테이너가
재포장된다. 원본의 프로그램 식별값은 `d1d1d8e02020ebe081cb0af257453477ca781ae9cda063e18c8f10cb46aa18c7`,
재포장본은 `0070b8a6ccd84820c1bcba132b5e56e49af106f25e4e149587db40fd6afbb999`다.
원본 파일과 collection index로 인증한 주 프로그램을 다른 대체 프로그램으로 오인해,
합성 굵기의 HFT 추가 전진이 들어갔다. ‘인도네시아’ 상자는 67.5px에서 70.9px로 늘었다.

OS face의 PostScript 이름으로 CoreText의 실제 파일 URL을 조회하되, 이름 일치만으로
공급하지 않는다. 선택 Typeface와 후보 파일의 **모든 SFNT 테이블**을 바이트 단위로
대조한 경우에만 원본 바이트와 collection index를 보존한다. 파일 크기·collection 수의
상한을 두고, 로컬 파일을 확인하지 못하면 기존 stream 경로를 사용한다. PUA 공급과
Native OTF 메트릭 준비가 이 공통 공급 결과를 소비한다. 글꼴 설치나 대체 설정은 바꾸지 않는다.

`tests/cases/native_fractional_glyph_origin.rs`의 MIT CFF 검사에서 실제 CoreText 시스템
공급을 사용했다. 수정 전에는 보통/합성 굵기 전진이 각각
`[0, 18.0266667, 36.0533333]` / `[0, 18.9866667, 37.9733333]`로 달라 실패했고,
수정 후 같은 관계 검사가 통과했다. 명시적인 custom 공급으로 시스템 경계를 우회하지 않는다.

Linux의 Fontconfig 재현은 유지한다. Mac은 검사 자식 프로세스의 CoreText 등록만 사용하고
사용자·시스템 글꼴을 설치하거나 삭제하지 않는다. Windows는 DirectWrite가 실제 선택한
OS 기본 face의 출력과 같은 프로그램을 명시 공급한 출력을 대조한다. GDI private 등록을
DirectWrite 등록으로 간주하지 않는다. 다른 MIT fixture와 출력이 달라야 하는 반례도 유지한다.
공통 CFF 분할·family 선택과 SVG 검색 루트 경계도 Mac·Windows에서 수집되도록 했다.

Mac 최종 집중 회귀는 **28 PASS / 0 FAIL**이며 선택 밖 1,282건은 이번 실행 결과가 아니다.
Native Clippy도 통과했다. Windows 첫 실행의 `to_existing_font_data()` 없음은 검사 harness의
잘못된 가정이었다. 동일 Typeface의 stream fallback으로 보정한 Windows 10 Pro 19045의
SSH·PowerShell 재실행은 네 case 묶음 **27 PASS / 0 FAIL**로 완료됐다.
Windows와 Mac의 수정 소스 7개 SHA-256도 모두 일치했다. `rhwp` cdylib와 동명 CLI의
`rhwp.pdb` 충돌 경고는 남았지만 빌드·검사는 통과했다. 사용자가 output 정리 중 일부 검증 소스를 삭제한 뒤에는 해당
worktree를 제거하지 않고 누락된 compile 입력만 복원했다. Windows 주 작업장의 별도 변경은 보존했다.

새 Mac Native producer SHA-256은
`8b3cbc3f5f43b1ee6f7c68af2913d0dd98a7e43b6cd525970cbb303b6fb80b56`이다.
동일 원본·Print·기존 독립 PDF PNG의 해시를 확인하고 공식 Sweep으로 TSV를 다시 산출했다.
sample16 3종 전 192쪽은 보호 TSV의 공식 두 자리 표시값 대비 하락이 없다.
중첩 표 전 17쪽은 최저 96.04%이며 8쪽은 99.36147%에서 99.49328%(표시99.49%)로 복원됐다.
다만 **3쪽은 99.76% → 99.75%**로 남으므로 보호 조건을 전체 통과했다고 기록하지 않는다.
3쪽의 이전·후보 렌더 트리 134개 노드는 같고, 동일 PDF PNG와 재측정한 불일치 픽셀은
각각 83/87개다. 원본 CFF와 이전 Malgun/HCR 프로그램을 명시 공급한 대조도 99.75%이므로
글꼴 버전 교체를 해결책으로 채택하지 않았다. 이 후보의 fresh WASM·전체 통합 검증은 미완료다.

이후 Mac의 locked wrapper `--target web --out-dir pkg --no-opt` 빌드는 성공했다.
root pkg·Studio public·캡처 브라우저 서버의 실제 HTTP 응답 JS/WASM SHA-256이 일치했다.
Docker 최적화 빌드 결과는 아니다. fresh WASM 전 209쪽도 산출했으나 보호 TSV 대비 하락이 남았다.
중첩 표는 3·8·11·17쪽, sample16 HWP2024·HWPX는 각각 18·35쪽이다. 후자의 18쪽은 87.45%,
35쪽은 93.60%로, Native의 전쪽 유지 결과를 WASM 통과로 재인용하지 않는다.
중첩 표 첫 WASM 실행은 이관한 글꼴 디렉터리의 0바이트 임시 TTF 때문에 실패했다.
유효한 글꼴이나 엄격한 파서를 바꾸지 않고 임시 파일만 별도 output으로 보존·분리한 뒤 전 17쪽을
다시 산출했다. 최종 부분 결과와 하락 목록은 [실행 증거](macos-program-identity/validation.json)에 남긴다.
보호 조건 전체 충족과 PR 준비 완료를 뜻하지 않는다.

이 단계의 실제 로그·진단 원장은 ignored output의
`pr-review/native-program-resume-20261108/`에 보존한다. PDF·점수 문턱·원본·보호 TSV는 변경하지 않았다.

## Mac 후속: WMF 축소 표본과 HFT 가운뎃점 전진의 실제 소비 복구

2026-11-09 KST에 부모 `f1e6ac11eb`와 미커밋 보정의 실제 Native/fresh WASM으로
sample16 3종 192쪽과 nested 17쪽을 각각 다시 산출했다. 이 단계는 통합 검증의 중간 보정이며
PR 제출 완료를 뜻하지 않는다. [전쪽 TSV·producer·같은 페이지의 raw/review/overlay](device-mipmap-hft-repair/validation.json)에
실제 결과를 보존한다. 임시 스크립트·추출한 글꼴·로그는 ignored output에만 둔다.

### WMF 최종 표면의 축소 표본

WASM의 sample16 HWP2024·HWPX 18쪽은 87.44904%, 35쪽은 93.60137%였다.
실제 `drawImage`의 최종 WMF 표면 PNG, 장치 CTM, 출력 사각형을 기록했다.
같은 PNG를 Native Skia의 mipmap으로 재생하면 실제 Native 그림 영역과 100% 일치했고,
단일 레벨 선형·cubic 및 Chrome의 `high`/`medium`/`low` 선택은 같은 결과를 보장하지 못했다.
JPEG 복호화 교체 대조도 PNG가 같아 원인으로 인정하지 않고 되돌렸다.

Canvas의 quality 문자열에 맡기던 경계를 최종 metafile 표면 marker의 실제 축소율로 제한했다.
짝수 축의 2표본 평균·홀수 축의 `[1,2,1]` 표본, premultiplied RGBA,
두 mip 레벨의 선형 보간을 Native Skia와 대조한 공통 수치 모듈에 둔다.
장치 픽셀 재생은 기존 `replay_raster`를 사용한다. 이 경로는 회전·반전 CTM,
일반 bitmap·JPEG·잘라낸 원본 창을 대신하지 않으며 잘못된 크기·할당 한도는 기존 painter로 넘긴다.
문서명·쪽 번호·목표 점수로 분기하지 않는다. 실제 수학 계약은
[SkMipmap.cpp](https://github.com/google/skia/blob/main/src/core/SkMipmap.cpp),
[SkMipmapHQDownSampler.cpp](https://github.com/google/skia/blob/main/src/core/SkMipmapHQDownSampler.cpp)와
같은 바이트를 재생한 Native oracle로 교차 확인했다.

새 합성 관계 검사 3개는 홀수·짝수 축, 투명 픽셀의 숨은 RGB, 원본·장치 한도를 확인한다.
Native 실행 안에서 실제 생산 수치 모듈과 Skia mip 결과를 비교하며 실물 문서 px 위치를 고정하지 않는다.
현재 suite 48개 제한 안에서 자동 수집한다. 실제 fresh WASM 전쪽 결과에서 HWP2024·HWPX는
각 최저 99.99%, HWP2022는 64쪽 모두 100.00%다.

### HFT 가운뎃점의 원본 전진

nested 17쪽의 `예방‧관리하고`는 설치한 Gulim 두 버전의 366개 사용 글리프 윤곽이
독립 PDF의 embedded subset과 같았다. 문제의 U+2027은 별도 HFT 한양신명조 run이며
표시 CFF에 글리프가 없어 자연 전진이 약 0.30133em으로 바뀌었다. 이미 검증된 원본
512/1024em 전진을 실제 측정·paint의 공통 선언 경계로 전달한다. 일반 TTF의 자연 전진과 구분한다.
새 검사는 MIT 합성 CFF만 사용하고 비공개 설치 글꼴에 의존하지 않는다.
기존 4HU 장치 반올림의 최대 반 양자 2HU와 배율 관계·비 HFT 반례를 함께 확인한다.

최종 집중 검사는 HFT 15건과 mip 3건의 **18 PASS / 0 FAIL**이다. 최초 harness의
`Arc<String>` 타입 오류, 장치 반올림을 빠뜨린 exact assert 실패는 최종 검증으로 세지 않는다.
실제 소스 전진 결함은 수정 전 8 PASS / 1 FAIL, 수정 후 최종 묶음 통과로 확인했다.
같은 원본·Print·글꼴 환경에서 nested 17쪽은 Native 99.97771%, WASM 99.97805%로 복구됐다.
review와 overlay에서 가운뎃점 뒤의 간격 복원과 본문·쪽 번호 보존을 직접 확인했다.

### 남은 보호값과 전체 검증 상태

| 범위 | 실제 전쪽 수 | 공식 보호값보다 낮은 쪽 |
| --- | --- | --- |
| Native sample16 3종 | 192 | 없음 |
| fresh WASM sample16 3종 | 192 | HWP2024·HWPX 35쪽 각 100.00% → 99.99% |
| Native nested | 17 | 3쪽 99.76% → 99.75% |
| fresh WASM nested | 17 | 없음 |

35쪽의 raw 미대응 7픽셀은 Native에도 같은 위치에 남고, 내용 판정 경계의 옅은 색 차이다.
유사색 보조값이 100%인 것으로 raw 보호값을 대신하지 않는다. nested 3쪽은 이전 Linux와
새 Mac의 반올림된 render tree 134개가 같지만 이것만으로 세부 paint 일치를 입증하지 못한다.
원본 Type3·선택 CFF의 윤곽·공백 전진·실제 소비를 계속 대조한다. 허용폭·PDF·보호 TSV·마스크를 바꾸지 않는다.

Native/WASM Clippy·workspace build·all-target Clippy는 통과했다. 새 테스트의 표현을
`is_multiple_of(2)`로 정리한 것은 같은 짝수 판정이다. 시각 producer는 그 표현 정리 전의
동일 동작 보정 바이트로 고정했으며 이후 source/head의 최종 검증과 구분한다.
현재 manifest 재준비 후 manifest·unit-tier 검사는 통과했다. 전체 nextest는 436.279초에
**13,069 PASS / 1 FAIL / 45 skip**으로 완료됐다. 실패한
`regulatory_22_legacy_latin_dot_preserves_the_source_word_boundary`는 `덮개·울`의 줄 소속을
검사하며 관측한 두 번째 줄은 `덮`으로 끝났다. 원 해결 근거·독립 PDF·현재 글꼴 소비를 대조해
원인을 보정하며 검사와 기대값은 유지한다. 최종 제출 검증 완료로 기록하지 않는다.
이관 manifest의 Linux 절대 경로 때문에 0쪽에서 멈춘 실행과 최초 harness drift는 환경·준비 오류로
보존하고, 동일 파일의 Mac 경로 및 현재 파생 suite를 확인한 재실행만 유효 결과로 사용한다.


## 2026-11-09: #7518 Print 글꼴 조건과 회귀 검사의 정합

원 해결 `a9dfc4e664`·후속 `646aa19edfcd` 및
[기존 검토 기록](../../archives/pr_7518_review_impl.md)의 Palatino 근거를 확인했다.
현재 Mac에는 실제 HCI Poppy CFF가 설치되어 있다. CLI 자동 공급은 이 원본을 선택하며,
같은 환경을 적용한 추적에서 측정과 paint 모두 HCI Poppy를 선택한다. 앞선 진단에서
자동 환경을 적용한 CLI와 환경 없는 DocumentCore를 비교해 얻었던 소비 불일치 추정은 철회한다.
독립 Print는 Palatino Linotype을 사용하므로, 서로 다른 글꼴 조건에 같은 줄 경계를 요구하던
검사 조건이 잘못된 것이다. 원본 프로그램 우선 선택을 되돌리는 제품 보정은 하지 않았다.

검사는 Print의 `HCI Poppy → Palatino Linotype` 환경을 fixture로 명시했다. 기존
`덮개·`/`울`과 인용문 `사고`/`하다가`/`회전날` 줄 소속 assertion은 모두 그대로다.
공용 도우미의 기본 자동 공급 동작도 유지하며 이 한 검사만 명시 환경을 전달한다.
글꼴 파일은 추가하지 않고 기존 검증 메트릭 경로를 사용한다.

변경 전 전체 회귀는 13,070건 중 13,069 PASS·이 검사 1 FAIL·45 skip이었다.
변경 후 해당 모듈 23건은 23 PASS·0 FAIL(11.241초), 283건은 선택 밖이다.
전체 회귀 재통과로 확대하지 않는다. 검사 변경 전에 같은 원본·독립 Print·명시 환경으로
22쪽 Native **99.94307%**, fresh WASM **99.94699%** 및 직접 판독을 확인했다.
82쪽 전체 수용을 의미하지 않는다.

[생산자·hash·실행 범위](regulatory22-print-environment/validation.json) ·
[Native TSV](regulatory22-print-environment/native-p022-silhouette.tsv) ·
[WASM TSV](regulatory22-print-environment/wasm-p022-silhouette.tsv) ·
[Native review](regulatory22-print-environment/native-p022-review.png) ·
[Native overlay](regulatory22-print-environment/native-p022-overlay.png) ·
[WASM review](regulatory22-print-environment/wasm-p022-review.png) ·
[WASM overlay](regulatory22-print-environment/wasm-p022-overlay.png).

같은 시점의 Native 전 범위 측정은 93조건·2,994쪽이다. 이 중 새 devel의
`incoming8754-nov-original`·`incoming8754-nov-resaved`는 각각 17·18·19쪽이 90% 미만이다.
이는 최종 제출 통과가 아니며, 고정 base 출력과 원 PR을 대조해 보정을 계속한다.
정본 보호 Native의 편람 HWP 24쪽·sample10 126쪽·outline 8쪽 및 앞서 기록한 nested 3쪽의
공식 표시값 하락도 남아 있다. 편람 HWPX 383쪽의 보호값은 유지됐다.
이번 한 회귀 검사 복구로 이 잔여를 면제하거나 제출 완료로 처리하지 않는다.

## 2026-11-09: 유형 미상 슬롯의 실제 주 TTF 결측과 기호 칸 정렬

편람 HWP의 198쪽 raw 미대응 225픽셀은 표 안의 U+2219 점 기호에만 모여 있었다.
동일 쪽 HWP/HWPX의 문단·표 프레임은 같았다. 원 HWP5 FACE_NAME 하위 비트를 TTF로
단정하지 않는 현재 사양 보정은 유지한다. 그 비트를 다시 TTF로 인정해 수치를 맞추지 않았다.

실제 적재한 첫 프로그램은 KoPub TrueType이며 점은 해당 프로그램에 없다. 유형 미상 공급은
뒤 표시 후보의 coverage도 보존하므로, 전체 사다리의 `has_glyph`를 주 프로그램의 보유 여부로
사용하면 결측 판단을 놓친다. 공통 `canvas_cluster_fit`에서 유형 미상·비HFT 슬롯에 한해
실제 첫 TrueType의 cmap 결측을 조회하고, 기존 음수 자간·반각 칸·작은 잉크의 정렬 규칙을
소비하도록 보정했다. 측정 전진·문단 원점·원본 유형·CFF/HFT의 원본 전진은 바꾸지 않는다.
명시 TTF의 기존 판정도 유지한다. 같은 함수를 Native Skia와 Canvas가 소비한다.

MIT 합성 TTF의 결측과 뒤 CFF의 실제 표시를 메모리에서 공급하는 최종 Native paint 검사로
수정 전 standalone 1 FAIL을 재현했다. 보정 후 현재 파생 suite의 새 검사와 기존 빈 cmap
정상 대조군은 2 PASS·0 FAIL(2.733초), 331건은 선택 밖이다. 상용 글꼴·OS 설치에 의존하지
않으며, helper 반환값이나 실물 좌표 핀으로 대신하지 않는다. 기존 CI의
`issue_8153_symbol_fallback` 모듈 호출에 함께 수집되므로 CI 호출은 추가하지 않았다.

현재 후보 Native HWP **383/383쪽**, 최저 표시 **99.96%**, 표시100.00% **382쪽**,
정본 보호값 하락 **0쪽**이다. 198쪽은 Native **99.71102→100.00000%**,
fresh WASM **99.68423→100.00000%**로 복원됐다. fresh WASM의 197·198·224쪽도
각각100.00%다. WASM은 이 선택3쪽 결과이며 전383쪽 검증으로 확대하지 않는다.
Skia root lib3852 PASS·12ignore, 다른 lib182 PASS, PNG2 PASS·PDF35 PASS,
fmt·manifest·unit tiers도 통과했다. 전체 nextest 재통과와 다른 최종 게이트를 뜻하지 않는다.

[생산자·조건·실행 범위](primary-truetype-symbol-repair/validation.json) ·
[Native 전383쪽 TSV](primary-truetype-symbol-repair/native-handbook-hwp-383-silhouette.tsv) ·
[WASM 선택3쪽 TSV](primary-truetype-symbol-repair/wasm-handbook-hwp-selected-silhouette.tsv) ·
[Native 수정 전 review](primary-truetype-symbol-repair/native-before-p198-review.png) ·
[수정 전 overlay](primary-truetype-symbol-repair/native-before-p198-overlay.png) ·
[Native 수정 후 review](primary-truetype-symbol-repair/native-after-p198-review.png) ·
[수정 후 overlay](primary-truetype-symbol-repair/native-after-p198-overlay.png) ·
[WASM 수정 후 review](primary-truetype-symbol-repair/wasm-after-p198-review.png) ·
[WASM 수정 후 overlay](primary-truetype-symbol-repair/wasm-after-p198-overlay.png).

HWPX 198쪽은 Native100.00%지만 WASM99.65%가 남는다. 처음에는 새 기호 보정의 회귀로
추정했으나, 보정 전 고정 WASM을 같은 조건으로 실행해도99.65%인 것을 확인해 그 인과
판정을 철회했다. Native의100%와 수정 후 WASM만 대조해 새 회귀라고 부를 수 없다.
실제 Canvas는 Haansoft Batang의 점을 측정하는데, 별도 Print 윤곽은 미확인 앞 family를
건너뛰어 맑은 고딕 Semilight의 점을 그린다. 같은 canvas의 최종 path와 후보 프로그램의
윤곽 경계를 대조해 확인했다. `predecessors_known`를 판독하는 기존 합성 굵기 경로와 달리
보통 TTF fallback 윤곽 경로에는 이 소유 확인이 빠져 있다. 이 차이는 다음 단계에서 실제
Canvas 공급자를 유지하도록 보정하며, 본 커밋으로 전체 통합 PR 준비 완료를 선언하지 않는다.


## 2026-11-09: Print 대체 윤곽의 앞 후보 coverage 소유 보존

편람 HWPX 198쪽의 WASM 99.64940%는 앞 Native 보정에서 생긴 새 하락이 아니었다.
수정 전 frozen WASM도 같은 조건에서 같은 값을 냈다. 실제 Canvas는 Haansoft Batang의
점 기호를 선택하지만, Print 윤곽 경로는 coverage를 수집하지 않은 앞 후보를 건너뛰고
뒤의 Malgun Gothic Semilight 윤곽을 대신 그렸다. `predecessors_known`으로 이미 보존하던
faux-bold의 소유 규칙을 보통 TTF Print 대체 윤곽에도 적용했다. 미확인 앞 후보가 있으면
실제 CSS 사다리를 아는 Canvas painter에 맡긴다. 글리프를 숨기거나 글꼴 순서를 바꾸지 않았다.

기존 실제 브라우저 검사 파일에 MIT 합성 TTF 세 개의 관계형 검사를 추가했다. 앞 글꼴의
coverage가 미확인인 경우 Canvas 문자 호출·PNG 보존을 확인하고, 전부 등록한 정상 조건의
윤곽 재생과 원 face의 문자 paint도 확인한다. 수정 전 실제 1 FAIL → 수정 후 통과다.
Print/faux-bold 묶음 4 PASS, 기존 자동 수집 Canvas 묶음 전체 17 PASS/0 FAIL이다.
기존 CI가 해당 파일을 import하므로 CI 호출을 추가하지 않았다. fmt와 WASM Clippy도 통과했다.

같은 입력·Print·글꼴 조건의 HWPX 198쪽은 fresh WASM 100.00%로 복원됐고 review·overlay를
직접 판독했다. HWP 197/198/224쪽도 각각100.00%, outline8쪽99.95%, sample10 629쪽99.96%다.
이 여섯 쪽 결과로 전쪽 통과를 주장하지 않는다. HWP/HWPX 각383쪽의 WASM 전쪽 비교는
이 기록 시점에 실행 중이며, 다른 보호값과 새 devel 수식 문서의 하락도 보정 대상이다.

[생산자·검사·실행 범위](print-predecessor-coverage/validation.json) ·
[수정 전 review](print-predecessor-coverage/before-wasm-p198-review.png) ·
[수정 전 overlay](print-predecessor-coverage/before-wasm-p198-overlay.png) ·
[수정 후 review](print-predecessor-coverage/after-wasm-p198-review.png) ·
[수정 후 overlay](print-predecessor-coverage/after-wasm-p198-overlay.png).

이 단계 전 `975450f54d`까지 커밋한 후 최신 `06181802e5`로173개를 충돌 없이 리베이스했다.
새 후보 `4b55aea528`와 리베이스 전 후보 사이의 변경은 #8793 배포 문서·기록8개뿐이며,
제품 소스·검사·입력·시각 도구는 동일하다. 기존 producer의 근거를 새 실행으로 재표기하지 않았다.

### 같은 producer의 편람 WASM 전쪽 완료

이후 동일 frozen fresh WASM에서 HWP/HWPX 각각383쪽, 총766쪽을 끝까지 산출했다.
각 입력의 전체 쪽 번호가1~383이며 누락·중복·90% 미달과 정본 보호 표시값 하락은 모두0쪽이다.
각 최저 공식 표시값99.96%(130쪽),382쪽100.00%다. 두 문서의 TSV 수치는 같지만
입력·Print는 별개이고 각각 실제 실행했다. 위 validation.json에 전쪽 TSV·producer·입력
manifest와 로그 hash를 연결했다. 다른 보호 문서와 신규 수식 문서 하락의 검증·보정은 계속 남아 있다.

신규 수식 문서의 고정 devel Native21쪽씩도 완료했다. 원본 최저98.79%, 재저장본99.04%이며
후보의17~19쪽 미달은 기존 기준의 미달이 아니다. 두 기준4aa8/0618의 차이가 배포 문서뿐임을
확인했고 실제 고정 Native producer의 source는4aa8로 유지했다. 첫 출력 차이는17쪽 문단의
수식 후속 줄에서 시작한다. 원인 보정 전 점수·독립 PDF·판정 기준을 바꾸지 않는다.


## 2026-11-09: 부분 수식 재조판의 현재 점유·후속 줄 소유 복구

원 해결 #7640 → 실제 통합 #8754 → 독립 한컴2020 Print·고정 devel을 대조했다.
첫 차이의 렌더 문단811은 원본 최상위 문단 번호가 아니라 미주 확장 ID였다. 실제 소유는
section0/문단318/미주0/내부 문단10이며, 네 수식과 저장 세 줄을 직접 대응했다.

부분 재조판 앞줄을 현재 글자 높이만으로 수용해 수식의 점유 높이·기준선을 잃고, paint에서
이미 같은 줄로 확정한 수식 목록을 문단 처음처럼 다시 채웠다. 생성한 후속 첫 줄의 indentation
플래그도 문단 첫 줄로 게시했다. 저장 앞줄의 현재 인라인 점유, 이미 확정한 줄 소유, 후속 줄의
실제 들여쓰기 세 계약을 공통 경로에서 보정했다. 문서/쪽별 수치·임의 여백 예외는 추가하지 않았다.

Native 원본/재저장본 각21쪽 최저는98.78583%/99.03786%, fresh WASM은98.77947%/99.02612%다.
재저장본17쪽 Native86.88111→99.15980%,18쪽42.42→99.04%,19쪽62.83→99.48%로 복원됐다.
원본12쪽96.51→99.88%도 복원됐다. 두 입력 모두 전쪽90% 미달·쪽 누락·중복은0이다.
17쪽의 새 수식 앵커 고정은 양쪽raw98% 이상과 독립 Print 시작점·기준선 판독 후에만 추가했다.
관계형 소유·점유 검사와 독립 Print ±2 CSSpx 고정형 검사는 실제 assertion을 구분해 주석에 적었다.

새 두 검사는 수정 전 의도한 원점 결함으로2 FAIL, 수정 후 standalone2 PASS다.
정식 자동 수집 모듈은 기존7건과 새2건 모두9 PASS/0 FAIL이다(309건은 선택 밖).
같은 컴파일된 suite를 개인 글꼴 파일/Registry 접근을 막고 실행해도9 PASS이며 전체 CoreText 격리
검증으로 확대하지 않는다. 실제 fresh WASM Canvas 묶음은17 PASS/0 FAIL이다.
Mac WASM은 locked wrapper의 no-opt 로컬 대체 빌드이며 루트pkg/Studio public/frozen의 hash가 같다.

추가 영향 대조에서 sample16 세 저장본의17·35·38쪽과 nested3·8·17쪽, 총12쪽은
수식 보정 전후 표시값이 같다. 최초 네 조건은 이관하지 않은 Linux 글꼴 환경 경로 때문에
측정 전에 중단돼 제품 실패로 세지 않았고, 기존 Mac 환경 manifest·공급 경로로 정정해 실제 완료했다.
재저장본2쪽의 base99.68→99.66%와 sample16·nested·outline·sample10의 기존 보호값 미달은
이 단계로 해결됐다고 주장하지 않는다. 전체 통합 회귀·전쪽·게이트·PR 준비는 계속 남아 있다.

[단계 입력·producer·Native/WASM 전쪽 TSV·검사 증거](partial-equation-frame-repair/validation.json) ·
[Native 수정 후 review](partial-equation-frame-repair/after-native-p017-review.png) ·
[overlay](partial-equation-frame-repair/after-native-p017-overlay.png) ·
[WASM 수정 후 review](partial-equation-frame-repair/after-wasm-p017-review.png) ·
[overlay](partial-equation-frame-repair/after-wasm-p017-overlay.png).

이 단계의 Native Skia lib Clippy(`-D warnings`)·fmt·현재 base의 suite manifest48/48·변경 문서 링크 검사도 통과했다.
전체 nextest·나머지 lint/범위 게이트와 최종 보호 TSV는 이 결과로 대신하지 않는다.


## 2026-11-09: HWP3 암시적 쪽번호의 기본 프로그램 소유 복원

HWP3 사양10.14절의 쪽번호 제어는 글꼴을 저장하지 않는다. sample10의 사용자 스타일
`쪽번호`를 canonical `쪽 번호`로 해석하는 진단은 독립 Print와 다른 한양신명조를 골랐으므로
채택하지 않았다. 실제 Native renderer trace에서 암시적 `바탕`은 Windows Batang TTC를
선택했고, 독립 Print의 Haansoft Batang과 숫자·줄표 폭이 달랐다. Native HWP3의 명시 스타일이
없는 기본값만 명확히 해 공통 측정·paint 소유를 맞췄다. 문서/쪽별 좌표 보정은 없다.

sample10 Native/fresh WASM 각각763쪽, 누락·중복·90% 미달0이다. Native 최저99.41%,
보호값 하락0/상승11쪽이며629쪽은 두 경로 모두99.96→100.00%다. WASM 최저99.38%,
보호값 하락8/상승5쪽이므로 전체 복원 완료가 아니다. 수정 전 frozen WASM으로 남은8쪽과629쪽을
다시 출력해7쪽의 기존 표시값 동일·658쪽99.93→99.94%를 확인했다. 전후 픽셀 차이는 자동
번호 영역에만 있으며 gate에서 그 영역을 제외하지 않았다. 본문의 남은 차이는 계속 보정한다.

명시한 쪽번호 스타일의 HWPX2쪽은 Native/WASM 모두100.00%이고 Native 전후 PNG가 같다.
현재93조건의89개 원본에서 형식·canonical 쪽번호 스타일을 감사해 sample10 외 Native HWP3
두 원본에는 명시 스타일이 있음을 확인했다. 이 감사만으로 다른 문서 전쪽 검증을 대체하지 않는다.
새 관계형/고정형3개는 실제 수정 전1 PASS/2 FAIL→수정 후3 PASS이고, 같은 컴파일 검사도
Mac의 개인 글꼴·CoreText 서비스 접근을 차단한 환경에서3 PASS다. 629쪽의 양쪽100%와 독립
Print 원점 확대 판독 이후에만 ±2 CSSpx 검사와 정식 suite 등록을 진행했다.

[전쪽 TSV·producer·정식 실행 결과](hwp3-default-page-number/validation.json) ·
[Native review](hwp3-default-page-number/after-p629-review.png) ·
[overlay](hwp3-default-page-number/after-p629-overlay.png) ·
[fresh WASM review](hwp3-default-page-number/after-wasm-p629-review.png) ·
[overlay](hwp3-default-page-number/after-wasm-p629-overlay.png).


## 2026-11-09: 일반 양쪽 정렬의 자연 대시·낱말 공백 예산 복원

outline8쪽의 첫 대시는 Native595.57333 CSSpx, 독립 Print591.67867 CSSpx였다.
처음 문단 끝 분기를 보정한 후보는 실제9쪽PNG가 모두 같아 제외했다. 실제 대상은 끝 줄이
아닌 일반 Justify 줄이었고, `justified_dash_restoration`이 자연 폭 복원 부족을 대시에만 배분했다.
이미 음수 여백에 쓰는 공백20% 한도로 자연 대시를 수용하되 글자 압축·탭·다른 정렬이면
기존 배분을 유지하도록 공통 경로를 보정했다. 각 런의 자연 폭과 필드·선두 공백 소유는 유지한다.
마지막 줄의 #5830 경로는 변경하지 않았다. 문서/쪽별 좌표·글꼴·판정 기준 변경은 없다.

Native9쪽·fresh WASM9쪽 모두100.00%, 누락·중복·보호 표시값 하락0이다.
Native8쪽99.94275→100.00000%, 첫 대시591.86133 CSSpx로 복원됐고 마지막 낱말 원점도
독립 Print ±2 CSSpx 안이다. 두 경로의 review·overlay와 확대 판독 뒤 고정형/관계형2개를 등록했다.
실제 수정 전2 FAIL→수정 후 최소 글꼴 standalone2 PASS다. 새2건·기존PII 대시2건·앞 단계
쪽번호3건은 같은 최종 코드의 정식 suite에서 최소 글꼴7 PASS/0 FAIL(1015건은 선택 밖)다.

standalone가 갱신되지 않은 `release-test/librhwp.rlib` 별칭을 링크한 실패는 무효로 분리했다.
실제 새 `release-test/deps/librhwp.rlib`를 hash 고정해 재링크한 결과만 위PASS로 인용했다.
역사적 #5830 Print 대조군은 수정 전부터 Native64쪽/PDF65쪽으로 달라 정상 검증으로 쓰지 않았다.
별도 원본의 기존 미달을 새 golden·신규 회귀로 등록하거나 이번 변경의 실패라고 바꾸지 않았다.

새 후보의 Native 영향 대조는 sample16 세 저장본17·35·38쪽과 nested3·8·17쪽, 총12쪽을 완료했다.
아래의 기존 미복원과 sample10 WASM 보호값 미달은 계속 남는다. sample10의 기존 미달8쪽과629쪽
WASM 집중 재측정도 완료했으며 최저99.38%다. outline18쪽 완료로 전체93조건/2994쪽씩의
검증·다른 보호 문서·전체 회귀·실제 소비 경로를 완료했다고 주장하지 않는다.

앞 단계 쪽번호 주석의 `pdftoppm` 표기는 실제 wrapper와 달라 Linux PDFium PNG의 SHA 검증
재사용으로 정정했다. 입력·PNG·TSV·점수·판정은 바꾸지 않았다.

[입력·producer·전쪽 TSV·검사·확대 판독 증거](justified-dash-space-budget/validation.json) ·
[Native review](justified-dash-space-budget/after-native-p008-review.png) ·
[overlay](justified-dash-space-budget/after-native-p008-overlay.png) ·
[fresh WASM review](justified-dash-space-budget/after-wasm-p008-review.png) ·
[overlay](justified-dash-space-budget/after-wasm-p008-overlay.png).

## 2026-11-09: 중첩 셀의 명시 공백 예산과 자연 대시 복원

`issue2007_nested_cell_pagination_42065.hwp` 3쪽의 실제 소유 경로는 Section0/문단2의
바깥 표 control1/cell1/문단3 → 안쪽 표 control0/cell15/문단0이다. 이 문단의 원본
문단 모양25는 attr1=13248, 공백 최소값25%를 저장한다. HWP5 표44의 bit9~15 범위와
독립 한컴2020 Print를 대조했다. 해당 줄의 원본은 한양신명조 HFT이며 Gulim 줄이 아니다.

기존 일반 양쪽 정렬의 자연 대시 복원은 공백20% 한도로만 부족분을 검사했다. 이 줄의
부족4.48 CSSpx는 기본 예산3.73333 CSSpx를 넘지만 원본25% 예산4.66667 CSSpx 안에 든다.
이미 해석한 원문 `condense`를 줄 배분 → 자연 대시 복원 → 같은 공백 압축 계산으로
전달했다. 일반 음수 여백의20% 기본값, 마지막 줄의 #5830 분기, 필드·선두 공백 소유는
바꾸지 않았다. 문서 식별자나 쪽 번호로 분기하지 않는다.

독립 Print의 `제3항` 숫자 원점은563.5345865885416 CSSpx다. 수정 전567.92에서
수정 후563.44로 복원됐으며, 이 문단의 명시 예산만 지운 대조군은567.92를 유지한다.
설치 글꼴에 의존하지 않는 진단2건은 수정 전2 FAIL → 수정 후2 PASS다. 정식 검사는
`tests/cases/nested_declared_word_space_budget.rs`에 고정형 원점 검사와 관계형 예산
대조 검사로 구분했다. 실제 최소 글꼴 nextest는 새 검사2건·기존 대시4건·쪽번호3건의9 PASS / 0 FAIL이다.
선택 밖1337건을 전체 회귀 통과로 계산하지 않는다.

[검증 기록](nested-declared-word-space-budget/validation.json)의 Native/fresh WASM에서
중첩 문서17쪽과 outline 문서9쪽, 각26쪽을 새로 산출했다. 중첩3쪽은 두 경로 모두
100.00%, 전17쪽 최저는 Native98.36%/WASM98.37%다. 나머지 Native16쪽의 점수는
그대로이며 두 문서의 보호 표시값 하락은0쪽이다. outline9쪽은 두 경로 모두100.00%다.
결합 대조12쪽의 PNG는 변경 대상인 중첩3쪽만 달라졌고 나머지11쪽은 byte 동일하다.
독립 PDF와 입력·글꼴·계산 기준을 바꾸지 않았다. Native/fresh WASM3·16쪽의 review와
같은 쪽 overlay, 변경 줄 확대를 직접 판독했다. raw100%와 RGB 픽셀 완전 일치는 구분한다.

Mac fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
빌드 exit0, 브라우저 계약4 PASS를 확인했으며 root pkg/Studio public/고정 검증 pkg의
JS·WASM 해시가 일치한다. 이 집중 결과를 현재 통합 브랜치 전체 검증 완료로 간주하지 않는다.

sample10 WASM의 남은 차이를 분석하기 위해 원 #8397의757쪽 review에서 후보 raw 패널을
재표본화 없이 복원했다. 같은 review의 PDF 패널이 현재 독립 기준 PNG와 byte 동일한
오프셋34px를 확인했다. canonical 재계산은 원 후보99.39720%, 현재99.37893%로 과거
TSV와 일치한다. 원 PDF에서 이미지가 문자를 가리는 현상은 기존 검토에도 기록돼 있다.
원문 문자를 숨기거나 PDF·마스크·판정 기준을 바꿔 보호값을 맞추지 않으며, 원 후보와
현재 실제 글꼴 프로그램·paint 경로 차이를 계속 확인한다. 복원 PNG와 진단 스크립트는
ignored output에만 보존한다.

같은 소스의 최소 글꼴 Native 라이브러리는 root 3852 PASS / 12 ignored, 다른 라이브러리
182 PASS다. fmt, Native 제품·선택 suite Clippy, WASM Clippy, suite manifest 검사와
변경 문서 링크 검사를 통과했다. 새 검사·증적은 이 단계의 한국어 로컬 커밋으로 정리한다.
