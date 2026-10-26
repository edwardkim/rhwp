# #8153: 행정업무편람 회귀 검사의 전면 재구성

## 사용자 지정 범위

2026-10-25 지시에 따라 기존 회귀 중 아래 두 실물 파일을 사용하는 검사와 공용
corpus 항목을 제거한 뒤 #8153 기준으로 새로 구성한다. 과거 기대값을 새 이름으로
복사하는 작업으로 대신하지 않는다. 제거 직전 source는 `9740178e2c`다.

- `samples/2025 행정업무운영 편람(최종).hwp`
- `samples/2025 행정업무운영 편람(최종).hwpx`

이는 사용자가 요청한 검사 재구성이며 시각 미달 이슈 이관이 아니다. 원본 두 파일과
기존 독립 한컴 Print PDF는 유지한다. 두 파일을 읽지 않는 일반 합성·사양 계약과
다른 문서의 검사는 유지한다. 공용 검사에서는 편람 항목과 그에 종속된 assertion만
제거하며 전체 모듈을 무조건 삭제하지 않는다. 제품 serializer·renderer 데이터는
검사 참조와 구분한다.

## 제거·수집 확인

직접 경로·format 경로·공유 상수·helper 호출을 추적한다. 전용 검사 외에도
parser/serializer·CLI·Studio E2E·fixture catalog·corpus baseline과 자동 수집을
확인한다. 혼합 모듈의 비대상 함수와 입력을 보존하고, 같은 파일을 동적 순회하는
기존 corpus에서도 두 문서를 제외한다. 변경한 테스트 수와 실제 파일 목록은
ignored `output/pr-review/issue8153-handbook-raw100-20261022/handbook-regression-replacement/`에
기록한다. 원본 코드를 새 회귀의 기대값으로 쓰지 않는다.

## 새 회귀의 독립 기준과 완료 조건

정식 검사는 `tests/cases/issue_8153_*.rs`에 둔다. PR 준비 전에는 전쪽 Native/fresh WASM raw100·
미대응 0픽셀 목표와 독립 PDF·쪽수·원문 대응·직접 확대 판독을 완수한다. 사용자 지시에 따른
새 검사 등록은 현재 코드의 전쪽 90% 이상과 해당 완료 페이지의 양쪽 98% 이상·직접 판독을
먼저 확인한다. 새 검사 등록이 전쪽 100% 목표의 완료를 뜻하지 않는다.
문서마다 어려웠던 핵심 경계에 관계 검사를 두고, 같은 동작을 보호하는 완료 페이지는
양쪽 raw 최솟값 100%를 우선해 독립 ±2px를 추가한다. 두 문서의 모든 페이지에서 각각
중요한 영역을 보호한다. 전쪽에 같은 형식의 assertion만 반복하지 않고, 빈 페이지는 개체
좌표 대신 페이지 소유·빈 내용·앞뒤 순서를 검사한다. 기존 핵심 페이지 검사만으로
페이지별 중요 영역 보호를 완료했다고 판정하지 않는다.

보호할 계약은 원문/구역·바탕쪽·쪽·문단·표·셀 소유, 순서·누락·중복, 저장/다시 읽기,
표 시작·분할·종료·후속 본문, 실제 글줄/안 여백, 그림/crop/테두리, 용지/출력 좌표다.
기존 검사 제거만으로 이 보호가 복원됐다고 보고하지 않는다. 새 검사 추가 후 정식
suite 수집·관련 회귀·필수 전체 Rust·lint·Skia·Studio·fresh WASM을 실제 실행한다.

단계별 실제 결과는 아래에 구분한다. 전쪽 100%·PR 준비는 아직 완료가 아니다.

## 기존 검사 제거 단계

직접 경로와 상수/helper 종속을 추적해 기존 Rust 검사 함수 59개를 제거했다.
혼합 모듈의 편람 전용 함수와 assertion만 제거하고 다른 문서의 검사·독립 합성 계약은
유지했다. 기존 #8153 묶음 그림 검사도 포함하며, 새 파일에서 독립 기대값으로 다시 만든다.
Studio 전용 검사 2개·실제 WMF browser 검사 1개와 혼합 Local Font Access의 실물 편람
부분을 제거했다. 나머지 독립 글꼴 계약은 유지했다.

공용 baseline·catalog·격리 시나리오·JSONL 및 원본에서 추출한 동일-id corpus 34행을
제거했다. 생성기의 두 corpus 정의와 관련 관측값도 함께 제거했다. ignored 경로에
생성기를 실행해 남은 17개 산출물이 저장소와 바이트 단위로 같은 것을 확인했다.
두 기존 공통 제외 helper를 통해 직접·동적 corpus와 serializer/wasm_api 단위 스윕에도
같은 범위를 적용한다. 실제 helper의 6개 대상 경로 제외·3개 비대상 유지 검사는 통과했다.

AST 문자열 검사에서 두 문서의 활성 경로는 제외 helper 두 곳에만 남는다.
남은 사양 주석·과거 PNG 설명은 실행 입력이 아니다. 원본 HWP/HWPX·PDF·golden SVG의
diff는 없다. Node 구문·Python 구문·변경 Rust 포맷과 diff 검사는 통과했다.
제거 단계에서는 정식 suite 재수집·Rust 실행과 새 #8153 검사 생성·검증이 남아 있었다.
이후 실행 결과는 아래에 구분한다.

