# #8174 T04 B03 Native 실행 증거

고정 전 source `bfafd891f6`와 후보 `1611ec923e32a3c539750241c675fe84cebf1223`를 대조했다.
[실행 기록](../../../../working/task_m100_8174_t04_allocation_audit.md)에 호출 경계와
인덱스/메트릭 계상의 분리, 실행 명령·실패 보정·잔여 범위를 기록했다.

`contracts.json`은 실제 수정 전 4 FAIL/12 PASS와 후보 Native 167 PASS 및 backend Clippy다.
`native-producer.json`은 실제 최적화 CLI와 적용 제품 파일의 SHA-256을 고정한다.
`native-smoke.json`은 CFF·Type3·익명 maintainer 문서의 HWP/HWPX 저장·전쪽 재열기와
작은 font 한도의 거절·기존 출력 보존·정상 후속 요청이다. source 경로·원본·개별 실행 로그는
ignored output에 두고 공개하지 않는다. 로그와 중간 JSON은 이 폴더에 복사하지 않았다.

이 절편은 B03의 Native 증거다. T04의 나머지 frontend/Core/직렬화와 최종 fresh WASM·시각·
성능은 완료를 주장하지 않는다. CLI smoke 중 Clippy가 겹쳤으므로 wall/RSS를 비교 측정으로
사용하지 않는다. 파일·네트워크 confinement도 별도 T05의 미완료 항목이다.
