# Task #7949·#7950 — 후속 백로그

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 사용자 결정: 첫 재현 PDF의 출력 차이는 “후속 백로그 분리·시각 예외 수용”

| 대상 | 남은 차이·계약 | 현재 처리 |
| --- | --- | --- |
| MOF 1쪽 | 표의 파란 배경·흰색 글자, 글꼴·간격. Native 82.79780%, WASM 83.29803% | 기본 빌드·명시적 Mask 범위의 예외 승인. path 채움·문자 paint의 별도 보정 대상 |
| 저해상도 Mask 확대 | 최종 이미지 paint의 보간과 PDF 샘플링 차이. 실제 저장 PNG alpha는 정상 | 합성 샘플의 decoded-resource 검사만 사용. 시각 미달을 golden으로 고정하지 않음 |
| Mask 범위 밖 | 색상 키 배열·독립 stencil·추가 codec·graphics-state soft mask | 명시적 미지원 유지 |

새 19쪽 PowerPoint 샘플의 SMask·Matte는 사용자의 추가 승인으로 **현재 구현 범위**로 이동했다.
첫 샘플의 예외는 이 문서의 전쪽 시각 판정을 면제하지 않는다. 개인 PDF·TSV·PNG는 ignored
`output/pr-review/issue7949/`와 사용자 검증 폴더에 보존하며 공개 fixture로 추가하지 않는다.
