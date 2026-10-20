# 단계18: 개체 번호 범주와 본문 원문 슬롯 소유 복구

## 선행 계약과 독립 근거

#5864 → 원 PR#6006 → 통합#6010/fd2231a3779580859270be5b232201956dedcd3f →
실제 반영077e46c0cd00208b86a1fcac4febbde1bd855f56의 본문·논의·diff를 읽었다.
번호 범주는 flags26–28의 열거이며 NONE을 일괄 PICTURE로 바꾸는 계약이 아니다.
기존 bit29/왕복 검사는 유지했다. 이번 반례는 범주를 이미 읽은 뒤 표시 번호를
필드마다 증가시키거나 저장 캐시로만 표시하는 별도 소비 오류다.

독립 Hancom2020 Print(method0/one-up/preprocess none/Hancom11.0.0.9136)로
NONE, PICTURE, 혼합, 같은 캡션 두 필드, 다른 번호 종류, 시작7, 앞/뒤 본문 필드를 대조했다.
모두16쪽이고 입력 생성기는 기존 문단·저장 줄을 유지한다. 여분 문단을 추가해17/18쪽이 된
초기 실험과 직접 native export의 개방 거절 파일은 최종 수용 입력에서 제외했다.

## 생산·측정·paint 공통 소비

- parser/auto_numbers는 실제 소유 개체에서 범주 순번을 한번 증가시키고 캡션 필드는 그 번호를 반복한다.
  캡션 없는 그림도 센다. NONE은1이며 원본 number 캐시는 표시값으로 덮어쓰지 않는다.
  본문 필드는 현재 번호를 읽고 순번을 증가시키지 않는다. HWPX에서는 개체 번호만 처리하여
  각주·쪽 번호 캐시는 보존한다. HWP3의 기존 필드 순회 뒤 암묵적 그림 범주를 enum/원시 비트에 함께 정규화한다.
- composer의 기존 원문 placeholder 투영을 PICTURE/TABLE/EQUATION에도 사용해 본문 필드를 표시한다.
  source 문자는 그대로 두고 display_text만 사용한다.
- Paragraph::control_utf16_positions가 유지된 번호 공백의 원점에서8units를 다시 빼던 것을 고쳤다.
  기존 control_owns_char/control_text_positions와 같은 소유 판단을 사용한다.
  이를 composer가 같은 슬롯으로 소비하고 공통 측정·TextRun paint까지 전달한다.
  좌표 clamp·문서명/쪽수 예외·추가 공백 삭제·paint x 보정은 넣지 않았다.

중간602에서는 텍스트1/1은 표시됐지만 번호 뒤 원문 공백이 남아 둘째 번호가194.88CSSpx로 밀렸다.
607은 슬롯을 올바르게 소비하여 독립 PDF188.217997CSSpx의±2px 안으로 복구한다.

## 전쪽·정식 회귀 증거

[전체 입력/PDF SHA·전쪽 TSV·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/object-number-ownership/README.md).
부모df52e39848f65fccb973e7bf846bb6396a33df23 위7개 소스 변경의 Native607/fresh610이다.
canonical PDFium smooth_rgb_v1/192DPI/compat2022/raw2px90%,12입력×16쪽×2경로=384쪽을 새로 산출했다.
원본 최저 Native95.34478/WASM95.33483%, 나머지11입력 최저99.02719/98.92066%,90% 미달0쪽이다.
전쪽 수와1..16 순서·누락/중복을 확인했다. 직접 판독은 asset README에 실제 본 쪽만 기록했다.
본문1쪽은 양쪽99.03275/98.92675%로98% 조건을 충족하여 독립 PDF의 번호 x 두 개를±2CSSpx로 보호한다.
원문 placeholder 소유·필드 수·캡션/본문 순서와 저장 왕복 관계 검사를 함께 유지한다.

새 tests/cases/issue_265_object_number_ownership.rs는5검사다.
최종 소스 그대로 frozen602에서4PASS/1FAIL(실제 둘째 번호 x194.88),607에서5PASS다.
최종 정식 nextest5PASS/0FAIL,239선택 제외,1.838초. 파생 manifest는--prepare 후 동적 배정으로 실행했다.
관련86검사도86PASS/0FAIL,2667선택 제외,1.356초이며 upstream#7930 clipboard와#8085 unmarked HWPX 저장을 포함한다.
후속 테스트 정리 후 새5검사를 다시 통과했고 나머지81검사의 제품 소스는 같다.
초기 runner의-j/--test-threads 중복 거절과 fixture 확장자 오기는 최종 FAIL→PASS 증거로 사용하지 않았다.

fresh WASM은 Mac locked wrapper web/pkg/no-opt,270.094초 exit0의 로컬 대체 빌드다.
JS SHA23f085e36c78d952ddac0d91faed9752f33ee1124db78e290382687410917591,
WASM SHA477db4ef99b942201fbf9dcae7a2e74fbb7484f3c85c01091bd4403b6377a08e.
루트pkg/Studio public/Studio7736의 실제 캐시 없는 초기화 응답이 같은 빌드임을 확인했다.
초기 host CDP 모드 오류는 headless로 정정했다. 이것은 actual36 문서 paint 완료가 아니다.

## 기준 하락과 남은 작업: PR 보류

고정4a07 renderer 컴포넌트를 현재2e3b4f88과 소스 대조하고 같은12입력·PDF·글꼴로
Native/WASM 각각 전192쪽을 다시 산출했다. 그 사이 src 변경은clipboard.rs뿐이며,
이를 새 current-base 전체 빌드라고 주장하지 않는다. 실제 WASM 로드 글꼴 집합도 입력별 동일했다.
0.1%p 이상 하락은27관측이며 원본13쪽×양쪽26건+시작7의 Native8쪽1건이다.
최대 하락은 원본 Native6쪽4.07228%p다. 번호 기능 복구·90% 통과로 이 하락을 면제하지 않는다.

독립 원본3쪽 Print stream은 꼬리말 그림 외곽을 B(fill+stroke)로 그린 뒤 CTM0.120을 사용한다.
저장본은 외곽이 없어 정밀0.119935/-0.119869를 유지한다. 명세 그림 옵션offset14 bit0의
테두리 의미가 현재 HWP3 parser에서 누락돼 기존 Print 상태 전환에 전달되지 않는 경로를 추적 중이다.
이 공통 원인과 두 자리 캡션의 배치를 추가 보정한다. 별도로 exam_science3쪽 기준 하락도 남아 있다.
최신 전체 nextest·범위별 전체 게이트·전체 실물 Native/WASM·실제36경로는 아직 완료하지 않았으며,
PR 준비 완료·외부 게시 승인·전체#7445 해결을 주장하지 않는다.

로그/진단 원장: ignored output의repair611,617,624–631. HFT 표본·글꼴·로그·중간 JSON은 커밋하지 않는다.
