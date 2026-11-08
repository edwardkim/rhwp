---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-8403/validation/inputs/README.md
last_verified: 2026-10-31
---

# 공개 HWP/HWPX 재현 입력

[`samples/issue8403/`](../../../../../../samples/issue8403/)의 `patterns.*`, `stroke.*`는 [공개 PDF](../../../../../../pdf/issue8403/README.md)를
Native CLI의 `import-pdf`로 변환한 실제 저장본이다. `stroke-edited.*`는 저장본의
`Stroke` 14곳을 `ktroke`로 치환한 결과다. 생성 코드만 제공하는 대신 검증에서 다시 연
바이트를 그대로 보존했다. [manifest.json](manifest.json)에 원본 PDF·크기·SHA-256을 기록했다.

| 입력 | 대응 기준 | 전체 쪽수 |
| --- | --- | ---: |
| `patterns.hwp` / `patterns.hwpx` | `pdf/issue8403/image-patterns.pdf` | 5 |
| `stroke.hwp` / `stroke.hwpx` | `pdf/issue8403/text-stroke.pdf` | 7 |
| `stroke-edited.hwp` / `stroke-edited.hwpx` | `pdf/issue8403/text-stroke-edited.pdf` | 7 |

이 자료는 PDF 연산자로 작성한 공개 합성 대조군이다. 한컴이 작성한 문서나 한컴 인쇄 결과로
표시하지 않는다. 독립 PDF 렌더러와 Native/fresh WASM의 전쪽 검증 결과는 상위
[검증 기록](../README.md)에 연결한다. 비공개 사용자 원본과 그 변환본은 포함하지 않는다.

정밀 문자 획은 RHWP의 제한된 확장으로 왕복 보존한다. 다른 편집기의 기본 문자 효과와
동등한 표현을 보장하지 않으며, 저장 시 `MetadataReduced` 진단을 유지한다.
