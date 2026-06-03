# Task M100-1211 Stage 4 완료 보고 — 검증

## 자동 검증

```text
cd rhwp-studio && npm test
```

결과: 통과.

```text
tests 52
pass 52
fail 0
```

```text
cd rhwp-studio && npm run build
```

결과: 통과.

Vite 빌드 중 `fs`/`path` browser externalization 및 500 kB 이상 chunk 경고가 출력되었으나, 기존 CanvasKit/Vite 빌드 경고이며 이번 변경의 실패는 아니다.

## 수동 검증

브라우저에서 다음 URL로 `samples/exam_social.hwp`를 로드했다.

```text
http://127.0.0.1:7700/?url=/samples/exam_social.hwp&filename=exam_social.hwp
```

확인:

- 1쪽 상단 `성명` 칸에 `테스트` 입력.
- 입력 텍스트가 셀 내부에 반영됨.
- 브라우저 console error 없음.

## 검증 결론

이번 변경은 `exam_social.hwp`의 셀 내부 단일 텍스트 입력에 대해 full `document-changed` refresh 대신 page-local invalidation을 사용한다. 빌드/테스트/수동 입력 확인 기준으로 기능 회귀는 발견되지 않았다.
