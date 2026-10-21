# #7445 보정23 — Print JPEG source 창과 실제 그림 축소 경로 복구

Issue: #7445, 관련 #2006 / #7642, 품질 보존 기준 PR #8064.
parent `37aff1d19e5a724b502c2f4e0a8e6b389c05c892`, base
`2e3b4f880bd11d35398e6ea0100846611f13a72d`. 승인된 로컬 보정 체크포인트다.
전체 브랜치의 회귀·소비 경로·PR 준비 완료를 뜻하지 않는다. 원격 변경은 하지 않았다.

## 원 계약과 독립 근거

기존 PrEP 검사는 원본 BinData4의 물리15쪽 단독 소유, 그림→캡션→후속 본문의 순서와
독립 Print에 보이는 회색 내용 밴드를 보호한다. 원본·저장 형식과 기존 assertion을 유지했다.
원PR8064의 accepted Native 품질 TSV도 갱신하거나 완화하지 않았다.

독립 한컴2024 Print는 같은 `samples/issue2006/1790387_prep_final_report.hwpx`의140쪽이다.
PDF SHA는 `9580bb332813b70f27a367e79ad735b238bcd716ff51b849e716f7ddd1e60af0`이며
기존 `pdf/issue2006/1790387_prep_final_report-current-print-2024.pdf`를 사용했다.
817의 clip/Do 행렬과831의 독립 JPEG 자원·원본 source 대조에서 다음 계약을 확인했다.

- 앞쪽을 자른 그림 축의 시작은600dpi clip 장치 경계의 floor다. 뒤쪽만 자르거나 자르지 않은
  축의 ceil 원점은 유지한다. 너비/높이의 장치 반올림과 물리 종이 CTM도 유지한다.
- JPEG source는 원점과 길이를 각각 정수 절삭한다. 끝점을 기준으로 완전히 내포된 창을
  고른 초기 가설은137쪽의2989px 자원을2988px로 줄여 기각했다.
- 같은 source 창을 crop으로 전달할 때와 미리 잘라 전달할 때 축소 필터가 갈렸다.
  독립 내용의 선이 빠지는 원인이며, decoded PDF를 제품 자원으로 대체하지 않았다.
- 15쪽의 얕은 축소는 기존 Print 재구성 marker가 소유한다. 모든 축소에서 평균 필터를
  우회한 초기 실험은 채택하지 않고 기존 modest-reduction 조건에만 적용했다.

## 생산부터 실제 소비까지

`ImageNode 원문 crop/imgDim → PagePaintContext의 물리 image_frame → 복제 ImageNode의
정수 source 창 → 기존 tone 변환 → SourceWindow PNG → PaintOp → backend replay`다.
원본 Document IR/BinData/레이아웃 좌표는 수정하지 않는다.

`print_crop.rs`는 원본 JPEG의 정수 source 창을 한 번 만들고 crop/reference를 복제 paint에서
제거한다. 이 PNG는 최종 장치 표본이 아니므로 Native·Canvas의 기존 전체 source 축소 필터를
소비한다. SourceWindow 정규화는 `OutputDevice`/`BakedWatermark`와 비대상 fill mode를
제외하며, screen/다른 compat의 원본 crop 경로도 유지한다.
JSON의 lazy original bin key로는 파생 창을 얻을 수 없으므로 실제 PNG bytes를 싣는다.
내용 기반 캐시와 창 bounds가 다른 그림을 구별한다. SVG도 같은 resolved 자원을 소비한다.

35Mpix 원본을 잘라24Mpix 자원으로 만드는 정상 경로까지 GPU 원본 한도로 막던 부분은
CPU source 준비 한도64Mpix 안에서 처리했다. 실제 발행 PNG는 기존8192px/32Mpix 검사를
그대로 통과해야 한다. 큰 원본 전체를 지원 대상으로 바꾸지 않는다. CanvasKit의50k 작업·
decode·필수 글꼴 guard는 완화하지 않았다. 정적 정책의 지원 판정은 실제 CanvasKit 화면
통과 증거가 아니며 해당 실제 소비 검증은 아직 남았다.

## 같은 source의 전쪽 산출

Native844와 fresh WASM847은 `native-source.json`의8개 제품 파일 SHA가 현재 source와 같다.
WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
Docker 최적화 결과가 아니다. root pkg와 Studio/public 파일 hash가 일치한다. 실제 Studio
초기화 응답 hash와 actual36은 별도 미완료다.

canonical PDFium smooth_rgb_v1,96dpi,raw2px90%, 문서별 compat·입력/PDF·글꼴 조건을 고정했다.
8쪽 묶음/캡처 총8/배타 lock으로215쪽씩 새 TSV를 산출했다. 완료 SVG는 SHA manifest 후
정리했으며 PNG/TSV/log는 보존했다. 삭제 SVG를 재사용 가능 증거로 쓰지 않는다.

| 조건 | 쪽수 | Native 최저/쪽 | fresh WASM 최저/쪽 |90% 미만|
|---|---:|---:|---:|---:|
|PrEP|140/140|94.75150% /27|94.22354% /27|각0|
|palette 합성 대조|1/1|99.97513% /1|99.97512% /1|각0|
|가상융합 법령 대조|74/74|93.69360% /36|93.46860% /36|각0|

