# HWP3 저장 왕복의 글꼴 출처·그림 앵커·제어 슬롯

Issue: #7445. 기준 `fe5725dc384689b45d60ef8ea4c83138bc2701d8`, 부모
`5a0066b4d298348bc61f3b42cc6cd911170922e3`와 이 단계의 소스 변경을 검증했다.
전체 PR 완료 기록이 아니라 저장 경로 보정의 단계별 증거다.

## 독립 근거와 원인

입력 `samples/issue_265.hwp`와 `samples/hwp3-sample.hwp`는 같은 HWP3 바이트다
(SHA-256 `645525c8cd5ec11b1742ba7cfc759f68622861916233b5e982385cdb12f0ced2`).
원본 독립 기준은 `pdf/issue_265-hwp-2020.pdf`
(`c8e4218c7b80f1fb4f7877f5613eb6fd82fa1ca2b1ca3cde6266b81586357302`, 16쪽)다.

1. **글꼴 프로그램 출처 손실:** 원본의 추가 전진을 나타내는 프로그램 6개가 HWP5 재열기에서
   사라졌다. 고정 devel의 자기 왕복은 최대 변위 0px였지만 보정 전 후보는 121.053333px였다.
   그 6개 출처만 복원하는 진단에서 0px로 돌아왔다. 이전 HWPX 보존 계약을 HWP5에도 적용했다.
   실제 방출 DocInfo의 SHA-256·언어별 글꼴 수에 결합한 보조 스트림이며 표준 글꼴 선언이나
   자간을 만들어 맞추지 않는다. strict/lenient 읽기에 같은 1MiB 상한을 두고, 잘못된 버전·해시·
   크기·그룹·손상·삭제는 출처 없음으로 처리한다. lenient 강제 실행까지 검증했다는 뜻은 아니다.
2. **문단 그림의 저장 속성 누락:** 글꼴 보존만 고친 저장본의 독립 Print는 3쪽 이후 본문과
   쪽 경계가 달랐다. 원본을 한컴에서 직접 HWP5로 저장한 독립 변환본은 문단 기준 그림에
   `flowWithText`/`allowOverlap` 두 비트를 함께 썼다. 비트를 하나씩 바꾼 Print는 미해결,
   둘을 함께 보존한 Print는 그림 뒤 본문과 다음 제목의 쪽 경계를 복원했다. HWP3 그림의
   실제 기준 위치가 문단인 경우에만 그 의미를 IR로 해석하고 같은 두 비트를 pack한다.
   렌더러의 문서 ID 예외·좌표 clamp·추가 여백으로 맞추지 않았다.
3. **인쇄할 OBJ 문자 생성:** HWPX의 1쪽에서 원본에 없는 OBJ 상자를 발견했다. 머리말·꼬리말의
   실제 8유닛 슬롯이 빠져 구조/쪽번호 슬롯 정합이 깨졌고, 떠 있는 그림의 비인쇄 표식도
   본문에 남았다. HWP3 파서에서 실제 제어 슬롯을 계상하고, 떠 있는 그림은 그림 노드와
   비인쇄 앵커가 소유하도록 정규화했다. 일반 리터럴 U+FFFC나 HWPX 파서를 일괄 변경하지 않는다.
   그림 4개의 표식만 제거하는 선행 진단도 16쪽/최대 변위 0px였다.

소비 경로는 `HWP3 기준 위치/제어 문자 → 공통 IR의 개체 속성·8유닛 offset → HWP/HWPX
writer의 실제 슬롯/속성 → 다시 연 IR → 공통 조판·paint`다. 프로그램 출처는 각 저장 포맷의
파서에서 복원하며 자연 글리프 크기와 추가 전진을 같은 조판/paint 결과가 소비한다.
저장 LineSeg를 화면 좌표로 덮어쓰는 진단은 한컴 결과를 바꾸지 않고 Native를 17쪽으로
악화시켰으므로 채택하지 않았다.

## 같은 저장본의 독립 Print

