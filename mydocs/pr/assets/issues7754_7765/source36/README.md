# 소스 36의 확정 증거와 진행 중인 검증

생산 소스는 `436f8629b7f2b9aee520962297b79cdf15c17743`, 새 기준은
`1a27faacff48ea7329e76ce8eca15aa9ca8aa877`이다. 정식 검사 등록은
`c48c2b895d31ba0fd4969c837bfc7a6d8a2c65ac`이며 제품 소스는 생산 소스와 같다.

## 확정한 결과

| 범위 | 결과 | 근거 |
| --- | --- | --- |
| 평문 원본 10개 Native | 전쪽 583쪽, 모두 raw 98% 이상 | `issues/native/qualified-originals.json`, 각 원본의 TSV·실행 manifest |
| 새 기준과 후보 Native | 각 583쪽, 페이지별 표시값 하락 0쪽 | `issues/native/comparison.json`, `condition-comparison.json` 및 양쪽 전체 TSV |
| 평문 원본 10개 fresh WASM | 기준·후보 각 583쪽, 후보 전쪽 98% 이상·페이지별 하락 0쪽 | `issues/wasm/qualified-originals.json`, `comparison.json` 및 양쪽 전쪽 TSV |
| 실제 암호 원본 | Native 24쪽 최저 99.50%, WASM 24쪽 최저 99.48% | `encrypted/results.json`, 양쪽 전체 TSV |
| 암호 원본과 평문 대응 | 경로별 PNG 24/24 SHA 일치 | `encrypted/results.json` |
| Studio 유입 기능 | 기준·후보 각 7개 여정 통과 | `incoming/studio/validation.json` |
| 편집 기능의 저장 원본 Native | 기준·후보 각 33개 조건·50쪽, 전쪽 PNG SHA 일치·하락 0쪽 | `incoming/saved-native/validation.json`, 양쪽 전체 TSV |
| Native 실제 중첩 표 편집 | 원본 2종 × 30/70줄 × 3상태 × 2소스, 대응 PNG 전체 동일·최저 98.07% | `incoming/live-native/validation.json`, 전체 TSV·소유·순서 검사 |
| WASM CTRL_DATA | 기준·후보 각 4개 실제 입력 × 2회 저장·재열기, 8/8 소유·payload 보존 | `incoming/ctrl-data-wasm/*/physical-record-validation.json` |

전쪽 수는 측정 요청 수만으로 판단하지 않았다. 평문 Native 각 실행의
`document_page_counts.rhwp`와 `pdf`가 독립 PDF의 전체 쪽수와 같고,
페이지 TSV가 1쪽부터 끝 쪽까지 중복·누락 없이 있는지도 확인했다.

시장구조조사 315쪽의 최저값은 98.40%다. 안내문에서 같은 옛한글 동작을
보호하는 최고 완료 쪽은 2쪽이며 Native/fresh WASM 모두 99.99%다.
독립 PDF 좌표와 확대 판독 근거는 기존 `independent-pdf-coordinates.json`,
`direct-review/` 및 앞 단계 소스 35의 직접 판독 자료를 함께 사용한다.
원본 TTF → 검증된 대응 TTC → CFF 순서는 유지했다. 원본 HFT 폭과 표시 CFF의
프로그램 메트릭을 구분한 수정이며, 문서별 좌표 보정은 추가하지 않았다.

새 기준의 시장구조조사 최저값은 26.80%였으며 후보는 98.40%다.
기준의 90% 미만 종료 상태는 수정 전 원인 증거로 보존했다. 후보의 실패가 아니다.
각 쪽을 같은 독립 PDF·DPI·raw 방식으로 비교했고, 글꼴 공급·Native 대체 계획·
PDF 래스터 조건·레이아웃 호환 모드도 원본별로 같음을 확인했다.
후속 문서 커밋과 제품 생산·정식 회귀 코드의 동일성은
`gates/source-equivalence-checkpoint.json`의 Git 비교와 선택 파일 트리 해시로 잇는다.

## 유입 기능의 실행 근거와 한계

Studio는 기준·후보 사이에 바뀌지 않은 실제 소스에서 각각 고정한 최적화 WASM을
읽었다. 공용 artifact observer가 실제 네트워크 응답 SHA를 검사한다.
메모 저장·재열기, 본문 붙여넣기, 글꼴 입력 재사용, Canvas2D/CanvasKit 공급자
변경, 원본 분할 CFF, 문서 전환 시 눈금자를 실행했다.

초기 중복 CDP 본문 읽기 실패와 글꼴 선택 대기, sparse worktree 준비 오류는
`incoming/studio/initial-state.json`에 보존한다. 보정한 probe는 공용 observer를
사용하고 실제 '대체 글꼴로 보기' 버튼을 선택한다. 최종 눈금자 영수증에는
종료 코드 0과 세로 795px → 가로 1123px의 실제 상태가 있다.
#6821의 기존 부분 수용 범위인 가로 스크롤 25% 목표·실물 관성은 완료로 바꾸지 않는다.
원본 CFF 공급자 검사는 실제 글리프·캐시 기능 증거이며 독립 PDF 전쪽 측정을 대신하지 않는다.

CTRL_DATA는 렌더 결과나 직렬화기의 자기 보고에 의존하지 않는다. 보존한 별도
Python reader가 OLE/DEFLATE 안의 HWP 레코드 크기·레벨·tag를 읽고
CTRL_HEADER/SHAPE_COMPONENT 소유 스택과 각 payload의 길이·SHA를 비교했다.
새로 내보낸 HWP는 중간 산출물로만 두며 이 디렉터리에는 검사 코드와 결과를 보존한다.