PrEP59/65/72/114/124쪽 Native raw100%,125/129/137쪽은99.99727/99.99845/99.99686%다.
72쪽 양쪽 raw100%이지만 엄격 내용 pixel 일치는63.75%다. 직접 review/overlay에서 내용과
24개 긴 괘선은 보존됐고1px 수준의 선 위상 차이가 남는다.15쪽은98.40426/98.40390%이며
회색 내용 밴드·캡션·후속 본문을 확인했다. 테두리 진하기와 미세 가장자리 차이는 남는다.

[전쪽 TSV·source/입력/PDF/글꼴/producer·회귀 증적](../pr/assets/issue7445/completed-20261009/stage23-20261021/verification.md)

[Native72 review](../pr/assets/issue7445/completed-20261009/stage23-20261021/native-prep-p072-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage23-20261021/native-prep-p072-overlay.png) ·
[fresh WASM72 review](../pr/assets/issue7445/completed-20261009/stage23-20261021/wasm-prep-p072-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage23-20261021/wasm-prep-p072-overlay.png)

[Native15 review](../pr/assets/issue7445/completed-20261009/stage23-20261021/native-prep-p015-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage23-20261021/native-prep-p015-overlay.png) ·
[fresh WASM15 review](../pr/assets/issue7445/completed-20261009/stage23-20261021/wasm-prep-p015-review.png) ·
[overlay](../pr/assets/issue7445/completed-20261009/stage23-20261021/wasm-prep-p015-overlay.png)

## 정규 회귀와 독립 기대값

기존 자동 수집 경로 `tests/cases/`와 manifest prepare/`run-rust-test.mjs`를 사용했다.
generated suite 번호는 고정하지 않았다. 모든 Cargo는 순차/jobs8, tests8이다.

- `native_output_device_surface`: 기존 검사를 유지하고 정수 source 창·crop와 미리 자른
  같은 source의 실제 Native PNG·lazy JSON bytes·큰 source의 작은 창/큰 전체 거부를 추가했다.
  합성 입력은 실행 때 생성하며 공개 source/최종 PNG를 검사한다. source IR 불변도 확인한다.
- 위 세 경계는 각각 수정 전 실제 FAIL을 남겼다(832/839/843). 정적 CanvasKit 정책에서
  큰 전체는 unsupported 수가 아니라 hidden-overlay violation으로 막히는 것이 기존 계약이다.
  첫 기대값 오류851/854를 protocol에 맞춰 정정했고855는 실제 PASS다. guard 변경은 없다.
- 원15쪽 검사는859에서 수정 전 회색 밴드1개로1FAIL→수정 후1PASS다. 기존 소유·순서와
  ≥5밴드 검사를 유지했다. 전140쪽 양쪽90%·15쪽 양쪽98%와 직접 판독 뒤 독립 image Do
  프레임±2CSSpx를 추가했다.
- 새72쪽 검사는 두 backend raw100%와 전140쪽90%를 먼저 확인한 뒤 추가했다. 원문
  BinData20의 section2/para48/control0 소유·140쪽 중 단독 출현·최종 Print 프레임을 확인한다.
  독립 PDF에서 측정한24개 긴 괘선±2CSSpx 안의 실제 Native 잉크를 검사한다.
  856에서 수정 전 x158 선8/914행으로1FAIL→수정 후1PASS다.
- 정규 Native output suite857은4PASS/0FAIL, 최종 PrEP suite860은2PASS/0FAIL이다.
  최종15쪽 프레임 추가 뒤 prepare와 PrEP2검사를 다시 실행했다. fmt/diff check도 통과했다.

## 남은 품질 보존과 다음 작업

최종215쪽 Native 결과를 canonical accepted-quality 검사861로 다시 판정했다.
palette는 통과하지만 PrEP1쪽0.62299%p와 법령2/13/36쪽0.15331/0.18072/1.59272%p
하락은 계속 `re_review_required`다. 이번 그림 개선으로 상쇄하거나 baseline을 낮추지 않았다.

표지의 HYHeadLine81/81과 Dotum169/169 glyph outline/advance는 독립 PDF 내장 프로그램과
현재 Mac face가 일치했다(837/850). Linux 대체 face 실험은 완전히 일치하지 않았고 현재
사용자 글꼴을 교체하지 않았다. 글꼴 부재로 단정하지 않고 실제 원점/paint 소비를 계속 추적한다.
복사한 글꼴·PDF 내장 프로그램·진단 스크립트·로그는 ignored output에만 보관한다.

전체 nextest의 나머지 캡션3건/CSAP 간격/머리말 그림 검사, 과학 기준선과 위4쪽 품질,
전체 영향 범위의 base/head TSV·필수 lint/build/Skia/manifest/unit tier·실제 소비 검증은 남았다.
이 단계의6개 정규 검사·215쪽씩 결과를 전체 브랜치 통과로 재인용하지 않는다.

## 생성 산출물 정리

2026-10-21 현재 증적 정책에 따라 이 단계 생성 JSON5개는 ignored output의
`repair921-stage23-generated-json-relocation/`로 SHA를 유지해 옮겼다.
최종 지표·소스/입력/PDF/build SHA·품질 보류와 판독 한계는
[안정 Markdown 요약](../pr/assets/issue7445/completed-20261009/stage23-20261021/verification.md)에 남겼다.
원본·PDF·대표 PNG·전체 TSV·구현·회귀 source는 변경하지 않았다.
다른 단계의 생성 JSON42개는 소비 경로와 영구 기록 대체를 별도로 확인해야 하며 전체 정리 완료로 보고하지 않는다.
