# #4051 Stage 2 - 오류 타입 경계 보정 계획

## 목표

Stage 1의 `RhwpExportError`와 `anyhow::Result` 반환 경계 컴파일 오류를 보정한다.

## 변경

- `export_once`에서 typed command 오류를 `?`로 변환하고 `Ok(())`로 반환한다.
- `export_with_retry`의 `downcast_ref::<RhwpExportError>()`가 exit 1만 재시도하는지 통합 테스트로 확인한다.

## 완료 조건

- batch-convert 통합 테스트 전체 통과
- 충돌 사전 거부, 부분 포맷 실패 exit 1, exit 2 단일 호출 회귀 테스트 통과

## 테스트 결과

오류 타입 경계 보정 뒤 `cargo test -p batch-convert`는 컴파일에 성공했고, 새 부분 포맷 실패와
exit 2 비재시도 테스트도 통과했다. 그러나 동일 stem 충돌 테스트가 출력 루트 디렉터리 존재를
관측해 실패했다.

```text
duplicate_hwp_and_hwpx_stems_are_rejected_before_conversion
assertion failed: !bed.output.exists()
```

원인은 `BatchConverter::new`가 input 탐색·충돌 검증 이전에 output root를 생성하기 때문이다.

## Stage 3 계획

- output root 생성은 `convert_batch`에서 pattern filter 적용 후 충돌 검증이 성공한 뒤에만 수행한다.
- dry-run도 output root를 만들지 않도록 같은 순서를 사용한다.
