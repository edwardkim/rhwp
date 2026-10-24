# #8153: 원본 곡선의 점 체인과 구간 종류 보존

## 원인과 독립 근거

편람 HWP/HWPX 161쪽의 두 곡선은 같은 세 점과 두 CURVE 구간을 저장한다.
HWP 5.0의 점은 베지어 제어점 세 개씩의 배열이 아니다. 저장 구조는
[공식 문서 정오표 36절](../tech/hwp_spec_errata.md)에 원본 바이트·정상 저장본과 함께
정리했다. 공식 표 자체가 보간식을 정의한다고 주장하지 않는다.

이슈 #4676의 해결 PR #4684와 통합 `55eb2860`은 한컴이 읽을 수 없는 `hc:pt` 출력을
`hp:seg` 체인으로 고쳤다. 이후 `3741441a28`은 렌더러의 세 점 소비를 우회하기 위해
HWPX 구간 종류를 버렸다. XML 저장 형식 수정은 유지하고 타입 유실과 잘못된 소비를 함께 고친다.

독립 한컴 2024 PrintMethod 0 출력에 열린 3·4점 곡선, 동일 점의 직선,
CURVE-LINE-CURVE, LINE-CURVE-CURVE, 닫힌 곡선·혼합 구간, 두 점 퇴화를 대조했다.
진단 시제품은 8개 모두 raw 100%, 정확한 미대응 0픽셀이다. 입력 생성 절차·SHA·Print
영수증·원본 PDF·실제 Native 진단 PNG는 ignored 작업 루트의
`curve-chain-controls-v2/`에 있다. 합성 입력의 통과를 편람의 시각 검증으로 대신하지 않는다.
설치된 바이너리의 분석 기록은 `output/`에만 보존하며 Git 문서·구현 자료에 옮기지 않는다.

## 공통 계산과 소비 경로

`CurveShape.points/segment_types` → 원본 단위의 공통 곡선 변환 → 각 경로의 좌표 변환 →
기존 `PathNode` → 공통 LayerTree → 각 backend로 연결한다. 일반 경로의 도형 프레임과
글상자·채움·화살표 계약을 유지한다. 그룹 아핀 경로도 같은 변환을 소비하며 원본 점을
세 개씩 소비하거나 남은 점을 버리지 않는다. 빈 경로의 bbox는 무한값으로 만들지 않는다.

원본의 각 구간마다 끝점을 하나 보존한다. LINE은 직선이며 다음 곡선의 연결 상태를
끊는다. CURVE의 내부 접선은 이웃 원본 점 차이의 1/6로 구성하고 앞 곡선의 끝 제어점을
다음 시작점에서 반사해 연결한다. 열린 곡선 끝과 닫힌 첫 연결의 제어점은 독립 Print에서
확인한 연결 규칙으로 정한다. 화면에 맞춘 tension·문서 ID·좌표 보정 상수를 넣지 않는다.
누락·알 수 없는 타입은 기존 읽기 경로의 직선 fallback으로 처리한다. 저장기의 기존
누락 타입 CURVE fallback은 이 변경에서 바꾸지 않는다.

## 실행 순서와 완료 조건

1. HWPX parser가 명시한 LINE/CURVE를 보존하고, 타입 유실을 기대한 기존 구조 검사를
   원본 구간 보존 검사로 바꾼다. `hp:seg` 출력·점 수·`hc:pt` 미출력 보호는 유지한다.
2. 공통 converter를 일반·그룹 아핀 경로에 연결한다. source를 커밋한 뒤 jobs 8과 공용
   `target/pr-review`에서 Native를 빌드하고 producer를 고정한다.
3. 대조군 8개와 편람 양쪽 형식의 8·161·288·307쪽을 실제 출력해 독립 PDF와 비교한다.
   그룹·반사·회전의 점 소유와 연결, 정상 직선을 함께 확인한다.
4. 같은 source의 표준 최적화 fresh WASM으로 재검증한다. 시각 선행 조건을 마친 뒤
   `tests/cases/`에 실제 소비 경로의 관계 검사를 추가한다. helper만 검사하고 끝내지 않는다.
5. 전쪽 TSV·PR8148 corpus·새 base PR #8190 기능 보존과 필수 전체 검증을 계속한다.
   최종 목표인 1532개 raw 100%와 페이지별 중요 경계 ±2px는 부분 통과로 완료하지 않는다.

## 실제 공통 코드의 Native 검증

