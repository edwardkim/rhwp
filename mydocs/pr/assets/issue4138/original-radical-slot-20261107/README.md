# 원본 수식의 근호 인자 슬롯 복구

부모 `b82d0843c93c70b1e296f19dd5f96a32df8aaf33`에서 원본 구형 수식에 현대 수학 face의 1em 근호 슬롯을 적용하던 오류를 보정했다. 같은 공통 공개 근호 윤곽의 실제 전진은 833/1000em이다. 현대 face의 슬롯은 유지하고, 원본 face는 이미 사용하는 공개 윤곽의 폭을 소비한다. 문서·쪽 번호 예외와 시각 문턱 변경은 없다.

## 독립 근거와 검사

원본 `saved-split-1x2.hwp`의 기존 독립 한컴2024 Print 197쪽을 그대로 기준으로 썼다. 같은 입력을 재인쇄한 별도 대조에서도 10쪽 Q·12쪽 E 원점이 기존 PDF와 같았다. 재인쇄본은 기준 PDF를 교체하지 않으며, 다른 글꼴의 경고·전쪽 시각 수용과 구분한다. 독립 자연 저장 대조는 중괄호의 실제 공간 예약을 입증하므로 그 예약을 제거하지 않았다.

검증 기록 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/original-radical-slot-20261107/validation.json`)에 입력/PDF·공개 근호 글꼴 해시, 실제 hmtx 833/1000, 재인쇄 원점과 생산자·검사 요약을 고정했다. 글꼴 바이너리·복원 표본·로그는 커밋하지 않았다. 같은 로컬 85 glyph 공급 조건이며 최소 글꼴 시각 검증으로 주장하지 않는다.

[정식 검사](../../../../../tests/cases/issue_4138_original_equation_character_ownership.rs)에 독립 PDF 12쪽 E 원점의 192DPI ±2px 고정형을 추가했다. 기존 4개 관계형·1개 고정형은 유지한다. 실제 부모 5 PASS/1 FAIL → 보정 6 PASS, 기존 집중 묶음 136 PASS(신규 6개와 출력용 진단 1개 포함)다. 합산 중복으로 검사 수를 부풀리지 않는다.

## 전197쪽 대조

| 대상 | Native 이전 → 보정 | fresh WASM 이전 → 보정 |
| --- | ---: | ---: |
| 10쪽 | 99.23962 → 99.37646% | 99.40925 → 99.54381% |
| 12쪽 | 99.02184 → 99.35295% | 99.18145 → 99.51005% |
| 전체 최저 | 98.47416 → 98.47416% | 98.50155 → 98.50155% |

양쪽 모두 197쪽·누락 0·98% 미달 0이다. 부모 b82d 대비 PNG 변경은 각 10·12쪽뿐이며 195쪽은 바이트가 같다. 한 단계 전 문자 보정의 부모 971086과 같은 85 glyph 조건으로도 페이지별 0.1%p 초과 하락은 없다. 기존 12쪽의 0.206%p 하락을 예외 처리하지 않고 복구했다.

[이전 Native TSV](before-native-silhouette.tsv) · [이전 WASM TSV](before-wasm-silhouette.tsv) · [보정 Native TSV](after-native-silhouette.tsv) · [보정 WASM TSV](after-wasm-silhouette.tsv)

Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 결과가 아니다. root pkg/Studio public 파일 일치는 생산자에서 확인했으며, 실제 브라우저 초기화 응답 검증은 최종 단계에 남아 있다.

## 직접 판독과 남은 범위

- Native 12쪽 이전: [review](before-native-p012-review.png) · [overlay](before-native-p012-overlay.png)
- Native 12쪽 보정: [review](after-native-p012-review.png) · [overlay](after-native-p012-overlay.png)
- fresh WASM 12쪽 이전: [review](before-wasm-p012-review.png) · [overlay](before-wasm-p012-overlay.png)
- fresh WASM 12쪽 보정: [review](after-wasm-p012-review.png) · [overlay](after-wasm-p012-overlay.png)

양쪽 review·overlay를 직접 열어 E 원점 복구와 앞뒤 본문·같은 쪽 다른 수식의 소속을 확인했다. 일부 수식 숫자와 본문 글리프의 미세 간격·윤곽 차이는 남으므로 100% 일치를 주장하지 않는다. fmt·Native/WASM/전체 target/Skia Clippy·workspace build·manifest·unit tier는 이 단계의 범위 검사로 기록한다. 다른 대상 문서의 98% 보정, 최종 전체 nextest·편람 전쪽·실제 소비 경로가 남아 있으며 **전체 PR 준비 완료가 아니다**.
