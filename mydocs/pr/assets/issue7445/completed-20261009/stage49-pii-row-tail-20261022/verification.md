# Stage49 개인정보 2→3쪽의 독립 행 잔여

현재 로컬 보정 단계의 Native 증거다. 전 문서 수용·fresh WASM·전체 회귀·PR 준비 완료를 뜻하지 않는다.

## 입력과 생산 출처

- 부모 HEAD `338f9b58ed8597613b74a74ef2663350b18cca7b`, 고정 base `2e087a5b4bc1515ad877e8020b30114639a59392`.
- 원본 `samples/task2097/75544_pii_bunseok.hwpx`, SHA `f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`.
- 독립 한컴2020 Print `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`, SHA `0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`, 66쪽.
- 선언 환경 `mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json`, SHA `21e5c5a0b081bf3101749e5b80c1b053fa3eeff4fcb1ba3f8aa33fd3e2a4d544`. 설치 글꼴 원본·공급 경로는 ignored `candidate-source.json`의 hash 감사로 고정했다.
- Native 후보 P33 CLI SHA `e1c67e6ca913be0e62f25ec51503399c3fb20a52a0f042493848cd87b2bff068`, source patch SHA `703464a623523704b2d0df5c62500dab3310fcaef408fc23903f4e6241f8914d`.
- source/hash/명령/결과는 ignored `rebase-7752-20261022/repair-focused-v33/` 및 `pii-row-owner-native-v33/producer.json`에 보존한다. Cargo jobs8·test threads8·capture 총8·Native print·compat2022·96dpi·canonical PDFium152.0.7947.0 `pdfium_smooth_rgb_v1`·2px/90%를 유지했다.

| Runtime 파일 | SHA256 |
| --- | --- |
| `src/renderer/layout/table_layout.rs` | `3c2ffe1292c117922e46c99d38183f28af50c7e126de72ccee235ec573404300` |
| `src/renderer/layout/table_layout/stored_ordinary_row_frame.rs` | `164f3f7e16d77ddd02ceb6a5317f634edc344db368b750b8d41adb45dd93ac21` |
| `src/renderer/typeset/table/continuation/fragment/scan.rs` | `041b79efc766bf524e0fc994c5a3477ed4d960a34afcd191cb5a599720db9bf9` |
| `src/renderer/typeset/table/scan.rs` | `5cd98c2baa4fb01c52910055790cf09289813b3e91dce5e276ba5e352ab14c94` |

## 실제 검사와 전쪽 결과

- 기존 #5846 검사는 변경하지 않았다. P31은 본문 포함에서 실패, P32는 3쪽 빈 행 뒤 비용 시작에서 실패했다. P33은 그 앞의 본문 포함·독립 Print 좌표·빈 행 소유·내용 보존 검사를 모두 통과하고 이후 60쪽 중첩 표/다음 항목의 겹침에서 실패했다. 그 함수 전체를 PASS로 보고하지 않는다.
- P32 집중 95건 94 PASS / 1 FAIL. P33 집중 37건 36 PASS / 1 FAIL. #7752·#8105·#7518·#7870 정상 대조군을 유지했다.
- P33 기존 실패 57건 재실행은 37 PASS / 20 FAIL이며 #7752는 PASS다. 전체 회귀는 아직 남아 있다.
- base/전/후 각 전66쪽 TSV를 산출했다. 2쪽 **95.04886→100.00000%**, 3쪽 **44.89709→99.83156%**. 나머지64쪽 PNG는 수정 전과 byte동일, 수정 전 대비0.1%p 이상 하락0쪽이다.
- 같은 표·셀·문단 소유자별 연결 문자열1828개도 전66쪽에서 동일하다. 소유자 사이의 전역 순서 증거와 혼동하지 않는다. 2·3쪽의 내용·누락/중복·빈 행과 후속 행 순서는 기존 정식 #5846 검사가 별도로 확인했다.
- 후보 최저59쪽46.20358%, 미달57·59·60·66쪽. 고정 base 대비41쪽99.23712→97.93657% 하락도 남아 있어 전체 수용은 보류다.
- 새 테스트·px 핀·기대값 갱신은 없다. 현재 source의 fresh WASM·전체 게이트·actual36은 미완료다.

[base 전쪽 TSV](base-native.tsv) · [수정 전 전쪽 TSV](before-native.tsv) · [수정 후 전쪽 TSV](candidate-native.tsv)

## 직접 판독

2·3쪽 수정 전후 review와 standalone overlay를 열어, 앞쪽 영향평가 내용은 한 번만 남고 다음 쪽 상단의 빈 행 뒤에 비용 행이 시작하는 것을 확인했다. 3쪽 하단까지 후속 행 소속과 외곽을 확인했다. 얇은 괘선·일부 글리프 프린지 차이를 완전 픽셀 일치라고 표현하지 않는다.

| 페이지 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 2 | [review](before-p002-review.png) / [overlay](before-p002-overlay.png) | [review](candidate-p002-review.png) / [overlay](candidate-p002-overlay.png) |
| 3 | [review](before-p003-review.png) / [overlay](before-p003-overlay.png) | [review](candidate-p003-review.png) / [overlay](candidate-p003-overlay.png) |

60쪽의 큰 간격·표 외곽 차이는 ignored 실행의 review/overlay에서 직접 판독했고 계속 보정한다. P34의 종료 질의 복구 시도는95건94 PASS/1 FAIL이며 P33과 전66쪽 PNG/TSV가 byte동일했다. 이 불충분한 시도와 P35의 임시 배치 추적은 최종 구현에서 제거하고 위 P33 source hash와 정확히 일치시켰다.

## 보존

TSV·PNG는 정식 증거로 바이트 그대로 복사했다. 로그·명령·중간 JSON·진단 SDK·임시 소스·원본 글꼴은 ignored output에만 보존하고 커밋하지 않는다. 원본·PDF·공유 target·다른 작업장은 보존했다.
