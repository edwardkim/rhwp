# Native 제품 보정 단계 결과

> 이 문서는 당시 단계의 분석 기록이다. 최종 결과와 대표 증적은 [검증 기록](../verification-record.md)과 [최종 대상 증적](../final/main/README.md)을 따른다. 원시 로그·중간 분석 JSON·중간 PNG는 제출에서 제외했다.

진단 프로그램의 설정 변경을 제품의 공통 Native Print 재생 정책에 반영했다.
실제 새 제품 binary로 기존 한컴 Print PDF와 전 14쪽을 다시 비교했고,
**Native 모든 페이지의 2px raw 일치율이 100.00%**다. 원본·PDF·원점·글자 크기·장평·
비교 영역·판정 임계값은 바꾸지 않았다. 엄격한 RGB 동일성을 뜻하지 않는다.

| 검증 | 실제 결과 |
| --- | --- |
| 최소 글꼴의 수정 전 관계형 회귀 | FAIL: 원본 1/4px 이동에 실제 잉크 이동 0 |
| 수정 후 Print·Screen·캐시 전환 집중 검사 | 6 PASS |
| 최소 글꼴 Native `--lib` | rhwp 3,853·관련 workspace 182 PASS, 12 ignored |
| Native 그림 누락 보호 | 2 PASS |
| Native 직접 PDF 출력 | 36 PASS |
| 실제 제품 전 14쪽 Native | 모두 raw 100.00% |

추가 검사는 저장소의 공개 TTF만 명시 공급한다. 사설 설치 글꼴을 사용하거나 글꼴이 없을 때
검사를 건너뛰지 않는다. 합성 입력의 상대 이동을 검사하며 실문서의 절대 좌표 핀을 추가하지 않았다.
시각 측정은 별도로 PDF와 일치하는 정상 로컬 글꼴 프로그램을 공급했다. 글꼴 파일은 배포하지 않는다.

변경 경로는 `PageLayerTree`의 Print profile → Native `ReplayCacheContext.geometric_text`
→ `SkiaTextReplay` → 실제 Font 설정·단색 glyph 윤곽 재생이다. Native Print는 이전 인쇄
세대에서도 같은 소수 원점·윤곽을 사용한다. 실제 Native 정책을 캐시 신원에 포함했다.
화면·벡터 출력과 공유 layer·Canvas의 세대별 정책은 변경하지 않았다.

5쪽에 표 첫 줄, 6쪽에 나머지 두 줄, 7쪽에 후속 본문이 있다. 원본 셀의 공백을 정규화한
전체 내용과 각 조각의 독립 PDF 대응을 확인해 누락·중복이 없음을 검증했다.

- [Native 전쪽 TSV의 검증 요약](../verification-record.md)
- [생산·입력 SHA와 단계 검증 영수증](../verification-record.md)
- [실제 측정 글꼴 환경](font-environment.json)
- 5쪽: [review](../verification-record.md) · [overlay](../verification-record.md)
- 6쪽: [review](../verification-record.md) · [overlay](../verification-record.md)
- 12쪽: [review](../verification-record.md) · [overlay](../verification-record.md)

이 기록은 Native 보정 단계의 완료다. 전체 Rust·lint·최종 fresh WASM·정본 보호 전쪽 검증과
상승 기준 반영·최종 PR review·원격 CI·병합은 아직 완료로 판정하지 않는다.