다음 파일은 Native가 실제 만든 산출물이며 공개 WASM 저장 API의 바이트도 완전히 같다.

| 입력/기준 | SHA-256 |
| --- | --- |
| `samples/issue7445/hwp3-native-font-program-roundtrip.hwp` | `8c749a2e08ed11e0ed332c9044e6888bef011acdb6c47e218f7b7f27a3242a24` |
| `pdf/hwp3-native-font-program-final-hwp5-20261017-2020.pdf` | `579648569f6b2ef7d16b4b094525407c9b5b737c5139d8d405d67ee42bd70fca` |
| `samples/issue7445/hwp3-native-font-program-roundtrip.hwpx` | `e5f97fba42b514ffed7e746b8b76a9d4c2e77b80408f8cf050bf7e72e29509d9` |
| `pdf/hwp3-native-font-program-final-clean-hwpx-20261017-2020.pdf` | `a79739f7ac4e719e039c913a483e4baecc6bfa2452e9f891129f1ae011140a01` |

공식 client `start → status → download`, engine2020, timeout900초, 한컴11.0.0.9136,
Print method0, 전처리 없음으로 각각16쪽을 얻었다. job은 HWP
`940c3154-2d83-4c1d-9aeb-5a2894587e1b`, HWPX
`3c9734df-fbe1-421b-a0f0-97322bd99d1c`다. source bytes가 달라진 각 단계의 Print를
분리했으며, 다른 저장본의 PDF를 같은 입력이라고 바꾸어 기록하지 않았다.

## 전쪽 결과와 확대 판독

canonical raw 실루엣, 96DPI, compat2022, 2px 반경, 기본 글꼴 공급을 유지했다.
캡처는 공유 lock 아래 총8개, Cargo는 jobs8로 순차 실행했다.

| 문서/산출물 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 원본 HWP3 | 16 | 98.68641% | 99.12411% |
| 실제 HWP5 저장본 | 16 | 99.03367% | 99.34905% |
| 실제 HWPX 저장본 | 16 | 99.03367% | 99.34905% |
| 빈 셀 원본 | 1 | 98.47720% | 98.58006% |
| 기장 인계 원본 | 1 | 98.92409% | 98.89442% |
| 영월 서식 원본 | 4 | 99.54902% | 99.54796% |

각 backend6조건54쪽 모두 누락·쪽수 차이·90% 미만이 없다. 원본·빈 셀·기장·영월의22쪽은
C87의 같은 원본 PNG와 각각 byte-identical이며 0.01%p 하락도 없다. 이것은 직전 보정본과의
대조이고 고정 devel 전체 비교를 대신하지 않는다. 전체48조건1745쪽의 새 head 통과도 아니다.

초기 글꼴 보존만 된 HWP5는 Native 최저6.42868%, WASM6.69264%로9쪽이 미달이었다.
두 저장 속성을 복원한 뒤 전16쪽90% 이상을 확인했고, HWPX는 1쪽97.37581%여도 OBJ가 보여
완료 처리하지 않았다. 슬롯 보정 후1쪽은 Native99.35683%이며 OBJ가 사라졌다.
1쪽 제목, 3쪽 그림·캡션·뒤 본문·쪽 경계, 12쪽 한영 혼용 제목의 폭·끝 괄호를 직접 판독했다.
글자 윤곽/AA와 한컴 Print의 작은 크기 차이는 남아 있으며 엄격 RGB100% 일치를 주장하지 않는다.

3쪽의 raw는 원본 Native99.65333/WASM99.65077%, 두 저장본
Native99.88912/WASM99.88798%다. 12쪽은 원본99.67332/99.67249%, 두 저장본
99.90660/99.90623%다. 각 문서의 전쪽90%·해당 쪽의 양쪽98%·직접 판독을 확인해,
기존 제목12쪽의 독립 ±2CSSpx와 함께 그림 뒤 본문3쪽의 첫 원문 글리프
`x=113.4400024414`, `baseline=708.1599934896`도 보호한다. 좌표 영역으로 대상을 고르지 않고
구역0/문단43/문자0의 소유와 본문 순서·누락·중복 관계를 함께 검사한다.

