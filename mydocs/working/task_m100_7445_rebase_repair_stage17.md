# 단계17: 저장된 대체 글꼴 후보와 선택된 프로그램 구분

## 원 PR·독립 출력·실제 회귀

#1105의 원 해결 PR#1107(merge e77d51f0d549ef0496411b7df86dc04334be5957)은
sample16의 HWP3→HWP5 저장본 쪽 나눔·기호와 정상 저장 LineSeg 재사용을 보호했다.
이슈/PR 본문·댓글·diff·실제 반영 및 로컬 검사 도입 이력을 읽었다.
저장2022 원문의 올바른 계약은23쪽 `Heart Bit` 끝 /24쪽 `구성은` 시작이며 문단460의
내용을 두 조각에서 합치면 원문과 같아야 한다. 이전 후보는 `구`를23쪽에 수용했다.
기존 기대값·쪽수64·원본·LineSeg·낱말 나누기 모드는 바꾸지 않았다.

유효한 독립 한컴 Print `pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf`를 재사용했다.
Hancom PDF1.3/Hwp2020이며, 별도 파일 `hwp3-sample16-hwp5-2022-2020.pdf`의 Cairo 출력과
구분했다. 그 Cairo 파일은 이번 독립 기준이 아니다.

- 원문 SHA256: `fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`
- 독립 PDF SHA256: `6737b57af568a591eb073eeb27054e10aaf7587cbdfd6f18a06d1033b4025a19`
- 고정 base:4a07d3ae4dd3255a57fb0166b2b2a2dc335a61fa.
- 검증 runtime:c6d27d2c5966473498e573182b7fd52bcadd295a 위의 style_resolver 수정.
  최종 style_resolver SHA256:`711dc78d99a7278749d0d432b6f61000504990ce9249eabda2cecb07318fd1ab`.

## 원인·일반 보정과 반례

로컬 df88f3d5699/C442는 개인정보의 실제 Palatino Type0/CID `<=>` 폭500을 보호하면서,
문서에 type1 substFont가 **있기만 하면** 선택된 TTF 프로그램으로 취급했다.
그러나 canonical 이름 결정에서 Batang은 보조 후보로 남고, 독립 Print는 원본 Type3를 유지한다.

같은 원문 HWP3와 저장2022 Print의 `/T3`는 S525/slash605/W998 및 CharProc SHA가 같다.
별도 한컴2020 대조군은 기존 입력의 substFont Palatino→Batang만 바꿨고, job
37a92dee-e7ff-4ca6-9ed4-9bacfdeba06b/11.0.0.9136/Print method0/
hancom2020_pdf_driver_one_up/preprocess none에서 Type3 `<=>`605를 확인했다.
기존 Palatino 대조군은 Type0/CID500이다. 이 합성 Print는 프로그램 구분의 근거이며
미검증 합성 페이지 배치를 정답으로 고정한 증거가 아니다. 원시 글꼴 표본·중간 PDF는 output에만 둔다.

같은 canonical 치환 결과로 대체 후보가 실제 주 공급자로 선택됐는지 구분했다.
명시 Palatino TTF·호출자 TTF 환경·내장 글꼴 제외·실제 HFT와 미측정 글자 A/중점의
기존 공급자는 유지한다. 전체 HCI 폭표·새 글꼴·폭 상수·문서/쪽 예외를 추가하지 않았다.

생산·소비 경로는 style_resolver의 original decision과 언어별 original glyph face →
text_measurement의 언어 슬롯/전진 결정 → 공통 줄 채움과 TextRun 원점이다.
별도 paint x 보정 없이 동일 폭을 소비한다. 수정 전 `S/W` 폭26.88/후속 한글x163.573333,
수정 후 폭31.04/x167.733333으로 독립 Print 폭31.04/한글x167.68에 복구됐다.

## 같은 원문 전64쪽 TSV와 직접 판독

canonical192DPI/compat2022/PDFium smooth_rgb_v1/raw2px90%, capture8.
16쪽 묶음 각각 canonical TSV를 생성하고 같은 원시 행을 순서대로 연결했다.
PNG는 바이트 동일한 hardlink로 집계했다. 완료된 본 실행 소유 생성 SVG만 SHA manifest 후
정리했으며 삭제된 SVG를 재사용 가능 증거로 보지 않는다. PNG·TSV·로그·원본/PDF는 보존했다.

