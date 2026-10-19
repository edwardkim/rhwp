# #7642 유효한 저장 셀 어절 경계 보존

Issue: #7642.

## 원본과 독립 기대

원본은 `samples/hwp-3.0-HWPML.hwp`, 독립 기준은 사용자가 원본 그대로
Print한 `pdf/hwp-3.0-HWPML-2020.pdf`다. 양쪽 122쪽이며 producer
`9a1e7806`의 93쪽은 raw 60.52351%다. review/overlay에서 TABLE의
RepeatHeader 기본값 `true`가 한 줄로 합쳐지고 이후 행과 다음 표·캡션이
위로 당겨진다. Print는 `tru`/`e` 두 줄이다.

원본 HWP 파싱 결과의 저장 정보를 읽기 전용 HWPX 내보내기로 확인했다.
section3의 해당 표 row5/col4는 textpos 0/3, vertpos 0/1172인 두 저장 줄을
가지며 각 줄 높이 900, 저장 폭 1640HU다. 가공한 대조군을 원본의 독립
기준으로 쓰지 않는다. 진단 파일과 hash/명령은 ignored stage2863에 있다.

## 원인과 소비 경로

`recompose_cell_lines_in_frame`은 정상 저장 줄에
`UnmodelledUnlessStale` 수용 정책을 전달한다. 공통
`recompose_stored_lines_in_frame_with_known_square_band`는 이 정책
자체를 어절 꼬리 재구성의 이유로 취급한다. `can_rejoin_stored_word_tail`
→ `project_stored_soft_wraps`가 현재 측정에서 더 적은 줄이 된다는 이유로
캐시를 stale로 바꾼다. 그 결과를 셀 측정과 실제 배치가 함께 소비해 행과
후속 표가 압축된다. helper 자체의 주석은 줄 수 감소만으로 무효화하지
말라고 명시하지만 호출 조건이 그 계약을 위반한다.

수용 실패 정책을 독립적인 무효화 근거로 쓰지 않는다. 실제 공백 메트릭
변경, dirty partition, 저장 폭/현재 상자 불일치 등 기존 근거가 있는 경로는
유지한다. 문서·쪽·문구·고정 좌표 조건과 최소 행 높이 clamp를 추가하지 않는다.

## 검증

새 Native/fresh WASM으로 HWPML 전체 122쪽을 다시 비교하고, 해당 셀의
원문 두 줄·행 점유·다음 표와 캡션을 같은 영역에서 직접 판독한다.
공통 셀 경로의 시험지, 혼합 꼬리 문단·부분 행 경계, 명시적 편집/폭 변경,
KoPub 공백 갱신 대조군과 유입 PR #7969/#7970을 함께 검증한다.
전쪽 양쪽 90%와 직접 판독 뒤 새 이름의 관계 회귀를 추가하며, 실제
실패를 허용치나 baseline 갱신으로 숨기지 않는다.
