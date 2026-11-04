# #8174 v2-S54 — 정규화의 요청 예산 전파

2026-10-27, 기준 `81aa056d3e`(제품 S53 `4024cf68fc`). 기존 통합 진행 승인에 따라
T04의 render normalization·파생 문단/구성 줄·저장 프레임/index 생성을 같은 요청에 연결한다.
성능 조사에서 찾은 Canvas 메트릭 증분화는 이번 작업에 추가하지 않는다.

수정 전 경로는 `PdfImportSession::open → from_parsed_document_with_storage →
paginate_pass_with_storage → compute_render_normalized`다. 마지막 일반 호출 뒤에서
section 벡터·파생 문단·줄과 overlay의 path/map/frame backing이 생성된다. 이를 같은 Storage로
연결하되 기존 원본/파생본 구분·cache reuse·정규화 조건·원점/높이 계산은 보존한다.

옵션을 반환하는 저장 정보 판정의 정상 `None`과 자원 오류를 구분한다. 요청 오류는 즉시
상위로 반환하고 다른 정규화 또는 원본 fallback으로 성공시키지 않는다. 일반 HWP/HWPX의
기존 API는 같은 구현의 Unbounded wrapper를 유지한다.

변경 전 source와 Native library는 ignored `output/pr-review/issue8174/v2-s54-final-20261027/`에
고정했다. 아래 Native·최적화 fresh WASM·실제 브라우저 검증을 완료했다.
T04 전체나 PR 준비 완료를 아직 선언하지 않는다.

## 같은 생산·소비 경로에 연결한 것

| 경계 | 실제 보정과 유지한 계약 |
| --- | --- |
| Core 정규화 | section revision/이전 상태/결과 벡터·대상 인덱스·파생 문단·구성 줄을 같은 Storage로 생성한다. revision hit의 Arc 재사용과 원본 문서 보존은 유지한다. |
| 수식·글상자·저장 꼬리 | 기존 판정·좌표 산술·쪽/단 되감김과 lazy fallback 순서를 유지하며 문단/표 복사·행 높이/map·뒤 원점 projection을 예약한다. |
| 정규화 overlay | 경로의 entries·두 인덱스/map·reflow set·새 projection/overlay Arc와 저장 프레임 backing을 예약한다. 이전 projection의 공유와 pointer/path 판정은 유지한다. |
| 저장 물리 프레임 | 부모의 닫힌 행·자식 프레임을 확인하는 기존 proof가 같은 요청으로 line/row/unit/frame 벡터를 만든다. 허용 조건·시작/끝 컷·빈 밴드와 패딩 계산은 바꾸지 않는다. |
| 공통 줄/텍스트 폭 | 기존 수식 줄 재발행·인라인 글상자 helper가 같은 composition/reflow·style 복사와 scalar/cluster/mask backing을 소비한다. 원래 탭 해석·글자 폭·마지막 반올림과 줄 원점을 유지한다. |
| 오류 전파 | optional proof의 정상 `None`과 reservation 오류를 분리한다. `optional(result, failure)?`가 즉시 proof를 끝내고 원 오류를 바깥 `Result`로 전달한다. 실패를 음성 판정이나 일반 fallback 성공으로 바꾸지 않는다. |

최종 feature-off 검사에서 재사용한 `try_reflow_line_segs`의 기존 PDF 전용 cfg를 발견했다.
이를 같은 공통 helper의 내부 가시성으로 정리했다. 기본 feature의 계산은 같지만 source가
바뀌었으므로 최초 검증은 `v2-s54-20261027/`에 보존하고, 최종 source를
`v2-s54-final-20261027/production-freeze.json`으로 다시 고정해 검증했다.

## Native 결과

| 항목 | 실제 결과 |
| --- | --- |
| 새 공개 예산 검사 | 같은 test source·Native64 cap에서 **S53 2 FAIL / 후보 2 PASS**. 실패 유지·같은 요청 재사용 거절·새 기본 요청의 같은 쪽수/HWPX bytes 복구 |
| 공개 요청·저장·wire | **35 + XML 2 = 37 PASS** |
| Core·측정·typeset·pagination·정규화 | **629 PASS / 기존 8 ignored**. S53의 624개에 기존 정규화 5개도 실행 |
| composer·float·table/paragraph | **244 PASS** |
| 텍스트 폭·글상자 측정 | 기존 **38 PASS** |
| serializer/adapter·Allocation | **662 + 9 PASS**. 다른 그룹과 중복이 있어 고유 합계로 세지 않음 |
| 기존 frame integration | TAC/리더/번호/condense **17 PASS** |
| 기존 정규화 integration | 실제 중첩 표의 앞/뒤 컷·후속 문단·왼쪽 앵커 **3 PASS**. 검사·원본·기대값은 변경하지 않음 |
| header/ZIP | 실제 11개 문서의 header·정상 ZIP·기존 양식 거절 등 **22개 파일 bytes 보존** |
| PDF 저장/재열기 | 17개·49쪽, **34개 저장/98쪽 재열기·상세 3개·선택 4개**, 저장 파일 **37개가 S53과 bytes 동일** |
| CLI 실패/복구 | 출력/진단/잔여 예산 × HWP/HWPX **6 PASS**, 기존 출력 보존과 정상 후속 요청 |
| lint/구성 | Native/WASM Clippy `-D warnings`·feature-off·fmt·절편 unit 정책·suite 계약을 포함한 **12개 실행 PASS** |

