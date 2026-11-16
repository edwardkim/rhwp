# 원본 HFT en dash의 전진폭

중첩 표 보호 입력의 16쪽에서 두 en dash 뒤 본문이 앞당겨진다. 원본 바이트의
글자모양 5는 1500 HWPUNIT, 기호 슬롯 5의 글꼴 ID 7인 한양신명조 HFT다.
독립 한컴 Print PDF의 T6 ToUnicode는 07을 U+2013으로 연결하고, /Widths[7] 및
/HFT376 CharProc의 d1은 1000이다. FontMatrix는 .001이다. 따라서 원본 전진은
1em이며, 현재 누락된 메트릭의 반각 추정으로 재지 않는다.

- 원본: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`,
  SHA-256 `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626`.
- 독립 PDF: `pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf`,
  SHA-256 `bb486ef1215e79c2ca0c891b2b194bdf988691b0e91b8a07ebdbffc6d749473b`.
- 전진 근거: T6 xref36 → Widths xref549 `[1000 1000 500 1000 1000 1000 1000 1000]`,
  ToUnicode xref550 `<07> <2013>`, HFT376 xref488 `1000 0 ... d1`.
- 같은 PDF의 구분점 U+2027은 0.5em이다. 이를 en dash와 같은 폭으로 합치지 않는다.

선행 PR #8315의 `a9febd969c`는 기호 슬롯 글꼴의 실제 폭을 보존한다. 그 정상 대조군인
궁서 TrueType en dash는 약 0.5em이며 그대로 유지한다. `font_metric_legacy_hft.rs`의
기존 U+2022·U+221A 보충과 같은 원본 프로그램 경계에 U+2013만 추가한다.

소비 경로는 원본 HFT 메트릭 → `char_width_decision` → 공통 문자 위치 → 실제 run의
charX·bbox와 뒤 글자의 원점이다. backend별 좌표·문서 ID·쪽 번호에 따른 예외나
그림 숨김을 추가하지 않는다. 표시 glyph의 fallback과 원본 전진의 소유는 구분한다.

보정 전 최소 글꼴 환경의 공개 측정 API 반례는 13.333333333333334px em에 대해
6.666666666666667px로 전진해 FAIL(exit 101)했다. 해당 값은 원본 비율을 입증하기
위한 실행 관측이며 실제 문서의 절대 좌표 핀이 아니다. 새 소스의 실제 전쪽 Native/
fresh WASM 검증과 기존 기호 대조군은 보정 후 함께 확인한다.

## Native 단계의 실제 확인

[최소 글꼴 Native 실행](verification-record.md)은
신규 관계형 검사·기존 겹침과 기호 대조군·원 PR 검사·Native 필수 3종 및 Clippy를 통과했다.
새 PUA 검사의 전쪽 문자열 개수 단언은 제목 외 본문에도 같은 문구가 있는 의미 오류여서
해당 제목 run의 원문 보존으로 바로잡았으며, 서식·크기·인접 원점·명시적0 대조는 유지했다.

[현재 Native 전17쪽의 검증 요약](verification-record.md)은 기존 보호값 대비 하락0쪽이다.
12쪽100.00%, 16쪽99.96%이며 원 PR 입력 전14쪽도100.00%를 유지했다.
[12쪽 review](verification-record.md)·[overlay](verification-record.md),
[16쪽 review](verification-record.md)·[overlay](verification-record.md)를
직접 판독했다. 원본 전진이 복구되어도16쪽의 좁은 대체 dash 자체는 원본 전각 칸에서 왼쪽에
치우쳐 있어, 뒤의 공통 glyph 맞춤 단계에서 보정한다. fresh WASM·보호 전수·최종 통합이
완료되었다는 판정은 아니다. 기준 PDF·원본·마스크·판정값과 보호 하한은 그대로다.