구현 source는 `dd8ea4e0544481994cf8a61a0e93f24ee5896052`다. format 검사는 exit 0,
jobs 8·공용 `target/pr-review`의 Native 빌드는 7분 41초·exit 0이다.
첫 실행은 컴파일 오류 없이 SIGTERM(143)으로 끝났다. 종료된 Cargo/rustc를 확인하고
그 로그·캐시를 보존한 뒤 실행 세션과 분리해 재실행했으며, 성공한 producer만 고정했다.

| 실제 출력 범위 | 결과 |
| --- | --- |
| 일반 점 체인의 열린 끝·직선·혼합·닫힘·두 점 8개 | raw 100%, 미대응 0픽셀 |
| 같은 8개를 그룹 항등·90도 회전 경로로 소비 | raw 100%, 미대응 0픽셀 |
| 편람 HWP/HWPX 각각 161쪽 | raw 99.98609% → 99.99934%, 미대응 21 → 1픽셀 |
| 편람 양쪽 형식의 8·288·307쪽 | raw 유지: 99.73850%·100%·100% |

위 대조군의 compare·overlay·review PNG와 편람의 Print/이전/현재 곡선 확대를 직접
판독했다. HWP와 HWPX의 현재 161쪽 해당 crop은 RGB도 같다. 남은 1픽셀은 `(282,471)`이며
100%로 반올림 처리하지 않았다. 이 결과는 시제품이 아니라 실제 parser·공통 converter·
일반/그룹 배치·LayerTree·Native renderer를 거친 출력이다.

자료는 ignored 작업 루트의 `curve-chain-dd8ea4e054/`와
`curve-chain-dd8ea4e054-focus/`에 있다. Native binary SHA-256은
`fcb9c0a2f580c8ca045dc10298fc70ddb83041f555307527adac799c59428e43`이며,
입력·PDF·PNG·직접 판독의 해시도 producer/manifest/inspection 기록에 고정했다.
그림뿐인 입력의 CLI JSON에는 글꼴 관측에 따른 `sourceSha256` 필드가 없으므로 실제
입력·producer sidecar·PNG manifest로 연결한다. 비교 디렉터리 누락과 이 스키마 가정을
고친 진단 드라이버의 실패 기록도 보존했으며 제품 컴파일 실패와 구분한다.

같은 source의 표준 release fresh WASM 빌드는 종료 코드 0이다. Cargo jobs 8,
codegen-units 16, wasm-bindgen과 `wasm-opt -O`를 거쳤으며 실제 WASM SHA-256은
`4aa0d44e4ad5e3d780b24280c4d47d171ef156c51b5a70962c638e6599ee26c3`이다.
Canvas2D의 일반·그룹 곡선 16개도 raw100·미대응 0이며 Native와 RGBA 전체가 같다.
두 review와 원본 161쪽의 Print/Native/WASM 확대를 직접 판독했다. 161쪽의 미대응
1픽셀은 두 형식·양쪽 backend 모두 남아 있으며 완료로 바꾸지 않는다.

원본 두 형식의 Native 766쪽과 최적화 fresh WASM 766쪽 검사가 각각 종료 코드 0으로
끝났다. 같은 입력·독립 PDF·정확한 DPI 래스터의 SHA를 확인했고, 모든 PNG를 실제 재생했다.

| 형식 | backend | 쪽수 | 최저 raw % | 미대응 0인 쪽수 | 미대응 픽셀 합 |
| --- | --- | ---: | ---: | ---: | ---: |
| HWPX | Native | 383 | 99.35532 | 197 | 15,201 |
| HWP | Native | 383 | 99.35539 | 168 | 14,792 |
| HWPX | fresh WASM | 383 | 99.24378 | 218 | 15,615 |
| HWP | fresh WASM | 383 | 99.24402 | 202 | 14,748 |

총 1,532개 중 미대응 0은 785개이며 잔여는 60,356픽셀이다. 이 수치는 곡선 보정 source
`dd8ea4e054`의 결과이고, 후속 앵커 표 보정 source의 결과와 섞지 않는다. Native는 이전
`919780b`와 대조해 두 형식의 161쪽만 개선됐고 나머지 764쪽은 같으며 하락은 없다.
WASM의 이전 source 대비 전쪽 변화는 이 Native 비교로 대신하지 않는다.
전쪽 TSV와 exact audit는 위 작업 루트의 `native/<문서>/whole-score/`,
`wasm-whole/<문서>/whole-score/`, `whole-{native,wasm}-exact-audit.json`에 있다.
새 회귀 등록·전체 raw100·최종 전체 회귀와 PR 제출 gate는 아직 완료하지 않았다.
새 base PR #8190의 Native CLI 기본 capabilities는 `enabled=true`, `jbig2=false`,
`jpeg2000=false`를 실제 확인했다. 개별·동시 opt-in과 자원 예산·저장·WASM 검증은 남아 있다.
