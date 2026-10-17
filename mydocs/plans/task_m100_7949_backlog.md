# Task #7949·#7950 — 후속 백로그

2026-10-17 후속 착수: 사용자가 지정한 원본 commit
`b7a5388034ee66f58a7e1d5363974e83b6374f57`의 이 문서는 현재 devel과 동일하다.
최신 `upstream/devel` `515bd72dcde0eecdeece333666b675354b7fa91d`에서 실행한
남은 결함과 분할 처리 순서는 [후속 수행·구현계획](task_m100_7949_backlog_followup.md)에
연결한다. 아래 기존 시각 판정과 예외는 당시 기록이며 최신 출력의 통과로 재해석하지 않는다.

- Issue: [#7949](https://github.com/edwardkim/rhwp/issues/7949), [#7950](https://github.com/edwardkim/rhwp/issues/7950)
- 작성일: 2026-10-16 (Asia/Seoul)
- 사용자 결정: 첫 재현 PDF의 출력 차이는 “후속 백로그 분리·시각 예외 수용”

| 대상 | 남은 차이·계약 | 현재 처리 |
| --- | --- | --- |
| MOF 1쪽 | 채움 복구 뒤 표 승격이 글자를 가리는 회귀를 추가로 확인해 positional 경로로 보존. `5c9242b3b`에서 Native 97.47152%, WASM 97.52634%. 글꼴·미세 간격 차이 일부 남음 | 초기 시각 예외 기록은 보존. 실루엣 점수와 실제 글자 가시성 검사를 구분하며 최종 devel 통합 결과는 stage7 참조 |
| 저해상도 Mask 확대 | 최종 이미지 paint의 보간과 PDF 샘플링 차이. 실제 저장 PNG alpha는 정상 | 합성 샘플의 decoded-resource 검사만 사용. 시각 미달을 golden으로 고정하지 않음 |
| Mask 범위 밖 | 색상 키 배열·독립 stencil·추가 codec·graphics-state soft mask | 명시적 미지원 유지 |
| 추가 paint 지원 | text/path 전체 혼합 순서, 부분 투명 텍스트·텍스트 clip·blend mode | 현재 path/image 순서와 실제 보이는 텍스트 보정을 전체 PDF paint 지원으로 확장해 주장하지 않음 |
| 배경과 겹치는 표의 의미 승격 | 현재 표 본문과 앞면 positional 개체의 paint plane이 달라 색 배경이 표 글자를 가릴 수 있음 | 원본 순서를 보장할 수 없는 추론 표는 보류하고 원본 개체·글자를 보존. MOF는 semantic table 3개 중 1개만 승격. 배경을 포함한 paint group의 편집 가능한 표 복원은 후속 구현 |

새 19쪽 PowerPoint 샘플의 SMask·Matte는 사용자의 추가 승인으로 **현재 구현 범위**로 이동했다.
첫 샘플의 예외는 이 문서의 전쪽 시각 판정을 면제하지 않는다. 개인 PDF·TSV·PNG는 ignored
`output/pr-review/issue7949/`와 사용자 검증 폴더에 보존하며 공개 fixture로 추가하지 않는다.

19쪽의 남은 출력 차이도 사용자가 이번 범위로 승인했다. 도형 채움·투명 paint 보정 뒤
Native 최저 99.40149%, fresh WASM 99.40886%와 사용자 직접 시각 통과를 확인했다.
상세 paint 보정은 [stage6](../working/task_m100_7949_stage6.md), 최종 표 가시성·통합 검증은
[stage7](../working/task_m100_7949_stage7.md)에 연결한다.

## 2026-10-17 첫 후속 절편

[#7974](https://github.com/edwardkim/rhwp/issues/7974)의 colour-key Mask·독립 stencil과
최종 sampling 전달을 구현했다. 작은 입력 Native/fresh Docker WASM 전쪽 최저
92.92381%/92.64496%, 마스크 핵심 재현본은 양쪽 100%다. 실제 Studio Canvas2D·CanvasKit·
DOM 경로와 저장 재열기도 집중 검증했다. 사용자 직접 시각 판정·전체 회귀·PR 게이트는 남는다.
세부 근거와 미검증은 [첫 구현 기록](../working/task_m100_7974_stage1.md)을 따른다.

추가 세부 백로그: base image와 stream mask가 모두 변화하고 서로 다른 Interpolate를 요구할
때, 단일 RGBA 합성·profile이라는 기존 근사를 별도 sampling plane 표현으로 대체한다. 이미
지원하던 입력은 유지하며 이 근사를 이번 절편의 완전한 sampling 해결로 표시하지 않는다.