## 새 검사 초안과 관련 검증

ignored 작업 경로에서 새 파일 두 개를 작성하고 source `0bdcdbf92d`의 실제 Native 라이브러리에
연결해 4건을 실행했다. 두 문서 모두 본문·표·셀·머리말·꼬리말·각주·미주·구역별 바탕쪽을
실제 HWPX 저장/재파싱에서 검사한다. 실제 Print 검사는 독립 383쪽의 부록·빈 쪽·판권 소속,
94·163쪽 그림의 원본/crop/테두리 소유와 독립 PDF 경계 ±2px, 149쪽 행의 실제 줄·안 여백·
후속 접수란 관계를 보호한다. 현재 4건은 통과했다. 보정 전 `5d5cc9a16b`에서는 같은 검사 중
HWPX 149쪽의 줄·안 여백 하한 검사가 실패했고 HWP는 유지됐다. 정식 등록 결과는 아래에 기록한다.

기존 혼합 #4764 실행에서 편람 corpus 제거 뒤 PDF 격리 표본이 17개가 되면서 과거의
`built > 20`이 실패했다. 임의 개수 기준 대신 독립 활성 catalog의 모든 23쪽 간격 표본과
실제 실행한 `(corpus, page_index)` 집합이 정확히 같은지 검사한다. 중복도 금지하며 기존
PDF의 2쪽·이웃 쪽 보존 assertion은 유지한다. 정상 대조 #1658·#3386과 현재 source의
Native/fresh WASM 시각 검증, 정식 suite 수집은 이어서 실행한다.
해당 보정 뒤 남은 #4764 검사 11건은 실제 source 라이브러리 연결 실행에서 모두 통과했다.

실제 suite 파생 결과는 1,704개 source이며 제거한 전용 Rust 파일 15개는 하나도 배정되지
않았다. 수집기 단위 검사가 이미 삭제한 `issue_1035_alignment`를 일반 case 예제로 사용해
실패한 것도 확인했다. 일반 suite 선택·nextest 필터·cargo-test 필터라는 검증 목적을
유지하면서 실제 남아 있는 독립 합성 case를 사용하도록 수정했고 관련 Node 25건이 통과했다.
이 변경은 기존 삭제의 후속 참조 정리다. 사용자가 재확인한 대로 기존 sample16 삭제는
유지하며 sample16 새 보정·회귀 생성·개별 검증은 이번 PR에 추가하지 않는다.

## 선행 서버 작업 대조와 잘못된 삭제 복원

사용자 지시에 따라 먼저 처리될 `ubuntu-ted`의 `review/semanticist21-20261019`
(`b3a9ef3d08`)와 `ubuntu-ted2`의 `task/8160-sample16-raw100` (`021c486d1e`)를
읽기 전용으로 확인했다. 두 서버의 기준은 동일한 `9418ed0c7`다. 두 번째 서버의
staged/unstaged 변경도 포함해 편람 관련 49·53개 파일의 기준/현재 내용과 diff를 대조했다.
다른 서버의 작업 트리·브랜치·원격 저장소는 변경하지 않았다.

두 서버에 남아 있는 검사와 현재 브랜치의 함수를 AST로 대조한 결과, 잘못 함께 삭제한
`llm_export_contract::chunks_cover_the_document_body` 1건을 확인했다. 이 함수는 편람 외에도
`samples/hwp3-sample.hwp` 논문 본문의 청크 커버리지를 보호한다. 두 서버가 유지한 논문
부분과 필요한 helper 3개를 복원했으며 기존 0.97 기준을 유지했다. 편람 부분이나 과거 px
기대값을 복원한 것이 아니다. 실제 현재 Native CLI로 이 모듈 8건을 실행해 모두 통과했다.

서버별로 제거한 기존 함수는 56개이며, 편람 입력 54개와 별도 sample16 입력 2개다.
남은 혼합 모듈의 다른 원본·독립 합성 계약은 유지한다. 두 선행 작업이 통합된 뒤에는
새 devel의 기능 보존 검증과 함께 복원한 논문 검사 및 새 #8153 검사의 수집도 다시 확인한다.
상세 source·working diff·함수 대조와 실행 로그는 ignored
`cross-host-handbook-removal-audit/`에 보존한다.

## #8153 정식 회귀 등록

전쪽 Native/fresh WASM TSV 1,532조건을 확인한 뒤 아래 두 파일을 정식 경로에 생성했다.
초안에서 실제 실행한 코드와 SHA-256이 같은 바이트를 복사했다. 두 파일의 검사 4건은
통과했으며 수정 전 라이브러리의 HWPX 행 하한 실패도 같은 최종 검사 코드로 재현했다.

- `tests/cases/issue_8153_handbook_print_boundaries.rs`: 두 형식의 독립 383쪽과 부록·빈 쪽·판권
  소속, 94·163쪽 그림/crop/테두리 소유·순서·누락·중복, 153·149쪽의 닫힌 표 프레임과
  실제 제목 줄·안 여백·안내 두 줄·후속 접수란을 검사한다.
