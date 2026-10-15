# #7899 — Docker WASM·한컴 저장·시각 검증 결과

계획: [수행](../plans/task_m100_7899.md), [구현](../plans/task_m100_7899_impl.md). 구현·집중 검사: [단계 1](task_m100_7899_stage1.md). 검증 code head는 `b3741824f`이다. 저장 보존 계약은 충족했고, **PR 시각 gate는 미충족**이다.

## 같은 코드의 실제 저장 검증

| 실행 | 결과와 증거 |
| --- | --- |
| Native 저장 | 공개/출처 고정 HWP 10개에서 미편집·A 삽입·재열기/B 삽입 저장 성공. 다른 구역은 그대로이며 원본 tail의 tag/level/payload/순서를 유지한다. CLI 표준 빌드 hash `d155b0bff008e5c9f038d1ccbf05786d4332233bbf71a5b928b22dd15bda1e32`. |
| Docker fresh WASM | canonical `scripts/wasm-pack-locked.sh --target web --out-dir pkg`, Docker Rust 1.93.0 / wasm-pack 0.15.0. 공용 host `target/pr-review`를 `/build-target`에 bind하고 UID 1002로 빌드했다. 별도 target·cache 초기화 없이 기존 Docker cache를 재사용했다. 8m27s 성공. WASM hash `f77a7f19157e91cc637b7c713f53ebc22fd9bd005c31a15c88d08bd5f8e14662`, glue hash `f377c6be818c64ffd20752e29271000ff5053ea8c7c984f65abb8fe74f17eddd`. root pkg / Studio public이 각각 동일하다. |
| WASM save API | Node v24.15.0의 실제 HwpDocument로 같은 10개 문서의 미편집·편집·재편집·재열기 성공. 모든 모드에서 **모든 구역 바이트가 Native와 동일**하며, tail은 원본과 동일하다. |
| 실제 CDP Studio | 신규 검증 서버 `http://localhost:7799/`, Chrome 155.0.8059.40. math_8 1쪽 및 메모 문서 40쪽을 실제 로드/렌더 후 Q 삽입·브리지 export 실행. 수신 WASM hash가 Docker 산출물과 동일하다. 두 저장본과 메모의 reported-save 모두 원본 tail을 유지한다. 실제 명시적 저장 API의 content-loss report는 count 0이다. 개발 helper 호출의 저장 계약 검사이며 마우스 입력/다운로드 대화상자 검증으로 보고하지 않는다. |

## 한컴 독립 열기·재저장

모든 요청은 원본 바이너리를 바꾸는 preprocessing 없이 managed direct backend를 사용했다. API success만 보지 않고 내려받은 HWP를 독립 해독했다. 메모 번호·본문·소유 구역이 원본 목록과 같은지 대조했고, math 저장본의 A는 독립 Print 텍스트에도 존재한다.

| 보정 입력 | 실제 엔진·job | 결과 |
| --- | --- | --- |
| math_8 / A 삽입 | 2020 11.0.0.9136, `7017c34e-b7e3-42fa-8194-502da43c8acc` | 실제 one-up Print PDF 1쪽. A와 원본 바탕쪽의 1/20 표시 확인. |
| 단일 메모 / A 삽입 | 2020, `29f1ac6d-df7f-41e4-848a-9e3f2b0b21be` | 열기·재저장 성공. 번호 1, 본문 ‘수정’ 및 owning section 유지. |
| aift / 마지막 구역 A 삽입 | 2020, `baec53a3-8ede-4d58-8e58-fe27abb7ba7a` | 열기·재저장 성공. 번호 1·2, 각 본문 및 마지막 구역 소유 유지. |
| math_8 / A 삽입 | 2024 13.0.0.3901, `f54a722e-6823-4f30-b980-bd96bf3e2052` | 신고한 2024에서도 열기·재저장 성공. |
| 단일 메모 / A 삽입 | 2024, `b060a5b0-0fbb-4361-ae44-9b2f0aab8ba3` | 열기·재저장 성공. 번호 1과 원본 본문·구역 유지. |

