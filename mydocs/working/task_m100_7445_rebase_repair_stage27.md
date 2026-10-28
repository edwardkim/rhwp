---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_7445_rebase_repair_stage27.md
last_verified: 2026-10-21
---

# #7445 보정27 — 셀 문단 앞 간격과 중첩 이어받기 시작 띠의 단일 소비

Issue: #7445, 관련 #6776/#7518. 부모 `68af0105b0fd5d80addce5e711f0abf53a21fefc`, 고정 base `2e3b4f880bd11d35398e6ea0100846611f13a72d` 위의 로컬 보정이다. 전체 PR 준비 완료를 뜻하지 않는다.

## 독립 근거와 원인

원본 `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx`는 확장자와 달리 HWP5/한컴2018 저장본이다. SHA `5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617`, 독립 한컴2020 Print74쪽 PDF SHA `674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2`를 유지했다.

36쪽 표의 첫 본문 문단은 저장 LineSeg가 없지만 원본 ParaShape raw400과 독립 한컴 재저장의 before200HU가 앞 간격을 증명한다. 측정은 이미2.666667CSSpx를 예약했으나 paint와 가시 컷 경로는 저장 첫 vpos가 없다는 이유로 버렸다. 배치 상자가 없는 단 정의 메타데이터를 추가해도 텍스트 흐름의 소유는 바뀌지 않아야 한다.

앞 간격 복원 뒤37쪽이 악화되는 별도 원인도 해결했다. 원본 중첩 표의 시작 바깥여백141HU는 부모 컷의 continuation header에서 이미1.88CSSpx로 예약된다. `reflow_recursive_run_extra`가 자식 실제 RowCut 상자와 선택 유닛 합을 비교할 때 이 예약을 빼지 않아 부모 닫힘 높이에 다시 더했다. 실제 child 마지막 셀과 parent 종료 사이의 간격은5.64가 아니라 원본 child 하단141HU+parent 하단 padding141HU=3.76CSSpx다.

