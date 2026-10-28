# #4051 Stage 1 - batch-convert 메인터너 보정 초기 구현

## 구현 내용

- HWP/HWPX 동일 stem 출력 충돌을 변환 시작 전에 거부하도록 추가했다.
- 일부 포맷 실패를 파일 실패로 집계하고, rhwp exit 2를 비재시도 오류로 분류하도록 구현했다.
- mock 기반 충돌·부분 실패·exit 2 재시도 회귀 테스트와 README 계약을 추가했다.

## 테스트 결과

`CARGO_TARGET_DIR=target/review-kevin9327-4052-20260806 CARGO_INCREMENTAL=0 cargo test -p batch-convert`는
컴파일 단계에서 실패했다.

```text
converter.rs: export_once의 반환형은 anyhow::Result인데,
run_rhwp_export의 RhwpExportError 결과를 직접 반환해 E0308 타입 불일치 발생
```

## Stage 2 계획

- `run_rhwp_export(...)?`를 `Ok(...)` 경계로 감싸 `RhwpExportError`의 concrete type을 anyhow chain에
  보존한다.
- 같은 통합 테스트를 다시 실행해 exit 2의 downcast 기반 비재시도 분기가 실제로 동작하는지 확인한다.
