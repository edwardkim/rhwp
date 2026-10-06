# M100 #2513 Stage 1 완료 보고서 - SDK loadFile 옵션 계약

## 1. 결론

`@rhwp/editor.loadFile()`에서 `suppressDialogs`를 생략한 embed 호출은 이제
`suppressDialogs: true`를 전송한다. 호출자가 `false` 또는 `true`를 명시하면 그 값을 그대로
보존한다.

이 변경은 iframe 내부 안내창을 직접 누를 수 없는 embed 사용자의 기본 호출이 안내창 응답에
묶이는 원인을 SDK 경계에서 제거한다.

## 2. 변경 파일

- `npm/editor/index.js`: 생략 여부를 구분하는 기본값 계산 1줄 변경
- `npm/editor/tests/load-file-options.contract.test.mjs`: 생략, 명시 `false`, 명시 `true` 계약 추가
- `mydocs/orders/20260720.md`: 승인 및 Stage 1 완료 상태 반영
- `mydocs/plans/task_m100_2513.md`: 승인 기록
- `mydocs/plans/task_m100_2513_impl.md`: 승인 기록

## 3. RED 증거

명령:

```bash
rtk node --test npm/editor/tests/load-file-options.contract.test.mjs
```

수정 전 결과: exit 1. 옵션 생략 경로가 기대값 `true` 대신 실제값 `false`를 전송했다.
명시 `false`와 명시 `true` 경로는 기존에도 기대값과 일치했다.

## 4. GREEN 증거

```bash
rtk node --test npm/editor/tests/load-file-options.contract.test.mjs
```

- exit 0, 1/1 통과

```bash
rtk npm --prefix npm/editor test
```

- exit 0, 19/19 통과

```bash
rtk git diff --check
```

- exit 0, 출력 없음

## 5. 범위 보호

Stage 1에서는 다음을 변경하지 않았다.

- `npm/editor/index.d.ts`와 `npm/editor/README.md`
- `rhwp-studio/e2e/embed-transport.test.mjs`
- top-level Studio와 raw embed protocol의 기본값
- transport, MessageChannel version, origin/session 검증

따라서 실제 fresh iframe 회귀와 공개 문서 동기화는 Stage 2에서 별도로 검증한다.

## 6. 독립 검토

- 판정: PASS
- 확인: 구현 1줄과 계약 테스트 1개만 Stage 1 소스 변경
- 확인: 생략 → `true`, 명시 `false` → `false`, 명시 `true` → `true`
- 확인: Stage 2 대상 파일은 `HEAD` 대비 변경 없음
- 확인: 전체 Cargo/Studio gate는 Stage 3 전까지 실행하지 않음

## 7. 다음 단계와 승인

Stage 2 범위:

1. embed E2E에서 안내창 polling/click fallback을 제거한다.
2. fresh iframe의 zero-option `loadFile()`이 사용자 입력 없이 완료되는지 확인한다.
3. `index.d.ts`와 README의 기본값 및 예제를 실제 계약과 맞춘다.

- Stage 1 상태: 완료, 독립 검토 PASS
- Stage 2 상태: 2026-07-20 작업지시자 승인 완료
- 후속 상태: 전체 gate, upstream PR 생성 및 리뷰 요청까지 승인 완료
- 커밋, push, PR: 아직 수행하지 않음
