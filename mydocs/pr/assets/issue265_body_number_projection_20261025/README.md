# #265 본문 개체 번호의 공통 표시 복원

모델이 할당한 현재 개체 번호를 캡션뿐 아니라 본문에서도 원문 자리표에 투영한다.
쪽·총쪽수는 실제 페이지를 아는 후단에서 계산하며 공통 투영에서 제외한다.
입력과 독립 한컴 2020 Print는 기존 Git 파일을 재사용했다.
[실행·입력/PDF·전체 TSV·생산물 원장](evidence.json)에 SHA와 명령을 기록했다.

| 문서 | 후보 Native 최저 | 후보 fresh WASM 최저 | 비교 쪽수 |
| --- | ---: | ---: | ---: |
| 원본 | 99.46503% | 99.54115% | 각각 16 |
| 본문 필드 | 99.61406% | 99.60389% | 각각 16 |
| 캡션 중복 필드 대조 | 99.61140% | 99.60125% | 각각 16 |

두 backend의 총 96쪽 모두 독립 PDF와 전체 쪽수가 같고 90% 미달은 없다.
base 생산 소스는 `60972534680a0c5e51e54ec189b92e252e81a867`이며 정책 기준은
`9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`다. 두 SHA의 runtime·Cargo·font asset·
WASM wrapper 변경이 없음을 확인했으며 생산 소스를 정책 SHA로 바꾸어 적지 않았다.
고정 base 대비 페이지별 최대 하락은 Native 0.01987%p, fresh 0.00736%p다.
소수 원문을 Decimal로 비교했고 2%p 초과는 양쪽 모두 0쪽이다.
평균이나 문서 최저끼리 비교한 값이 아니다. 모든 원본 TSV는 `base/`, `candidate/`에서
backend와 문서별로 구분한다.

같은 배율에서 1쪽의 Before 1/1과 15쪽의 After 5/5, 앞뒤 본문·꼬리말을 직접 판독했다.
작은 글자의 raster 농도 차이는 남으며 raw 실루엣을 엄격 RGB 일치로 해석하지 않는다.
아래는 실제 PNG이며 다른 쪽의 점수로 확대 판독을 대신하지 않았다.

| 경로·쪽 | review | standalone overlay |
| --- | --- | --- |
| Native 1쪽 | [review](candidate/native/inline-unbound_p001_review.png) | [overlay](candidate/native/inline-unbound_p001_overlay.png) |
| fresh 1쪽 | [review](candidate/wasm/inline-unbound_p001_review.png) | [overlay](candidate/wasm/inline-unbound_p001_overlay.png) |
| Native 15쪽 | [review](candidate/native/inline-unbound_p015_review.png) | [overlay](candidate/native/inline-unbound_p015_overlay.png) |
| fresh 15쪽 | [review](candidate/wasm/inline-unbound_p015_review.png) | [overlay](candidate/wasm/inline-unbound_p015_overlay.png) |

15쪽 raw는 Native 100%, fresh 99.97071%다. 양쪽 98% 이상과 직접 판독 후
[독립 PDF 원점](independent_coordinates.json)의 ±2CSSpx를 새 이름의 CLI 회귀로 보호한다.
CLI bbox는 소수 한 자리이므로 최대 0.05px 반올림 오차도 2px 허용폭 안에서 소비한다.
원문 공백 한 칸·표시 5·두 필드·본문 소유와 누락/중복 검사도 유지한다.
1쪽의 기존 ±2px 및 앞뒤 번호·캡션 순번 관계 검사는 그대로 유지했다.

동일한 새 Rust 회귀를 Cargo가 제공하는 실제 CLI 경로로 실행해 수정 전 Native208에서
번호 0개/기대 2개로 FAIL, 보정 후 Native209에서 PASS를 확인했다.
외부 검사 harness의 지원 library는 동결 Native209이며, 수정 전 실행에서 실제로 실행한
CLI는 Native208이다. 이를 수정 전 library로 컴파일했다고 바꾸어 보고하지 않는다.
관련 기존 대조군을 포함한 현재 묶음은 28 PASS/0 FAIL이다.
전체 Rust·lint·통합 PR 준비 완료를 뜻하지 않는다.