새 cap은 S53에서 실제 측정한 초기 decoded + 64 bytes다. 새 counter의 정확한 기대값을
고정한 테스트가 아니라 외부에서 전달하는 유한한 입력 정책이다.

| 입력 | S53 초기 decoded | 후보 초기 decoded | 새 cap |
| --- | ---: | ---: | ---: |
| CFF | 1,626,516 | 1,626,908 | 1,626,580 |
| cell-nested | 1,824,434 | 1,824,954 | 1,824,498 |
| multiline-cell 대조 | 7,607,095 | 7,607,487 | 해당 없음 |
| paragraph-origins 대조 | 7,640,947 | 7,641,339 | 해당 없음 |

수치는 Native64의 **논리적 payload 작업량**이며 실제 heap/RSS·시간 증가율이 아니다.
일반 공개 호출은 같은 Unbounded 구현을 사용한다. 새로운 레이아웃 fixture/golden이나
좌표·쪽수 핀을 만들거나 기존 기대값을 완화하지 않았다.

## 최적화 WASM·전쪽 시각·실제 브라우저

| 항목 | 최종 source의 실제 결과 |
| --- | --- |
| 최적화 제품 | root에서 locked wrapper·codegen-units 16·`wasm-opt` 완료. root pkg와 Studio public의 실제 JS/WASM bytes 일치 |
| Worker | **249 assertions / 일반 11 + 메모리 실패 3 = 14 Workers**. 예산·출력 거부·취소·시간 초과·실제 메모리 실패와 새 요청 복구 |
| Studio | 실제 새 WASM 수신 hash 확인. 기존 문서·draft·recents 보존, 오류/취소/복구와 사용자 PDF 15쪽 가져오기·표시·HWP/HWPX 저장 통과 |
| Native 전쪽 | **17개·49쪽**, 최저 raw **98.34664%** |
| fresh WASM 전쪽 | **17개·49쪽**, 최저 raw **98.79246%** |
| 이전 출력 대조 | 각 runtime의 **실제 PNG 49개 전부가 S53과 bytes 동일** |

90% 미만·측정 불가·누락·쪽수 불일치는 없다. Native/fresh WASM이 서로 byte 동일하다는
주장이나 원본 PDF와 픽셀 동일하다는 판정과 구분한다. 각 runtime의 CFF 1쪽·Type3
최저 3쪽/대표 11쪽·중첩 셀·여러 줄 셀의 새 review 5개와 Type3 11쪽 standalone overlay를
직접 판독했다. 기호·ligature/Greek·간격, 표/셀 소속·앞뒤 내용·줄 구성, 그림/캡션·후속 행의
경계를 확인했고 새 누락·겹침은 관측하지 않았다. 기존의 문자 획·이미지 세선 래스터 차이는 남는다.

원시 TSV는 `visual/candidate/{native,wasm}/<문서>/silhouette.tsv`, 직접 판독한 새 PNG는
`full-page-reviews/`에 있다. `visual-evidence-audit.json`, `direct-review-{native,wasm}.json`,
`{native,wasm}-page-byte-equivalence.json`과 `final-evidence-check.log`에 실제 쪽수·최저값·
source/product/test hash·PNG hash와 실행 판정을 연결했다. 자동 TSV의 `not_evaluated`와
Worker의 고정 `visualJudgment` 문구를 직접 시각 승인으로 사용하지 않았다.

최적화 WASM은 **25,211,976 bytes**로 S53의 25,208,798 bytes보다 **3,178 bytes /
0.01261%** 크다. 이는 제품 크기이며 시간/RSS의 증가율이 아니다. 성능 최적화나 추가 벤치마크를
이번 절편에 넣지 않았고 최종 T08 source의 성능 비교는 남는다.

## 남은 범위와 판정

정규화의 section/문단/구성 줄·overlay·저장 프레임 backing과 연결한 측정 owner는 이번에
보강했다. 아직 남은 T04는 일부 일반 helper·kerning/shaping/frame metric 하위 경로,
TypesetEngine/Paginator 내부 state·body page-field 재분할·표 저장의 cell-unit/index cache와
content-loss 수집이다. 실제 bounded 제품 호출의 도달 경로를 기준으로 같은 통합 범위에서
계속 처리하며 일반 HWP 전체 엔진 재작성으로 넓히지 않는다.

T05–T09의 WASM/Studio 잔여·JPEG/의존성·최종 포장/소비·전체 회귀/성능·source unit 이관과
최신 base/기능 보존·PR 준비도 유지한다. 이번 절편 완료를 T04 전체나 PR 준비 완료로
표시하지 않는다. 논리 계상과 allocator/RSS를 구분하고 `parserAllocationBoundsProven=false`와
host 책임·기존 한도를 유지한다. 원격 push·PR 생성·registry 게시·토큰 열람은 하지 않았다.