## 정식 검사와 실제 API

- `issue_7445_native_bold_tracking::native_bold_title_keeps_independent_origins_and_source_order`:
  프로그램 손실의 실제1FAIL→1PASS, 앵커 속성의 실제1FAIL→1PASS,
  OBJ 계약의 실제1FAIL/정상 Skia대조군1PASS→최종2PASS(exit0, 실행0.200초).
  서로 다른 실행을 별개의 테스트 수로 합산하지 않는다. 테스트 helper 인자 누락으로 실패한
  초기 컴파일은 진단 오류로 보존하고 이 FAIL→PASS 증거에서 제외했다.
- 최종 명령은 `node scripts/run-rust-test.mjs issue_7445_native_bold_tracking --
  --cargo-profile release-test --features native-skia --test-threads 8`다. manifest를 prepare한
  실제 배정 target을 사용했다. HWP/HWPX 두 포맷의 출처·원문·16쪽·좌표, 잘못된 메타데이터,
  HWPX에 불필요한 OBJ 문자 없음, 자연 글리프의 비신장 검사를 실행했다.
- Native `render-diff --via hwp|hwpx --json`: 양쪽 exit0,16/16쪽, 최대 변위0px.
- Chrome155.0.8059.84의 실제 fresh WASM에서 `exportHwp`/`exportHwpx`→재열기:
  각16쪽, 인쇄할1772개 text run의 공개 좌표 차이0. HWP3의 비인쇄 U+FFFC 제어 표식은
  HWP5 제어 노드 표현과 구별했다. 바이트 전체가 위 Native 저장본과 동일하므로 제어 개체를
  버리고 문자만 맞춘 결과가 아니다. 실제 초기 JS/WASM 응답 hash도 확인했다.
- Native binary `d519b5d33c6ea6b5dba47da67132babe6b812d9e6f060764d0305120299e5621`,
  WASM `4b1800513a0de64d83a812f47760bc17d66c695f9a9b42d83a535aa8217add84`,
  JS `9ec13c6960f52856ca2c17686c3b47f814887cece0e2185072a2400dc7dfc7a4`.
  이 단계의 빌드는 Native91.301초, WASM175.354초/exit0다. Mac locked wrapper
  web/no-opt 로컬 대체 빌드이며 Docker 최적화 통과가 아니다.

## 최종 이미지와 남은 검증

후속 PR 설명/검토 댓글에는 다음 원인별 최종 패널을 사용한다.

- OBJ: [수정 전 review](before-obj-native-review.png), [overlay](before-obj-native-overlay.png),
  [수정 후 review](after-obj-native-review.png), [overlay](after-obj-native-overlay.png).
- 그림 뒤 본문: [수정 전 review](before-anchor-native-review.png), [overlay](before-anchor-native-overlay.png),
  [Native 수정 후 review](after-anchor-native-review.png), [overlay](after-anchor-native-overlay.png),
  [WASM 수정 후 review](after-anchor-wasm-review.png), [overlay](after-anchor-wasm-overlay.png).
- 글꼴 출처: [Native review](after-font-native-review.png), [overlay](after-font-native-overlay.png),
  [WASM review](after-font-wasm-review.png), [overlay](after-font-wasm-overlay.png).

전체 회귀·범위별 lint/Skia/Studio/Gym·전체48조건·실제 소비 경로36건의 최종 head 검증은
아직 갱신해야 한다. C87의10514PASS를 이 수정본의 전체 통과로 쓰지 않는다.
기존 sample16 WMF18/35쪽의 raw 미달도 별도 blocker로 남아 있어 PR을 생성하지 않았다.
TSV·로그·JSON·실패한 시도는 커밋하지 않으며 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-fe5725dc-20261017/font-program-hwp5-c88/`,
`font-program-hwp5-c89/`, `font-program-hwp5-c90/`에 구분 보존한다.
