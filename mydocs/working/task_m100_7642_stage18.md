# 이슈 #7642 — HFT 글꼴의 구두점 언어 슬롯

이전 단계: [아래첨자와 CASES 논리 기준선](task_m100_7642_stage17.md), source `8b0414b28`.

## 독립 근거와 원인

원본 `samples/exam_math_8.hwp`의 CharShape11은 한글 `신명 중명조`, 영문 `한양신명조`다.
같은 원본 MCP 2020 Print의 `(가)` 두 괄호는 영문 글꼴 T2, 한글은 T3로 구분된다.
Native는 `ascii_punct_latin_slot=false`로 두 괄호를 한글 HFT 반각 슬롯에서 측정·표시한다.
앞 괄호의 폭 차이가 뒤의 한글/공백/수식 원점에 누적된다. 좌표는 원인 추적용이며 고정
좌표나 픽셀 허용치로 화면을 맞추지 않는다.

글꼴 선택 경로는 `DocInfo fontRef/alt_type → resolve_single_char_style의 슬롯 정책 →
composer::char_lang_slot 및 줄 나눔/폭 측정 → ComposedTextRun 언어 슬롯 → 각 painter`다.
변경한 슬롯 정책을 측정과 paint가 같이 소비하는지 실제 같은 문단의 SVG로 확인한다.

한글 HFT의 표시용 치환과 영문에 선언된 HFT 프로그램을 구분한다. 한글 전용 HFT가
치환된 경우에도 별도의 영문 HFT 프로그램과 등록된 ASCII 폭 표가 있으면 구두점은 그
영문 슬롯을 사용한다. 기존 LegacyLatin의 라틴 전용 치환 경로를 유지하며, HCI Hollyhock처럼
한글 글꼴로 치환된 영문 슬롯·일반 TTF·같은 face·미확인 폭 표는 이 추가 계약의 대상이 아니다.
문서명·문단 ID·글자 크기·특정 좌표의 조합을 분기 조건으로 쓰지 않는다.

## 구현과 검증

- 슬롯 정책을 작은 모듈에 둔다. 기존 style_resolver 1901줄에서 해소 타입과 기존 테스트를
  별도 모듈로 이동해 수정 파일을 1000줄 이하로 유지한다. 기존 테스트 함수·내용·모듈 경로를
  보존하며 미달 입력의 테스트 기대값을 바꾸지 않는다.
- Native 원본과 eq002 HWP/HWPX·국어1p 대조를 새 source로 출력한다. 문단 `(가)/(나)`의
  앞뒤 내용·괄호·수식 원점과 확대 이미지를 직접 대조하고 전쪽 TSV를 확인한다.
- Native 90% 이상 및 직접 판독 뒤 fresh WASM/Studio 동기화와 같은 입력의 대조를 수행한다.
  그 결과를 확인한 뒤 필요한 관계형 회귀를 추가/변경한다. 기존 source 단위 검사 이동은
  새 렌더링 기대값 추가가 아니다. 전체 실패 90개는 이번 브랜치에서 계속 보정한다.
- 새 source에 영향을 받는 다른 원본도 최종 52개 검증에 포함한다. 이전 frozen worker의
  결과를 최종 source 통과로 대신하지 않는다.

원본/Print 해시·슬롯 진단·명령·TSV·중간 JSON·로그·nextrun은 ignored output에 둔다.
Windows의 H2MJSM.TTF는 존재하지만 로컬 복사는 아직 실패했다. 폰트 공급 조건은 바꾸지
않았으며 이 파일 확보 여부로 잘못된 슬롯 측정의 보정을 중단하지 않는다.

영문 폭 표 조회는 기존 메트릭 별칭 계약을 따른다. `한양신명조`는 실측 ASCII가
`HanyangSinMyeongJo`에 등록되어 있어 별칭 존재를 거부하지 않는다. 영문 표시 face를
다른 face로 치환한 경우나 ASCII 폭 표가 없는 경우에는 추가 슬롯 정책을 적용하지 않는다.

## 현재 작업의 사용자 지정 범위

현재 브랜치의 전체 회귀 실패 90개는 고정 base의 미달 여부와 관계없이 이 브랜치에서
보정한다. 기존 일반 지침의 별도 이슈 이관을 이번 실패 처리에 적용하지 않는다.
원본·독립 Print PDF의 시각 검증과 확대 판독을 먼저 충족한 뒤 필요한 검사만 관계형으로
교정하며, 실패 검사를 삭제하거나 허용치를 완화해 완료로 보고하지 않는다.

## Native 보정 결과

`cargo build --locked -p rhwp --profile release-test --target-dir
/home/tsjang/rhwp/target/pr-review --bin rhwp`는 6분23초, exit0이다. source 파일 해시를
고정한 binary SHA-256은 `6ef43159817a6dc85c20d755ca1bcc680debc950865d06b403136732d3f7706a`다.
기존 테스트26개의 토큰·모듈 경로가 보존됐으며 `cargo fmt --all -- --check`,
`git diff --check`, 고정 base의 source-unit-tier 검사4198개는 통과했다.

같은 원본·동일 한컴 Print PDF의 Native 전쪽 비교 결과:

| 원본 | 이전 단계 | 이번 Native | 누락 쪽 |
| --- | ---: | ---: | ---: |
| `exam_math_8.hwp` | 89.16346% | **94.16183%** | 0 |
| `eq-002.hwp` | 94.81146% | 94.81146% | 0 |
| `hwpx/eq-002.hwpx` | 94.81146% | 94.81146% | 0 |
| `exam-kor-1p.hwp` | 98.89162% | 98.92238% | 0 |

원본 math8 입력 SHA-256은 `f29e0d92c45c51306a482c9304e70820a18f55ec8c44b6cdffc26db0e54645ff`,
독립 Print PDF SHA-256은 `a4aab7757207e2cc4838afd739641d5ea1546f3b0a57944a5dbca3e7bf3e2d6f`다.
Math8의 해소 스타일에서 `ascii_punct_latin_slot=true`를 확인했다. source 검사·빌드·원본/PDF
해시와 명령, 전쪽 TSV는 ignored `stage15-current-validation/punctuation-slot-native*`에 고정했다.

`representative-native/exam-math-8-punctuation-slot/review/review_001.png`와
`math-region-3x.png`를 생성하고 사용자에게 직접 표시했다. `(가)/(나)`의 괄호 폭과 뒤 본문·
수식 원점은 개선됐으며 CASES 열 맞춤·아래첨자는 앞 단계와 같은 보정 결과를 소비한다.
조건 행·글꼴 모양의 잔여 차이는 90%로 면제하지 않고 확대 분석을 이어간다. 이번 결과는
Native 네 원본의 측정 결과이며 fresh WASM·52개 전수·전체90개 회귀 실패의 해결을 뜻하지 않는다.
렌더링 회귀 기대값·golden·허용치는 이번 단계에서 변경하지 않았다.
