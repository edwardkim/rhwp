# PR #9075 검토 증적

> 새 base `f8234f6f90` 반영 뒤 재검증 중이다. 아래 수치는 리베이스 전 source43의 완료 기록이며 새 head의 통과로 대신하지 않는다. [리베이스 보고서](rebase-f823-analysis.md)를 따른다.

[최종 대상 문서 증적](final/main/README.md)과 [검증 기록](verification-record.md)을 정본으로 본다.
제품 소스는 `43adf3f7fe412e71797456887e2be44b8ef8d14f`이며 기존 독립 한컴 Print를
재인쇄하거나 원본을 바꾸지 않고 Native·fresh WASM으로 대조했다.

- 원 PR의 공개 86036 입력은 두 경로 모두 전 14쪽 공식 raw 100.00%다.
- 첫 줄은 5쪽, 나머지 두 줄은 6쪽, 후속 문단은 7쪽임을 실제 내용과 독립 PDF에서 확인했다.
- [Native 전쪽 TSV](final/main/native_silhouette.tsv)·[fresh WASM 전쪽 TSV](final/main/wasm_silhouette.tsv).
- [5·6쪽 Native/fresh WASM review와 overlay](final/main/README.md)는 PR 주장과 같은 입력·쪽이다.
- [보호 전쪽 TSV](final/protected/README.md): 두 경로 총 36개 데이터셋·3,542쪽 완료, 하락 0쪽·상승 30쪽. 실제 편람 gate도 통과했다.
- 기본 Rust 13,562건, Native 3종, 필수 lint와 workspace 검사까지 모두 통과했다. 실제 명령·결과·SHA는 검증 기록을 따른다.

루트의 기존 Native/WASM TSV·5·6쪽 PNG는 원 PR 자료를 바탕으로 수행한 초기 글꼴 공급
대조 기록이다. 제품 보정 뒤의 최종 제출 증거와 혼동하지 않는다. 나머지 단계 문서는
당시 원인 분석과 수정 근거이며, 현재 미완료 항목의 최종 판정은 위 검증 기록에서 확인한다.

원시 로그·중간 run manifest·분석 JSON·중간 raster는 ignored output에서만 관리한다.
최종 대표 PNG·전체 TSV와 명시적 글꼴 환경, 정식 보호 baseline의 수용 조건만 제출한다.
글꼴 프로그램을 포함하지 않으며 100.00%를 RGB 동일성으로 주장하지 않는다.