유입 편집 기능의 저장 원본 33개 조건은 같은 글꼴 공급·래스터 설정으로 양쪽 각
50쪽을 실행했다. 후보 최저값은 98.07%이며 모든 대응 PNG가 바이트 단위로 같다.
이는 저장된 실제 입력을 다시 여는 보호 검사다. 공개 편집 API로 직접 내용을 추가한
중첩 표의 live·HWP 재열기·HWPX 재열기는 별도로 실행한다.

## 아직 완료하지 않은 항목

유입 기능 저장 원본의 WASM 비교와 실제 중첩 표 WASM 편집·재열기,
최소 글꼴 환경의 정식 전체 Rust 검증, 편람 gate와 정본 보호 비교는 실행 중이다.
Native 583쪽 완료와 위 기능 검사를 전체 검증의 완료로 해석하지 않는다.
보호 비교의 기존 글꼴·환경 재현 문제와 표시값 하락은 별도로 해결해야 한다.

공유 target은 보존했다. 사용 종료된 이 작업의 초기 실행 파일 28개만 삭제해
3,981,930,744바이트를 확보한 경로·해시·활성 참조 검사 근거는
`gates/source36-obsolete-binary-cleanup.json`에 있다. 현재 생산물과 원본·PDF·TSV·PNG는 보존했다.


## 정식 검사 중간 확정과 환경 resolver 보정

`gates/native-checkpoint/results.json`은 전체 검증 완료 이전의 확정 접두 실행이다.
최소 글꼴 Native unfiltered lib 4,033건 통과·12건 ignored, 기존 Native CI 논리
검사 21개와 관련 집중 검사, fmt·manifest·unit-tier·양쪽 Clippy의 통과를 담았다.
workspace build의 디스크 공간 부족은 보존하고 재실행하며 전체 nextest는 남아 있다.

`protected/environment-repair/`는 작업 전용 resolver가 chemical·decree의
정본 환경 4개를 누락한 오류의 보정 근거다. 고정된 조건 보고서와 TSV 해시를 대조해
46개 스키마를 판독했고 정확한 환경 파일 45개를 찾았다. outline Native의 정확한
환경 파일 1개는 계속 찾는다. 정본 환경을 적용한 decree 16쪽의 raw는
Native·fresh WASM 모두 100.00%다. 이전 잘못된 stock 환경의 52.97%는 보존한
진단 결과이며 제품 회귀나 글꼴 우선순위 결함으로 판정하지 않는다.
전체 보호 비교·정본 대비 하락 해소는 별도로 남아 있다.


Native 실제 편집은 기준·후보에서 각각 원본 HWP와 실제 한컴 HWPX에 30줄·70줄을
추가한 뒤 live·실제 HWP 저장/재열기·실제 HWPX 저장/재열기를 실행했다.
24개 상태의 독립 Print 전쪽 점수 최저는 98.07%이고 대응 12쌍의 모든 PNG가 같다.
렌더 트리의 실제 `줄 1`부터 마지막 줄까지 내용·순서·누락·중복과 독립 Print의
21줄·58줄 경계 쪽 소속도 확인했다. 별도 rustc probe의 serde 의존성 불일치는
연결 시도 기록으로 남기고 맞는 의존성에서 재실행했다. WASM의 실제 저장/재열기는
별도로 실행하며 이 Native 결과로 대신하지 않는다.


## 보호 글꼴 문맥의 실제 원인 분리

`protected/original-font-context/causal-diagnosis.json`은 sample16 HWP 2022의 51쪽을
다룬다. 원래 배포 백업에서 기존 보호 기록과 SHA가 같은 OTF 247개를 확보했다.
새 SPSMJ의 U+25CB 매핑을 정확한 옛 정본 프로그램으로 별도 프로세스에서만 대조하자
Native raw가 99.44%에서 100.00%로 바뀌었다. 두 원 기호 이외의 본문·표 픽셀은
변하지 않았다. 제품·전역 글꼴·폰트 우선순위는 그대로다. 폰트 파일 자체는 포함하지 않는다.
이 결과는 원인 확인이며 fresh WASM과 전체 정본 보호 gate의 통과는 아직 아니다.


## 소스 36의 최종 판정: 보정 필요

`gates/full-rust-source36/`의 전체 nextest 결과는 13,968 통과·2 실패다.
복원한 두 원본의 독립 1-up 참조와 기존 모아 찍기 제외 규약을 보정해야 하며,
이 실행을 전체 Rust 통과로 기록하지 않는다.

`protected/alias-regression/`은 동일 정본 글꼴 조건에서 기준 100.00%·후보 99.98%인
실제 별칭 회귀를 보인다. 검증되지 않은 HFT 논리 이름이 빈 glyph를 가진 다른
프로그램을 선택해 두 점을 잃었다. 선택한 실제 family만 조회하는 Native 진단은
기준 PNG를 정확히 복원하고 안내문 2쪽의 99.99%·기존 PNG 동일성도 유지했다.
stock WASM 안내문도 99.99%, 별도 Chrome 155 private context 중첩 표는 100.00%다.
후자는 진단으로 구분하며 최종 코드·stock 보호 검증은 다음 보정 head에서 수행한다.