비교는 `candidate-oracle/memo-content-comparison.json`에 있다. 구역 tail만 보존한 중간 빌드에서 메모가 사라진 결과도 보존한다. 그 중간 결과를 최종 통과로 재사용하지 않았다. 2024 전 문서·595/16개 전수 확인을 주장하지 않는다.

## 시각 gate와 범위 판정

Native는 native-skia feature 빌드까지 확인했다. 같은 A 편집 저장본의 독립 한컴 2020 Print와 Native/fresh WASM의 **전체 1쪽 TSV**를 각각 산출했다. 실제 `/mnt/c/Windows/Fonts`의 H2MJSM/HYHWPEQ 등을 확인해 전체 임베딩하고 font inventory hash를 기록했다. 일반 모드의 compare/overlay/review도 생성해 수식·앞뒤 텍스트·표 외곽·바탕쪽을 직접 판독했다.

| 문서 | PDF / Native / WASM 쪽수 | Native 최저 | fresh WASM 최저 | 판정 |
| --- | --- | --- | --- | --- |
| math_8 A 편집본 | 1 / 1 / 1 | 55.19024% | 55.19024% | re_review_required — 수식 누락·본문/표 배치 차이 |

고정 base와 실행 코드가 같은 기존 CLI(source `5eb992843`) 및 현재 보정 Native의 동일 입력·font policy SVG hash는 모두 `eaeb01af5e5e11edb109ef773baec8a85db488972ffe421723576eb5dcc187d1`이다. `5eb992843 → b3eaf7b10`의 parser/model/serializer/document_core/CLI는 변하지 않았고 renderer 두 파일의 diff도 **주석 번역뿐**임을 별도로 대조했다. 따라서 관측한 수식·표 차이는 이번 저장 보정의 새 회귀로 분류하지 않는다. 기존 차이라는 이유로 90% 계약을 면제하지 않는다.

다른 9개 입력의 전쪽 독립 Print/TSV는 미실행이며 저장 바이트 검사 통과로 대신하지 않는다. 새 렌더링 fixture/golden, 기존 검사 이관·완화는 하지 않았다. 계획의 저장 계약 구현과 집중 실행은 완료했지만 이 시각 차이를 수정할 범위 또는 메인터너의 저장 계약 한정 예외 판단이 필요하다. 긴 전체 Rust/Clippy 검증 및 원격 push·댓글·PR 생성·close는 수행하지 않았다.

## 로컬 증거와 재확인 경로

ignored `/home/edward/mygithub/rhwp/output/pr-review/issue7899-20261014/`에 입력/저장본·MCP 원시 결과·독립 해독·TSV·대표 PNG를 보존한다.

- `candidate-final-native/record-comparison.json`, `candidate-final-wasm/save-results.json`, `candidate-final-wasm/record-comparison.json`.
- `docker-wasm-build.log`, `browser-probe.json`, `browser-memo-probe.json`, `browser-record-comparison.json`, `browser-reported-save.json`.
- `candidate-oracle/*/{start,status,download}.json`, `candidate-oracle/memo-content-comparison.json`.
- `visual-final-native-skia/math-edited/silhouette.tsv`, `visual-final-wasm/math-edited/silhouette.tsv`.
- `review-final-wasm/math-edited/review/review_001.png`, `overlay/overlay_001.png`, `compare/compare_001.png`.
- `base-render-comparison.json`, `visual-review.html`(PNG 포함, 폴더 이동에도 독립적으로 열 수 있음).

Studio는 위 URL의 math·memo 검증 탭에 열려 있다. Docker 동기화가 만든 tracked public glue 변경은 로컬 확인 산출물이며 이 문서 commit·PR source에 포함하지 않는다. 승인 이후 단계는 해당 범위로 계획을 갱신한 뒤 진행한다.
