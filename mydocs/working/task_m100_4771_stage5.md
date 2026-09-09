# Task M100 #4771 Stage 5 — snapshot-only HWP lowering

- 모든 stateful HWP export API를 `prepare_hwp_export_snapshot` 경계로 모았다.
- CLI verify는 별도 lowering을 재구현하지 않고 바이트를 만든 동일 snapshot을 expected IR로 사용한다.
- 일반·report·password·verify API 호출 전후 live `Document` 구조 동일성을 검증했다.
