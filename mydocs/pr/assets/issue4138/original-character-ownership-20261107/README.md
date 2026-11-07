# 원본 수식의 문자·논리 전진 소유 보정

부모 `971086b752f9d5e869bdfcc18ff992541bf8276d`, base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서 원본 수식 표시의 문자 소유·논리 전진을 보정했다. 문서 ID·쪽 번호로 분기하지 않는다. 원본 전 197쪽은 Native/fresh WASM 모두 98% 이상이다. 다만 부모 대비 12쪽이 각각 0.20579/0.20636%p 하락하여 추가 보정 중이며, **PR 제출은 보류한다**.

## 입력과 실행 조건

- [분할 저장 원본](../../issue7445/completed-20261009/regression-policy-20261015/issue4138-current-production/saved-split-1x2.hwp), SHA-256 `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`.
- [독립 한컴2024 Print PDF](../../../../../pdf/issue4138-ascii-split-20261016-2024.pdf), 197쪽. 파일 해시는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/original-character-ownership-20261107/validation.json`)에 고정했다.
- 현재 정본 `visual_sweep.py`, 192DPI, compat2024, PDFium, 2px 내용 실루엣, capture 총 8. 양쪽 모두 197쪽 완료, 누락 0.
- 원본 glyph 85개를 확인한 로컬 공급 글꼴 프로필이다. 최소 글꼴 환경 통과를 주장하지 않으며 글꼴 바이너리·복원 표본·raw 프로그램은 제출하지 않는다.
- fresh WASM은 Mac locked wrapper의 `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 빌드 결과가 아니다. root pkg/Studio public의 JS/WASM 파일 일치는 빌드 생산자 기록으로 확인했다. 실제 브라우저 초기화 응답 검증은 남아 있다.

## 공통 원인과 소비 경로

일반 괄호를 현대 문법의 구조 괄호로 묶어 원본의 무브레이스 첨자 소유가 바뀌었다. 구형 표시 세대의 일반 괄호는 CHAR로 보존하며 명시적 LEFT/RIGHT와 현대 공개 문법은 구분한다. 원본의 기본 이탤릭 논리 전진은 표시 Regular Type3의 d1과 다르므로, 독립 정상 저장의 ASCII 260개·비정수 크기 56개 전진으로 확인한 논리 face를 사용한다. 그룹·첨자 여백은 자식의 축소 표시 크기가 아니라 root 논리 단위를 소비한다.

독립 계약 감사 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/original-character-ownership-20261107/independent-contract-audit.json`)는 초기 260조건에서 구분되지 않던 절삭/반올림을 별도 56조건으로 구분하고, root 그룹 관계 15개를 대조한다.

소수점은 원본 d1·공급 글꼴 hmtx 모두 500/1000em이지만 Number role에서 빠져 `0.5`·`0.4` 전체가 일반 수학 글꼴로 대체됐다. 같은 원본 run으로 복구했다. 두 논리 글줄의 LEFT/RIGHT는 원본과 같은 괄호 프로그램의 Tf/Tz를 FontGlyph에 보존하고, 분자·분모의 논리 기준선과 자연 글리프 기준선을 구분한다. 공통 LayoutBox의 글꼴·좌표·크기를 SVG/Native/WASM이 소비한다. 현대 face·일반 문자 괄호·미확인 중첩 프레임에는 이 두 줄 구분자 분기를 적용하지 않는다.

## 전쪽 결과

| 조건 | Native 최저 | fresh WASM 최저 | 98% 미달 |
| --- | ---: | ---: | ---: |
| 부모, 같은 85 glyph 공급 | 95.74902% | 95.80661% | 각 2쪽 |
| 후보, 같은 공급 | 98.47416% | 98.50155% | 각 0쪽 |

[부모 Native TSV](before-native-silhouette.tsv) · [부모 WASM TSV](before-wasm-silhouette.tsv) · [후보 Native TSV](after-native-silhouette.tsv) · [후보 WASM TSV](after-wasm-silhouette.tsv)

부모 대비 수치 변경은 양쪽 모두 10·12·40·44·45·129·130쪽이다. 12쪽의 0.206%p 하락은 허용 완료로 처리하지 않는다. 독립 16조건에서 근호 인자의 중괄호가 실제 논리 공간을 예약함을 확인했으므로 그 예약을 삭제하지 않고, 근호 표시 폭과 호스트 원점을 추가 대조한다. 나머지 대상 문서의 98% 보정·최종 전체 nextest·편람 전쪽·실제 소비 경로도 아직 남아 있다.

## 회귀 검사와 집중 게이트

[신규 검사 5개](../../../../../tests/cases/issue_4138_original_equation_character_ownership.rs)는 관계형 4개와 129쪽 첫 기저 괄호 원점의 고정형 1개다. 고정형은 후보 PNG가 아니라 독립 PDF의 Type3 원점을 192DPI ±2px로 비교한다. 남은 뒤 기저의 미세 간격은 잘못된 고정 기대값으로 등록하지 않는다.

실제 이전 코드 1 PASS/4 FAIL → 현재 5 PASS다. 소수점 직전 후보는 3 PASS/2 FAIL이다. 기존 집중 묶음 130 PASS(출력용 진단 1개 포함), 별도 기존 소비 검사 22 PASS, SVG 소비 검사 31 PASS를 확인했다. 묶음의 중복 여부를 무시한 전체 검사 수 합산은 하지 않는다. fmt·Native/WASM Clippy·workspace build·all-target Clippy·native-skia Clippy가 통과했다. fmt 후 준비 해시를 다시 생성하여 manifest와 unit tier도 통과했다. 최종 전체 nextest 통과로 대신 인용하지 않는다.

자연 저장 합성 대조군은 문자·메트릭 소유의 독립 원인 확인 자료다. 일부 합성쪽의 PNG는 미달이므로 전쪽 정답 fixture 또는 100% 시각 통과로 등록하지 않았다. 로그·임시 스크립트는 ignored output에만 보존한다.

## 대표 실제 출력

- 129쪽 이전: [review](before-native-p129-review.png) · [overlay](before-native-p129-overlay.png)
- 129쪽 후보 Native: [review](after-native-p129-review.png) · [overlay](after-native-p129-overlay.png)
- 129쪽 후보 fresh WASM: [review](after-wasm-p129-review.png) · [overlay](after-wasm-p129-overlay.png)
- 130쪽 이전: [review](before-native-p130-review.png) · [overlay](before-native-p130-overlay.png)
- 130쪽 후보 Native: [review](after-native-p130-review.png) · [overlay](after-native-p130-overlay.png)
- 130쪽 후보 fresh WASM: [review](after-wasm-p130-review.png) · [overlay](after-wasm-p130-overlay.png)

## 후속 보정

위 수치는 이 단계의 이력으로 보존한다. 12쪽 하락은 다음 [근호 슬롯 보정](../original-radical-slot-20261107/README.md)에서 원인을 수정하고 전197쪽을 다시 검증했다. 이 단계의 과거 결과를 새 코드의 실행 결과로 재인용하지 않는다.