- `tests/cases/issue_8153_handbook_source_roundtrip.rs`: 실제 HWPX 저장/재파싱에서 구역·바탕쪽의
  적용/내용, 문단·표·셀의 소속·병합·순서·본문, 머리말·꼬리말·각주·미주를 검사한다.

| 형식 | 원본 SHA-256 | 독립 Print PDF SHA-256 |
| --- | --- | --- |
| HWP | `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f` | `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593` |
| HWPX | `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6` | `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e` |

PDF는 기존 `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`와
`pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`를 사용했다. 재출력하지 않았다.
producer code source는 `0bdcdbf92d99820568f11d3efd2cc6aa5ffa56ec`이며 이후 변경은
검사·문서뿐임을 `src/`, `crates/`, `vendor/`, `Cargo.toml`, `Cargo.lock`의 diff로 확인했다.
Native와 표준 release·wasm-opt 최적화 fresh WASM은 같은 source를 실제 빌드했고 각
실행 파일·라이브러리·WASM·glue 해시를 고정했다. 공용 target은 `target/pr-review`다.

| 형식 | Native 전383쪽 최저 / raw100 쪽 | fresh WASM 전383쪽 최저 / raw100 쪽 |
| --- | ---: | ---: |
| HWP | 98.93537% / 270 | 98.93468% / 269 |
| HWPX | 99.36039% / 261 | 99.25665% / 261 |

판정은 `silhouette_raw_match_percent`, 물리 96dpi·2px 내용 실루엣, 좌상단 CSS px다.
94·153·163쪽은 두 형식·양 backend 모두 raw100이라 우선 선택했다. 149쪽은 Native
99.99721%·WASM 99.99723%이며 다른 안내 행 높이와 후속 접수란의 실제 경계도 보호한다.
독립 PDF의 pt를 96/72로 환산한 기대 경계에 ±2px를 적용하며 소유·내용·포함 관계를 유지한다.
원본 HU 하한은 제목 1000HU + 실효 안 여백 141+141HU이고, 표 전체는 153쪽 5345HU와
149쪽 5628HU다. 좌표를 현재 구현에서 복사하지 않았다.

전체 TSV는 ignored 작업 경로의 `handbook-row-frame-candidate/native/handbook-{hwp,hwpx}/whole-score/`와
`handbook-row-frame-candidate/wasm-whole/handbook-{hwp,hwpx}/whole-score/`에 있다.
`new-case-registration.json`에 원본 TSV SHA-256·전쪽 관문·검사 파일 SHA-256을 기록했다.
직접 판독은 같은 producer의 Native/fresh WASM·독립 PDF를 같은 원점과 배율로 비교한
`direct-regression-review/p094-same-origin-2x.png`, `p149-same-origin-2x.png`,
`p153-same-origin-2x.png`, `p163-same-origin-2x.png`다. 그림 외곽·설명 순서·제목·안내 두 줄·
접수란의 경계를 확인했다. 각 입력 형식의 해당 PNG가 바이트 단위로 같음도 확인했다.

새 검사 4건·남은 #4764 11건·정상 대조 #1658/#3386/#1133 7건·복원한 논문 모듈 8건이
실제 실행에서 통과했다. 수집기 Node 25건도 통과했다. 정식 파일 등록 뒤 검증 worktree에서
suite `--prepare`·`--check --base-ref upstream/devel`와 두 case의 실제 target 선택을 확인했다.
1,706 sources·7,507 static test attrs·28 suites + 20 exceptions = 48 targets다. 같은 공식
harness 생성기로 두 정식 원본을 모듈로 수집하고 실제 source 라이브러리에 연결해 4건을
다시 실행했다. 모두 통과했으며 로그는 `handbook-regression-replacement/formal-suite-run.log`,
수집·producer 구분은 `formal-suite-result.json`에 있다. 전체 Rust 회귀·lint·Skia·Studio의 최종
PR 게이트 완료를 의미하지 않는다. **전쪽 raw100은 1,061/1,532이며 471조건이 남는다.**
PR은 기존 지시대로 전쪽 100% 달성과 필수 검증을 완료한 뒤 승인 대기한다.

선행 서버 작업의 새 정상 검사나 이름을 변경한 검사를 이전 삭제 커밋으로 다시 없애지 않는다.
두 작업이 통합된 뒤 최신 devel을 기준으로 소유·등록·출력과 실제 회귀를 재검증한다.

최종 Native/fresh WASM 전쪽 비교는 각각 766조건으로 완료했다. 미대응 픽셀 증가 쪽은
없다. WASM에서 raw가 소폭 작아진 179·200·232쪽도 두 형식의 미대응 mask는 이전과
동일했고 엄격 이진 mask 오차는 줄었다. 숫자를 감추지 않고 배경 경계 개선에 따른 union
분모 변화로 구분했다. `whole-native-comparison.json`, `whole-wasm-comparison.json`,
`wasm-denominator-audit.json`과 각 원본 TSV에 기록했다.
