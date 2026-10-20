# #7445 보정24 — 새 개체의 번호 범주와 캡션 소유 줄 복구

부모 `e8145da134f64bd5a04d864531dc9aca1318aa99`, 고정 base
`2e3b4f880bd11d35398e6ea0100846611f13a72d` 위 로컬 보정이다.
전체 #7445 해결 또는 PR 준비 완료를 뜻하지 않는다.

Issue: #7445, 관련 #1470 / #2069 / #5864.

## 원 PR과 독립 계약

#1470 → [원 PR#1475](https://github.com/edwardkim/rhwp/pull/1475) →
`c0e93033642` / merge `7f24dfa096544c01f228136ef069547f9d02ac72`,
#2069 → [원 PR#2103](https://github.com/edwardkim/rhwp/pull/2103) →
`200f64d858a` / merge `82567a54ec8eac074a18e1a1c049f3e3e2770105`의 본문·diff와
현재 검사를 대조했다. 당시 검사는 캡션 삭제 뒤 나머지 번호를1부터 줄이고
저장 number 캐시와 표시 assigned_number를 같은 값으로 기대했다.
Stage18의 독립 번호 범주 검증과 현재 parser는 개체가 순번을 소유하며 저장 캐시는 덮지 않는다.

독립 Hancom2020 Print는 공개 API로 새 표/그림3개를 만든 뒤 캡션을 붙이면1·2·3,
가운데 캡션만 지우면1·3이다. 개체 자체는3개가 남는다.
새 표/그림의 기본 번호 종류는 TABLE/PICTURE이며, 파싱한 NONE을 일괄 변환하는 계약이 아니다.
[한컴 캡션 도움말](https://help.hancom.com/hoffice100/ko-KR/Hwp/insert/caption_memory.htm)의
번호 종류와 캡션 존재를 구분하고 실제 Print로 표시 결과를 확인했다.
원본 번호 캐시를 표시값으로 다시 쓰거나 기존 parser의 NONE 계약을 되돌리지 않았다.

## 실제 원인과 공통 소비

1. 새 개체 생성기는 번호 범주를 NONE으로 남겼다. Table2경로와 Picture3경로에
   의미 enum/원시26–28bit를 함께 설정했다. 그림의 HWP legacy bit26 mirror도 동일하게 설정한다.
   mirror를 누락한 중간 후보는 HWP 저장 뒤 범주가NONE으로 돌아가1·1·1이 됐고,
   HWPX만 통과했으므로 수용하지 않았다. Table의 common margin도 원시/별도 outer margin과 맞췄다.
2. `inline_control_occupied_size_hwp`는 표의 측정 총높이에서 캡션 높이를 빼고 있었다.
   본체+위/아래 캡션 점유 결과를 발행하고, 빈 텍스트 host도 같은 helper를 소비하게 했다.
   캡션 속성 변경 뒤에는 캡션 문단뿐 아니라 host 줄과 후속 문단의 vpos도 같은 메트릭으로 재조판한다.
3. 저장 출력8조건이 raw100이어도 편집 직후 둘째 표는185.8CSSpx로 독립 Print193.228보다 위였다.
   common margin 정합만으로는 복구되지 않았다. 앞 TAC 줄의 prefix를 다시 더하는 측정과
   본체/캡션을 따로 쌓는 paint fallback이 달랐다.
   유효한 source 줄 소유·구성 높이·측정 점유 높이가 일치하는 독립 TAC 줄에서 공통 pen/end를
   `InlineBoxPlacement`로 발행하고 기존 배치가 그 원점·끝을 소비하도록 했다.
4. 편집 세션만 공통 결과를 쓰던 후보는 왼쪽 캡션의 HWP/HWPX 재열기 후 표를3.8CSSpx 위로 놓았다.
   현재 캡션 소유 줄도 같은 검증된 점유 질의를 사용한다. source partition/cell vpos가 dirty이거나
   같은 줄의 다른 개체·보이는 본문·높이 불일치가 있으면 적용하지 않는다.
   원본의 캡션 없는 저장 줄 검사와 각주 제외 조건은 유지한다.

경로는 공개 삽입/속성 API → 번호 범주/캡션 IR → composer의 전체 점유 높이 → host LineSeg →
TypesetEngine의 소유 줄/예약 높이 → InlineBoxPlacement → table paint/Native/SVG/fresh Canvas다.
별도 문서명·쪽수 예외, 좌표 clamp, 허용치 확대는 넣지 않았다.
실제 producer ledger의6 runtime 파일 SHA는 아래 asset README에 연결한다.

## 전쪽 시각 검증과 직접 판독

[전체 TSV·대표 review/overlay·source와 build SHA](../pr/assets/issue7445/completed-20261009/stage24-20261021/README.md).
[원본10개·독립 PDF SHA/job](../../tests/fixtures/issue1470_authored_caption_numbering/README.md).
기존 Hancom2020 blank에서 공개 API로 생성한5논리조건×HWP/HWPX다.
각1쪽이며 Native/fresh WASM 각각10/10쪽, canonical96DPI/compat2022/PDFium smooth_rgb_v1/raw2px
최저100.00000%,90% 미달·누락·중복0이다. 같은 입력 SHA에 대응하는 독립 Print PDF만 재사용했다.
각 PDF는 Hancom2020 11.0.0.9136 / managed direct DLL host / Print method0 / one-up이다.
HWPX 전처리none과 HWP scripts_disabled를 실제 job별로 구분했다. 요청 제한은1800초다.

Native 전체 캡션/중간 캡션 삭제/왼쪽 캡션, fresh WASM 왼쪽 캡션/그림 중간 캡션 삭제의
1쪽 패널을 직접 읽어 개체3개와 캡션1·2·3 또는1·3의 소유·앞뒤 순서·간격을 확인했다.
그림 대조군은 의도적으로1px 투명 그림이며 그림 자체의 모양 검증으로 주장하지 않는다.
가는 검정 괘선의 래스터 진하기는 다르지만 대응한 PDF0.36pt와 SVG0.48CSSpx는 같은 선 설정이다.
raw100%를 엄격RGB 일치라고 표현하지 않는다.

Mac fresh WASM은 locked wrapper web/pkg/no-opt 로컬 대체 빌드다.
루트 pkg/Studio public hash가 같고, 새 Studio7736 초기화의 JS/WASM 실제 응답도918에서 확인했다.
Vite JS는 WASM URL rewrite/source-map 원문을 frozen JS와 대조하고 WASM은 byte hash가 같다.
초기916의 응답 body 관측 실패는 수용하지 않았으며 CDP durable response 기록으로 새 실행했다.
이는 전체 실제Studio/CanvasKit paint 또는 actual36 완료가 아니다.
공개 Native/fresh WASM 편집 연산의 직후 SVG가 같고, 저장 입력10개의 byte SHA와
재열기 전후 control layout도911에서 일치했다. SVG 문자열 일치를 glyph 시각 판독으로 대신하지 않았다.

## 정식 회귀와 기존 기대값 정정

같은 최종 새 검사 source를915에서 frozen867로 실행하면0PASS/3FAIL,
중간900은2PASS/1FAIL(저장 후 왼쪽 캡션 표의 독립 위치 잉크0/555), 최종908은3PASS다.
새 검사 source SHA256은 `fac1e647925588215f45cb8b7cf5f9cbe1faf9194a49021a68a6782909161b95`다.
양쪽100%와 직접 판독을 마친 뒤 `tests/cases/issue_1470_authored_caption_numbering.rs`에 반영했다.

- 원문 개체3개·캡션 표시 순서·누락/중복·캡션 삭제 후 번호1·3과 저장 캐시 보존.
- HWP/HWPX 저장 왕복 뒤 같은 원문 개체/번호/배치 유지.
- 캡션은 자기 표 아래에 있고 후속 표와 겹치지 않는 관계.
- 독립 Print의 둘째 표 윗선193.228CSSpx와 왼쪽 캡션 표 윗선225.833333CSSpx를
  실제 Native Print PNG에서±2CSSpx로 검사한다. 각각 완성1쪽의 양쪽 raw100을 우선 선정했다.
- 기존 #1470 unit3검사는 원본 개체와 AutoNumber 슬롯·캡션 속성·삭제 검사를 유지하고,
  독립 계약과 충돌한 번호 재정렬/캐시 동치 기대만 정정했다.
- 보존한 새 입력10개를 독립1쪽 PDF에 대응하는 oracle page-count 원장에도 등록했다.

정식917 실행은 새 case3PASS(1.164초), #1470 unit12PASS(0.036초),
기존 개체 번호 소유5PASS(0.873초), 캡션 없는 저장 줄/바깥여백2PASS(0.032초)로 총22PASS/0FAIL이다.
각 실행의242/4041/254/249는 필터 미선택이며 전체 회귀 skip 수로 합산하지 않는다.
`--prepare` 후 실제 자동 suite026에서 새 case3개가 실행됐다.
manifest check와 unit tiers는 고정 base2e3b4f88 대비 통과했고 fmt/diff check도 통과했다.
관련 Cargo는 순차 실행하고 jobs8/test-threads8, 캡처 배타 lock을 유지했다.
WASM 이후 변경한 source는 cfg(test) unit와 integration 검사뿐이며 runtime6파일 SHA는 그대로다.
새 입력 oracle 원장은 현재 CLI1쪽 실측과 연결했으며, 전체 partition/전체 nextest는 최종 branch 게이트에 남긴다.

## 남은 작업

이 단계의 국소 결과로 전체 회귀 통과를 주장하지 않는다.
CSAP 제목/표 간격, 머리말 TAC 그림 높이, 과학 본문 기준선, #8064 Native 품질 하한
PrEP1쪽·법령2/13/36쪽을 계속 보정한다. 최신 전체 nextest·범위별 lint/build/Skia·전체 corpus/base TSV와
actual36/공유 그림 실제 소비 경로도 남아 있다. PR은 승인 전 생성하지 않는다.
로그·중간 JSON·SVG·실패 실행은 ignored output의repair863–919에만 보존한다.

문서 검사는 새 안정 링크의 실제 경로를 확인했다. 장기 문서 metadata 검사는기존16오류이며,
해당 검사 경로가 부모HEAD와 byte/파일 집합상 변경되지 않아 이 단계 신규 오류는0이다.
이를 저장소 전체 metadata 통과로 기록하지 않았다. 원시 로그는repair919에만 남긴다.
