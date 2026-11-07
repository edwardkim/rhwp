# 원본 병합 칸의 종료 줄 슬롯과 물리 하단 보존

부모 `16a99fdfaf`, base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`의 위임전결규정 종료 조각을 보정했다. 전103쪽 모두 Native/fresh WASM98% 이상이 됐다. 다른 문서·전체 회귀·PR 준비 완료를 뜻하지 않는다.

## 원본과 독립 기준

- `samples/issue5679/10857_delegation_rules.hwp`, SHA-256 `de43607f05072840097b19a294b4455905210402cc42823545125e03567df881`.
- 독립 한컴2020 Print `pdf/semanticist21-20261008/issue5679-10857_delegation_rules-de43607f0507-2020.pdf`, 전103쪽, SHA-256 `90eda86b2b4dfeabc601cd9b583408c6528a9efabf4e626fd6f2e28e3bcb2b97`.
- 정본 `visual_sweep.py`, PDFium96DPI·compat2022·색상 경계 보정 전2px raw·capture 총8. 실제 로컬 글꼴 공급 조건이며 의미 검사의 macOS 최소 글꼴 sandbox와 구분한다.

원본 병합 칸은 같은 문단의 다음 줄이 vpos0에서 재시작하며, 앞 줄은1000+300HU 슬롯과567+567HU 안여백을 소유한다. 수용한32.45333px에서 줄간격4px를 회수하여 물리 상자를28.45333px로 만들던 차이를 확인했다. 독립 PDF의 하단 벡터는96DPI1023.68px다.

공통 소유 판정을 수용·물리 높이·마지막 paint 접기에 함께 사용한다. 블록 차액은 실제 내용을 그리는 마지막 양수 행에 배정하고 미수용0높이 행을 새 빈 행으로 만들지 않는다. 쪽 상한 초과분은 기존 예산에서 별도로 처리한다. 원본 슬롯이 없는 편집/재조판·위 정렬·문단 종료에는 이 인증을 확장하지 않는다. 문서/쪽 분기·픽셀 가산·검사 제거·문턱 완화는 없다.

첫 후보의 빈 행 배정 실패와 둘째 후보의 paint 재차 회수 실패를 보존하고 구현을 보정했다. 공통 helper의 공개 범위 경고가 난 중간 빌드는 종료하고 최종 산출물을 새로 만들었다. 중간 후보는 통과 증거가 아니다. 빌드 뒤 rustfmt만 달라진 한 본문은 빌드 hash와 동일한 포맷 전 소스를 복원하여 rustfmt 출력이 최종 소스와 바이트 같음을 확인했다.

## 실제 검증

| 범위 | 이전 Native → 보정 | 이전 fresh WASM → 보정 |
| --- | ---: | ---: |
| 26쪽 | 96.65328 → 99.98249% | 96.65155 → 99.98249% |
| 전103쪽 최저 | 96.65328 → 99.21307% | 96.65155 → 99.21221% |

각103쪽·98% 미달0·누락0이다. 변경 PNG는26쪽 하나이며 나머지102쪽은 각 backend에서 바이트 동일하다. 직접 부모 및 고정 devel 대비0.1%p 초과 하락도 없다. 27쪽의 기존 미세한 수직 위치 차이는 그대로이며, raw100%나 RGB 전체 동일로 보고하지 않는다.

[이전 Native TSV](before-native-silhouette.tsv) · [이전 WASM TSV](before-wasm-silhouette.tsv) · [보정 Native TSV](after-native-silhouette.tsv) · [보정 WASM TSV](after-wasm-silhouette.tsv)

[회귀 검사](../../../../../tests/cases/issue_7700_source_owned_table_fragments.rs)는 기존9개를 유지하고 원본 슬롯·일반 peer의 같은 하단·앞뒤 줄의 단일 소유 관계와 독립 Print 하단±2px를 추가했다. 26쪽 양 backend98% 이상과 직접 판독 뒤 정식 등록했다. 실제9 PASS/2 FAIL→11 PASS이며 집중 묶음162 PASS에는 이11개와 기존 출력 진단1개가 포함된다. 기대값을 낮추지 않았다.

fmt·Native/WASM/전체 target/Skia Clippy·workspace build·manifest·unit tier를 통과했다. Native Skia 전체 unit·전체 nextest의 최종 head 통과로 확대하지 않는다. fresh WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 결과가 아니다. root pkg/Studio public hash는 같지만 실제 브라우저 초기화 응답 확인은 최종 단계에 남아 있다. 해시·명령·요약은 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/reset-span-slot-20261107/validation.json`), 로그와 임시 도구는 ignored output에만 보존한다.

## 대표 그림

- 이전 Native26: [review](before-native-p026-review.png) · [overlay](before-native-p026-overlay.png)
- 보정 Native26: [review](after-native-p026-review.png) · [overlay](after-native-p026-overlay.png)
- 이전 fresh WASM26: [review](before-wasm-p026-review.png) · [overlay](before-wasm-p026-overlay.png)
- 보정 fresh WASM26: [review](after-wasm-p026-review.png) · [overlay](after-wasm-p026-overlay.png)
- 다음 Native27: [review](after-native-p027-review.png) · [overlay](after-native-p027-overlay.png)
- 다음 fresh WASM27: [review](after-wasm-p027-review.png) · [overlay](after-wasm-p027-overlay.png)

직접 판독으로 마지막 일반 칸/병합 칸 괘선·`통계관리`/다음 쪽 `분석`의 소속·기존 행과 반복 제목을 확인했다. 다음 쪽의 기존 미세 차이는 별개로 남긴다.
