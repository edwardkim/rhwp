---
kind: guide
status: active
canonical: mydocs/tech/document_font_preferences.md
last_verified: 2026-11-07
---

# 문서별 사용자 글꼴 선택

기본 원본·대체 정책은 [글꼴 대체 전략](font_fallback_strategy.md)을 따른다.
이 문서는 사용자가 특정 원본에 명시적으로 선택한 표시 프로그램의 추가 계약이다.

기준 인쇄가 원본과 다른 글꼴을 사용했고 사용자가 같은 표시를 선택한 경우, 일반 전역
대체표를 바꾸지 않고 원본 바이트의 BLAKE3 digest에 묶인 설정을 사용할 수 있다.
Studio URL의 `font-environment` 값은 다음 JSON을 URL 인코딩한 것이다.

```json
{"documentDigest":"blake3:<원본 64자리 hex>","substitutions":{"원본 글꼴":"선택한 표시 글꼴"}}
```

같은 원본을 열 때 이 선택을 로컬 저장소 `rhwp-document-font-preferences`에서 읽는다.
다른 원본에는 적용되지 않으며 HWP/HWPX의 글꼴 선언이나 파일 바이트를 수정하지 않는다.
선택한 대상의 실제 프로그램이 준비된 경우에만 layout와 Canvas2D paint가 같은 환경을
소비한다. 대상의 이름 probe만 성공한 경우를 실제 프로그램 공급 성공으로 세지 않는다.
이 경로는 브라우저 Canvas2D의 사용자 표시 설정이며 Native CLI나 CanvasKit의 성공을
대신하지 않는다. 설정이 없는 일반 문서는 기존 원본 우선 선택을 유지한다.


## 선택 범위와 런타임 호환

새 WASM은 글꼴 선택 계획에 `supportsSourceMetricOverrides: true`를 반환한다.
Studio는 이 경우 자동 공급의 `preserveSourceMetrics`를 유지하고 사용자가 선택한 원본만
`replaceSourceMetricsFor`에 기록한다. 다른 원본의 메트릭·물리 프레임은 변경하지 않는다.

기존 `substitutions`·`trueTypeFaces` 선언에서 `preserveSourceMetrics`를 생략한 경우는
자연 윤곽·반복 하이픈 전진을 새로 선택한 요청으로 해석하지 않는다. 기존 메트릭 치환과
leader 규칙을 유지한다. 명시적인 `preserveSourceMetrics: false` 또는 해당 원본의
`replaceSourceMetricsFor` 선택이 있을 때 자연 프로그램 윤곽을 적용한다. 생략과 명시 false는
환경 JSON의 직렬화·재열기에서도 구분하며, `null`은 허용하지 않는다.
지원하지 않는 이전 WASM에는 이 새 필드를 보내지 않고 이전 명시 환경 계약을 유지한다.
최종 수용 검증은 새 WASM에서 원본별 범위를 실제 확인한다.

글꼴 감지 결과 초기화는 감지 목록·실제 로컬 face 캐시를 삭제한다. 문서 digest에 묶인
사용자 표시 설정을 다른 문서에 전파하거나 초기화 버튼에서 감지를 다시 요청하지 않는다.
다음 문서 로드에서 사용 방법을 선택하고 실제 프로그램 공급 뒤 첫 화면을 준비한다.

## 검증과 출처

- `rhwp-studio/tests/document-font-preferences.test.ts`: 선택 원본의 저장·공급·이전 런타임 호환.
- `tests/cases/issue_1994_document_font_program_scope.rs`: 공개 API로 생성한 문서에서 선택
  원본의 자연 전진과 다른 원본의 실제 paint 스타일·전진·저장 바이트 보존.
- [감지·초기화 실제 Chrome 증거](../pr/assets/issue7375/font-reset-20261107/README.md).

원 글꼴 전략 문서는 #7023의 기존 SHA 기반 근거이므로 그 원문을 보존하고, 새 사용자 표시
기능의 설명을 이 별도 문서로 둔다. 기존 글꼴 규칙·근거 해시·sealed v1은 수정하지 않는다.
