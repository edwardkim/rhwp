# Stage71: 원본 인라인 조각 완결과 캡션 새 쪽 예약 보정

Issue: #7445 / task1725 / #7379. 부모 HEAD `46fe448a0f`. 기존 입력·검사를 보정했으며 새 HWP 대상은 추가하지 않았다.

## 규칙과 원인

원 해결 task1725 `54c01b3420335d39507fd6252537e824118a7a91`(통합 `9853e8a1`)과 캡션 보정 `f72f1a68427`을 추적했다. 유효한 저장 글줄의 인라인 개체는 같은 CellUnit 내용과 패딩이 원본 조각에 들어가면 완결한다. 텍스트 전용 guard가 그 소유자를 거부해 0·1행 중 앞 행만 남겼다. 저장 상자 6303HU, 마지막 행 내용 19.76px, 잔여 41.89333px를 독립 원본·Print와 대조했다.

`stored_inline_content_cell_frame`의 저장 인증 → `stored_complete_source_row_content_fits`의 실제 유닛 완결 → `stored_partial_fragment_height`의 마지막 행 상자 → scan의 cut/fit 배열과 end-row override → 기존 emit/paint가 같은 물리 높이를 받는다. 새로운 문서/쪽 분기나 0.5px 허용을 넣지 않았다. 중첩 표와 floating/각주 개체·저장 재시작·수정된 줄은 제외한다.

아래 캡션은 아직 소비하지 않은 마지막 병합 블록과 실제 종료 예약이 새 쪽에 들어가면 블록을 함께 재스캔한다. 새 쪽보다 큰 블록이나 앞에서 일부 소비한 블록에는 이 조건을 적용하지 않는다. 기존 prepared total_footnote에는 현재 쪽의 다른 본문 각주가 포함될 수 있으므로 fresh 예약으로 쓰지 않는다. 마지막 블록의 실제 note 목록과 current/fresh가 공유하는 구분선·사이 간격 계산을 사용한다. 원본 raw2/CELL 내부 분할은 유지한다.

## 실제 실행

초기 후보 두 번은 각각 19 PASS / 1 FAIL이었다. 캡션의 현재 쪽 누적 각주를 새 쪽 예약과 혼동한 원인을 확인해 보정했다. 최종 정식 두 모듈은 20 PASS이며, 합성 셀 병합 뒤 주소 격자 재구성을 포함한 재실행도 20 PASS / 5.050초다. 검사 기대값·skip·검사 수는 바꾸지 않았다. 임시 진단 빌드의 private API 접근 오류는 고쳐 재실행했으며 제품 결함 재현이나 최종 성공 증거로 세지 않았다. 진단용 eprintln은 모두 제거했다.

동일 원본·독립 Print·같은 글꼴·96 DPI·PDFium153·2px raw로 Native와 fresh WASM 전 242쪽을 산출했다. Native 전후는 29쪽 96.36000→99.87634%, 30쪽 13.12266→97.08818%이며 다른 240쪽은 동일하다. fresh WASM 29·30쪽은 99.87630/97.08815%다. 양쪽의 201쪽은 각각 71.82002/71.95793%로 아직 미달이다. 29·30쪽 소유와 이어받기를 직접 판독했으며 수식 자간·자형 가장자리·얇은 괘선 차이는 남는다.

Mac fresh WASM은 locked wrapper web/pkg/no-opt 로컬 대체 빌드다. root pkg/Studio public/보존 package hash는 같지만 실제 Studio 초기화·Actual36 검증은 아직 아니다. 완료 생성 SVG만 manifest 후 정리하고 입력·PDF·PNG·TSV·로그는 보존했다.

[원본·producer hash·전쪽 TSV·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-7bb143879-20261023/complete-inline-source-frame/README.md). 원 로그는 ignored `output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage71-remaining-contracts/`에 있다.

201쪽·overlap partition12·C354/C356, 최종 전체 회귀·범위 게이트·devel 비교·실제 소비 검증은 계속 필요하다. 새 픽셀 핀은 전쪽 90% 선행 조건 때문에 추가하지 않았다. 원격 push·PR·merge는 실행하지 않았다.
