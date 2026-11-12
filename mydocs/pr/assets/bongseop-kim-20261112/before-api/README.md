# 수정 전 API 결과의 진단 보존

기준 devel `80b1c354c29b`의 실제 공개 HTML 붙여넣기 API가 만든 **잘못된 저장 결과**다. 정상 조판 fixture/golden으로 등록하거나 새 코드의 기대값으로 쓰지 않는다. 동일 HTML의 의도된 속성은 현재 저장본과 그 독립 Print에서 확인한다. `before-view-receipts.json`과 `direct-review/before-after/`가 이 저장본의 Native/fresh WASM 표시를 기록한다. 별도로 `focus-base-head-comparison.json`은 **완전히 같은 정상 입력 파일**의 base/head 전쪽 대조이며 이 진단과 구분한다.