원 생산/소비 변경 `3d23545846942137256bc95afbdd8c6390fade42`(#7518)와 이어받기 바깥 상자 `41d371b1797d785cddcaad67fae7c2095f0d5c49`를 추적했다. 기존 물리 child RowCut·반복 제목행·캡션 예약은 유지하고 이미 따로 예약한 시작 띠를 물리 요구량 비교의 예약 합에 포함한다.

로컬 독립 인쇄 대조1027은 before147 제거 시36/37쪽 본문이 모두 이동하고 border37 두께 변경은 본문을 이동시키지 않음을 확인했다. 대조1029는 빈 nested host before157 제거가37쪽 후속 본문 기준선을 바꾸지 않음을 확인했다. 따라서37쪽 간격만 제거하는 후보1023은 폐기했고 특정 문서/행/쪽 예외나 고정 이동량으로 처리하지 않았다. 원본과 기준 PDF는 변경하지 않았다.

## 공통 소비

- `CellParagraphBefore`의 기존 측정 결과를 새 `plain_text_before` 조회로 실제 셀의 no-LS 텍스트·배치 중립 메타데이터에 공유한다. 실제 개체 호스트·원본 저장 줄·셀 밖 텍스트는 기존 경로다.
- 가시 컷 수용과 실제 composed paragraph 배치가 같은 flow-before를 소비한다. 부분 프레임 호출자의 명시 spacing은 덮어쓰지 않는다.
- 재귀 중첩의 실제 물리 요구량은 선택 유닛 합과 이미 예약한 continuation opening을 합친 값과 비교한다. ordinary/block capacity는 header를 먼저 예산에서 빼며, row-cut 실제 높이는 같은 header와 추가 물리 높이를 합산한다. 측정·컷·paint가 같은 공간을 한 번 소비한다.

## 시각 검증

1035가 Native/fresh WASM 전쪽을 모두 완료했다. 법령74쪽 최저96.05600/96.01774%,PrEP140쪽94.75150/94.22354%,RowBreak HWPX18쪽92.05401/92.05433%,HWP18쪽93.07060/92.96483%로 양쪽 각250쪽의 미달·누락·쪽수 불일치0이다. 별도 수도시설3쪽은기존71.85405/72.05077%로 미달이며,1046에서 새로 만든6PNG가 이전1000/1002와 byte 동일함을 확인했다. 통과 문서에 합치지 않았다.

1036은 이전 브랜치848/962와500쪽을 대응 비교해0.1%p 이상 하락0을 확인했다. PNG는471쪽 byte 동일/29쪽 변경이다. Native는 법령36/37쪽만 바뀌고,WASM은 다른25쪽에도 RGB 차이가 있어1047에 pixel 수·최대 채널 차이·영역과 정량 차이를 기록했다. 가장 큰 하락은PrEP8쪽0.01887%p로 승인된0.1%p 경계 미만이다. 이 비교는 최신 고정 devel 전체 보존 검증의 대용이 아니다. canonical96DPI/PDFium smooth_rgb_v1/raw2px90%·기존 입력/PDF/글꼴을 유지한다. 37쪽의 Native99.98294/fresh WASM99.98291%와36쪽96.05600/96.01774%,38쪽100/100%를 확인했다. 양쪽37쪽 review/overlay에서 닫힘·뒤 행의 baseline을 직접 판독했다. 일부 자형·선의 미세한 차이는 남아 있으며 엄격 픽셀 일치100%라고 표현하지 않는다.

Native1032 CLI SHA `2bf4b63f44ba13828c5a12d20edbcaf077396ed0ff6b8f1bb67ad87d5b3e311f`, fresh1034 WASM SHA `f646c427da6515092c63d02f31135edc138d999e58c9a6fbfbd96fcd729169fd`, JS SHA `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`다. Mac locked web/pkg/no-opt232.589초 exit0 로컬 대체 빌드이며 Docker 최적화 결과가 아니다. root pkg/Studio public 및1041의 실제 Studio7736 초기화 응답 hash 일치·initialized true·오류0을 확인했다. 문서 Actual36 검증은 별도 남는다.

## 정식 회귀와 기대값 검토

`tests/cases/issue_6776_unstored_cell_before_ownership.rs`의3건은 원문 앞 간격의 단일 소비, 단 정의 메타데이터 중립성, child 실제 마지막 셀→parent 하단 관계와 후속 셀 전체 내용·쪽 소속을 검사한다. 독립 Print37쪽의 원문 첫 글리프 x348.4511533, 후속 두 행 baseline609.893472/903.9720853CSSpx를 최종 Native Print paint의 원문 소유·runToPage로 ±2px 보호한다.36쪽은98% 미만이므로 좌표를 고정하지 않는다. 같은 닫힘 동작의37쪽이99%대여서 다른 내용인100%의38쪽으로 대신하지 않았다.

초안1042의 row4 전체가37쪽에만 있다는 가정은 독립 PDF37/38쪽에서 반증했다. 원래37→38쪽으로 이어지는 두 조각을 각각 한 번 소유하고 결합 내용이 원본과 같아야 한다고 바로잡았다. 제품을 바꾸거나 줄을 숨겨 그 가정을 통과시키지 않았다. 같은 최종 소스 frozen1000에서2PASS/1FAIL(닫힘 여백),1032에서3PASS를 확인했다. 최초 앞 간격 결측과 메타데이터 반례의 수정 전후 증거1008도 보존한다.

임시 query/JSON/로그·독립 대조 HWPX/PDF는 ignored output에만 보존한다. 1043은 prepare→fmt→prepare 뒤 실제 수집된suite009에서3PASS/0FAIL·262선택제외(실행1.653초·전체390.243초)다.1044의 기존8case35PASS/0FAIL을 더해 이 단계 집중 검사는38PASS다.1045의 manifest/check·unit-tier/check(base2e3)·수정 Rust 파일 fmt/check·git diff/check 모두 exit0이다.1043의 최초 호출은 같은 소유의 직접 rustc probe가 아직 실행 중이라 preflight에서 중단됐고 제품 검사는 시작하지 않았다. 해당 probe 종료를 확인한 정식 재실행만 수용했다. source-side 단위 검사 수를 늘리지 않는다.

## 남은 범위

#8064 Native 품질 하한의 법령2/13쪽·PrEP1쪽, 과학3/4쪽 미세 baseline, 고정 devel/후보 전체 영향 전쪽 TSV·전체 nextest/필수 gate·Actual36은 계속 검증·보정한다. 이 단계의 개선과 전체 PR 수용을 구분한다. PR/push/외부 게시에는 사용자 승인이 필요하다.

[전쪽 TSV·대응500쪽·입력/제품 SHA·대표 review/overlay](../pr/assets/issue7445/completed-20261009/stage27-20261021/README.md)
