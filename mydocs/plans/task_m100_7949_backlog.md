# Task #7949·#7950 — 후속 백로그

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 사용자 결정: 첫 재현 PDF의 출력 차이는 “후속 백로그 분리·시각 예외 수용”

| 대상 | 남은 차이·계약 | 현재 처리 |
| --- | --- | --- |
| MOF 1쪽 | 초기 파란 배경·흰 글자 누락은 공통 paint 보정으로 해소. 현재 Native 97.39936%, WASM 97.47529%. 글꼴·미세 간격 차이 일부 남음 | 초기 82.79780%/83.29803%의 예외 기록은 보존. 현재 전쪽 90% 이상이며 과거 점수를 현재 결과로 쓰지 않음 |
| 저해상도 Mask 확대 | 최종 이미지 paint의 보간과 PDF 샘플링 차이. 실제 저장 PNG alpha는 정상 | 합성 샘플의 decoded-resource 검사만 사용. 시각 미달을 golden으로 고정하지 않음 |
| Mask 범위 밖 | 색상 키 배열·독립 stencil·추가 codec·graphics-state soft mask | 명시적 미지원 유지 |
| 추가 paint 지원 | text/path 전체 혼합 순서, 부분 투명 텍스트·텍스트 clip·blend mode | 현재 path/image 순서와 실제 보이는 텍스트 보정을 전체 PDF paint 지원으로 확장해 주장하지 않음 |

새 19쪽 PowerPoint 샘플의 SMask·Matte는 사용자의 추가 승인으로 **현재 구현 범위**로 이동했다.
첫 샘플의 예외는 이 문서의 전쪽 시각 판정을 면제하지 않는다. 개인 PDF·TSV·PNG는 ignored
`output/pr-review/issue7949/`와 사용자 검증 폴더에 보존하며 공개 fixture로 추가하지 않는다.

19쪽의 남은 출력 차이도 사용자가 이번 범위로 승인했다. 도형 채움·투명 paint 보정 뒤
Native 최저 99.40149%, fresh WASM 99.40886%와 사용자 직접 시각 통과를 확인했다.
상세 범위·남은 제품 검증은 [stage6](../working/task_m100_7949_stage6.md)에 연결한다.