| 경로 | 쪽수 | 최저 raw | 최저 쪽 |90% 미만|
|---|---:|---:|---:|---:|
| fixed4a07 Native |64/64|59.36766%|40|50|
| 수정 전552 Native |64/64|93.26891%|41|0|
| 수정 후565 Native |64/64|94.59411%|40|0|
| fresh568 WASM |64/64|94.60213%|40|0|

[Native](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/source-font-selected-program/native-whole64.tsv) ·
[fresh WASM](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/source-font-selected-program/wasm-whole64.tsv) ·
[전후/base 비교](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/source-font-selected-program/native-before-base-candidate.tsv)

수정 전 대비45쪽 개선,0.1%p 이상 하락0쪽이다. 가장 큰 하락은43쪽−0.06050%p다.
fixed base 대비0.1%p 이상 하락도0쪽이다. 이전 결과를 새 TSV로 인용하지 않았다.
Native/fresh WASM23·24쪽과 최저40쪽,기존 기호59쪽 review/overlay를 직접 판독했다.
클러스터 문단의 앞 끝/다음 시작,다음 요구사항,표·기호의 소속·원문 보존을 확인했다.
40쪽 일부 문장의 기존 자간·줄 구성과 미세 잉크 차이는 남으며 완전 일치라고 하지 않는다.

## 기존 검사 보강·수정 전후 실행

- 기존 original_font_program_ownership 검사에 implicit/Batang 후보/선택 Palatino를 함께 넣었다.
  원문 export 바이트 보존·미측정 글자의 공급자·호출자 명시 TTF를 유지한다.
  동일 최종 검사:frozen552에서1FAIL →565의 전체 해당 파일21PASS.
- 기존 #1105 검사에 문단의 두 쪽 소유·내용 순서·누락/중복·후속 문단 검사를 유지했다.
  전64쪽 검증 후,경계의 두 완료 쪽 중 양쪽 최솟값이 높은23쪽을 선택했다.
  23쪽 Native99.71668/WASM99.71773%,24쪽99.46509/99.47145%다.
- 독립 Print23쪽의 S/W x136.64,뒤 한글x167.68,같은 줄 기준선1040.32를
  96dpi CSSpx±2로 보호한다. px 범위로 대상을 고르지 않고 원문 문단/내용으로 찾는다.
  같은 최종 source는 수정 전552에서 한글x163.573333으로 실제1FAIL,
  수정 후565에서x167.733333으로1PASS다. 기준선은1040.333333이다.
- 정식 prepare 후 집중 nextest:53실행53PASS/0FAIL/2213선택 제외,
  실행73.415초/전체423.572초/exit0. 기존 개인정보·수식·캡션·쪽번호·zero-size·offCanvas 대조군 포함.
- px를 보강한 최종 #1105 source의 prepare/정식 재실행:1PASS/245선택 제외/0FAIL,
  실행0.341초/전체12.176초/exit0. 앞53건과 중복되므로54개 고유 검사로 합산하지 않는다.
  px 보강은 테스트만 바꿔 runtime565/568의 소스·산출물은 동일하다.

## 빌드·실제 초기화·잔여

Native565:locked release-test/native-skia/bin+lib/jobs8,150.518초/exit0.
CLI SHA256:`b5243eb30bd3147936571885d01c12080223e98cd574c6520efa21ffe9ebc78f`.

fresh WASM568:locked wrapper web/pkg/no-opt/jobs8,301.667초/exit0.
Mac 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.
JS SHA256:`23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591`.
WASM SHA256:`242ac0f01772c636d84e25712010fadbb325543f8ffb787155ddfe99f3afad2d`.
root pkg·Studio public 및 실제 Studio7736 초기화 JS/WASM GET200 응답 hash가 같고
캐시 미사용/errors0/초기화true다. 이는 문서 paint·CanvasKit 준비·actual36 완료가 아니다.

[53건 로그](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair570-selected-font-focused-gates/nextest.log) ·
[최종 px 검사 전후](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair574-final-server-before-after/results.json) ·
[최종 정식 검사](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair575-final-server-official/nextest.log) ·
[실제 초기화](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair571-selected-font-browser-init/result.json).

Science3쪽 fixedbase 대비−0.32810%p,저장 캡션 번호와 최종 전체 nextest·범위 게이트·
전체 대상 시각/actual36은 남아 있다. 이번 결과는 #1105와 관련 프로그램 보정의 검증이며
전체 PR 준비 완료가 아니다. 지침·원격 상태는 변경하지 않았다. 임시 스크립트·JSON·로그는 output에만 둔다.
