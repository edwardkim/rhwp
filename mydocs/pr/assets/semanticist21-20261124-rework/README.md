# semanticist21의 보완 답변 3건 재검토

메인테이너가 재제출본을 보완한 결과는 [최종 검증 기록](final/README.md)에 있다. 아래 표는 기여자가 처음 재제출한 세 커밋을 검토했을 당시의 측정값이다.

2026-11-24에 이전 보류 요청 뒤 답변과 새 커밋을 모두 제출한 3건을 확인했다. 3건 모두 `edwardkim`이 리뷰어다. 원 PR마다 최신 CI는 성공 35건, 조건부 생략 6건이고 `devel`과의 개별 병합 시뮬레이션은 충돌 없이 끝났다. [원격 상태와 검사](github-checks.json)

| PR | 검증한 원 head | 입력과 쪽 수(backend별) | 판정 |
| --- | --- | --- | --- |
| [#9281](../../archives/pr_9281_review.md) | `f528069f3b` | 8개·32쪽 | 기존 두 구역 원본은 개선됐다. 글자가 끼면 첫 구역의 용지와 쪽 번호가 잘못된다. |
| [#9286](../../archives/pr_9286_review.md) | `65ca6f4128` | 11개·11쪽 | 타원 배치는 개선됐다. v0.9.1이 저장한 원본 표지 없는 HWPX의 투명 채움이 깨지며, Native에서 일부 획이 사라진다. |
| [#9325](../../archives/pr_9325_review.md) | `6f4ded68c7` | 12개·12쪽 | 요청한 인쇄 8사례와 편집 API가 통과했다. Studio에서 가운데 손잡이를 끌면 여전히 개체 크기가 바뀐다. |

원 기여 head를 최신 `devel`의 `626a5b30ab`에 순서대로 병합한 로컬 소스 `327be655a1`로 Native Skia와 최적화한 WASM을 빌드했다. Chromium이 실제 읽은 WASM의 SHA-256은 `0b69eb561b47f67269d2bc46358485aa53d6ff24014e05986a98ae51d6840239`다. 기준 비교에는 `devel`과 제품 소스·빌드 입력이 같은 `c1504cb121`의 산출물을 사용했다. 공개 글꼴 5개만 노출한 환경에서 세 변경 모듈의 Rust 집중 검사 23개가 모두 통과했다. 각 PR의 보류 판정은 아래 실제 원본·조작 반례에 근거한다. 최종 병합용 보호 문서 전체 검증은 아직 실시하지 않았다. [소스·빌드 식별값](validation.json) · [공개 글꼴·집중 검증](focused-tests.json)

한컴으로 인쇄한 31개 원본·PDF를 같은 96 DPI로 렌더링했다. Native와 fresh WASM은 각각 55쪽이다. 쪽별 표의 raw 값은 색상 경계 보정 전 2px 실루엣 일치율이다. 원본·PDF 경로와 해시, 실행한 한컴 빌드와 인쇄 job, 전체 쪽 수는 입력 목록에 있다. PNG 해시와 출력 크기는 쪽별 표에서 확인할 수 있다. [입력 목록](inputs.tsv) · [전체 쪽별 TSV](pages.tsv)

Print 작업의 성공 상태, 실제 엔진·빌드, 한 쪽 인쇄 설정(`pdf_print_method=0`), 다운로드 해시와 PDF의 Hancom PDF Producer를 대조했다. 저장소 영수증 감사 도구에서도 원본·PDF 해시는 31쌍 모두 일치했다. 서비스가 반환한 전처리 필드는 `input_preprocess`이고 2024의 출력 모드는 `hancom2024_pdf_driver_one_up`이다. 현재 감사 도구는 `preprocess` 필드와 2024의 다른 모드명만 인정해 31쌍을 `candidate`로 표시한다. 수신한 필드와 이 판정을 그대로 보존했다. [실제 Print 영수증](print-receipts.json) · [감사 결과](print-audit-summary.json)
