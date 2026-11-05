# 리베이스 후 각주 선 굵기 복구

사용자가 지적한 `rebase-ece9-r1-pre-rebase-native-comparison.json`의 네 하락을
제품 코드 `dc39658f812d22c961ddd0fbc8e71c7ec7a52e5b`에서 보정했다.
이전 보고서는 당시 실패 기록으로 보존하며 수치를 덮어쓰지 않았다.

| 원본 | 쪽 | 리베이스 전 | 보정 전 R1 | 보정 후 Native | 보정 후 fresh WASM |
| --- | ---: | ---: | ---: | ---: | ---: |
| `note-formats.hwp` | 10 | 100.00% | 97.31% | 100.00% | 100.00% |
| `note-formats.hwp` | 11 | 100.00% | 96.53% | 100.00% | 100.00% |
| `note-formats.hwpx` | 10 | 100.00% | 97.31% | 100.00% | 100.00% |
| `note-formats.hwpx` | 11 | 100.00% | 96.53% | 100.00% | 100.00% |

원인은 각주 복선·삼중선을 BorderFill 생성기에 연결하면서 표 테두리용 3px/4px 최소
밴드까지 적용한 것이다. 각주의 원본 선언 굵기를 하나의 공통 `LineRenderType` 노드에
전달하는 경로를 복구했다. 일반 파선·점선, 새 base의 표 테두리·원형·물결 구현은 유지한다.
문서별 수치 보정이나 기존 기대값·통과 기준 완화는 하지 않았다.

## 입력·출력과 측정

- 원본: [HWP](../../../../../tests/fixtures/number_shapes/note-formats.hwp),
  [HWPX](../../../../../tests/fixtures/number_shapes/note-formats.hwpx).
  임시 수집본과 기존 정식 fixture의 SHA-256이 각각 동일하다.
- 독립 기준: [한컴 Mac 12.30 Print PDF](../../../../../pdf/number_shapes/note-formats-hancom-mac-12.30.pdf).
  원 PR의 기존 Print를 재사용했으며 PDF를 재산출하거나 변경하지 않았다.
- 방법: 인쇄 프로필, compat 2022, 96dpi, 색상 경계 보정 전 2px 내용 실루엣 raw.
  전 19쪽과 PDF 쪽수가 일치하고 누락·90% 미만은 없다.
- HWP/HWPX 각각 Native·최적화 fresh WASM의 전쪽 최저값은 모두 **99.98%**다.
  대상 8·9·10·11쪽은 네 측정 모두 **100.00%**다.
- [Native HWP TSV](hwp-native.tsv), [WASM HWP TSV](hwp-wasm.tsv),
  [Native HWPX TSV](hwpx-native.tsv), [WASM HWPX TSV](hwpx-wasm.tsv).
- 실행 명령·입력/PDF/producer SHA·측정 완료 시각은 [검증 출처](../historical-validation.md)에 기록했다. Native/fresh WASM은 같은 제품 source를 사용했다.
- 시각 측정에는 설치된 시스템 글꼴 환경을 사용했다. 회귀 실행은 별도의 최소 글꼴
  환경에서 수행하며 글꼴 파일을 추가 배포하지 않는다.

## 직접 판독

같은 실제 측정 PNG를 canonical Visual Sweep 함수로 합성했다. 재렌더나 좌표 변경은 없다.
독립 PDF와 Native/fresh WASM의 10·11쪽을 같은 배율로 확대해 선 종류·굵기·각주 내용과
소유 영역을 대조했다. 표 테두리 최소 밴드가 각주에 확대 적용되던 차이는 제거됐다.
100.00%는 2px 실루엣 지표이며 정확한 RGB 일치를 뜻하지 않는다. 선 경계의 잔여 색상 차이는
review·overlay에서 함께 확인할 수 있다.

| 쪽 | Native | fresh WASM |
| --- | --- | --- |
| 10 | [review](hwp-native-p010-review.png) · [overlay](hwp-native-p010-overlay.png) | [review](hwp-wasm-p010-review.png) · [overlay](hwp-wasm-p010-overlay.png) |
| 11 | [review](hwp-native-p011-review.png) · [overlay](hwp-native-p011-overlay.png) | [review](hwp-wasm-p011-review.png) · [overlay](hwp-wasm-p011-overlay.png) |

## 보존 범위와 회귀

리베이스 전 `809097923a0d8f10c24eb3d42572b2888954a121`의 Native 56사례·136쪽을
새 제품과 대조한 [전체 결과](../historical-validation.md)에서
표시값 하락은 **0쪽**이다. 새 base `ece9c3383bb2c73e0f055459ede727a9d3101ac6`와
104사례·262쪽을 비교한 [Native 결과](../historical-validation.md)도
2%p 초과 하락은 없다. 후보 전쪽 최저값은 91.36%다.
새 base에 포함된 PR #8640의 실제 46상태·124쪽을 포함한다.

새 `note_separator_declared_width` 회귀는 HWP/HWPX 각각 네 복선·삼중선의 원본 선 종류,
선 굵기와 최종 논리 선의 소유·수평 관계를 검사한다. 절대 배치 좌표나 특정 시스템 글꼴에
의존하지 않는다. [최소 글꼴 실제 실행 결과](../validation-r6/runtime-checks.md)는 2 PASS·0 FAIL이며,
기존 35모듈 집중 검사 169 PASS·0 FAIL과 구분한다. [수정 전후 실제 렌더 트리](../historical-validation.md)에서
R1은 복선 2개·삼중선 3개의 별도 확대 선을 만들고, R2는 원본 폭을 소유하는 하나의 선을 만든다.

이 기록은 지적한 네 하락의 복구 근거다. 원 PR 18건 전체의 fresh WASM, 보호 문서,
전체 Rust·lint와 개별 최종 리뷰 완료를 대신하지 않는다.
