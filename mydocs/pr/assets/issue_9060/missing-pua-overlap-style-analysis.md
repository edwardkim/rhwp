# PUA 글자겹침의 누락 서식과 제어 슬롯 소유

중첩 표 보호 입력 12쪽 제목의 원본 PARA_CHAR_SHAPE는 `start=0, id=106`이다.
글자모양106은 한양중고딕13pt이며, 원본 PARA_TEXT는 0x17 `tcps` 제어 슬롯 뒤에
공백과 ‘중앙선거관리위원회’를 둔다. CTRL_HEADER의 ID 뒤 전체 payload는
`02 00 80 db b3 de`다. UTF-16의 두 유닛은 U+F02B3 한 글자이고 별도 글자모양
개수·ID는 없다. IR의 `CharOverlap.char_shape_ids`도 비어 있다.

- 원본: `samples/basic/issue2007_nested_cell_pagination_42065.hwp`,
  SHA-256 `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626`.
- 기준: `pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf`,
  SHA-256 `bb486ef1215e79c2ca0c891b2b194bdf988691b0e91b8a07ebdbffc6d749473b`.
- 독립 PDF의 해당 번호 F8과 제목 T14는 모두108 인쇄 장치 단위의 Tf다.
  같은 변환에서 번호의 Tm x503 뒤 제목 제어 슬롯 x611로108단위 전진한다.
  제목 본문은 공백 뒤 x662에서 시작한다. 번호는 같은 13pt 전각 칸을 소유한다.

현재 `inline_control_projection.inc.rs`는 PUA를 일반 Composition으로 펼치지 않는
경로에서 비어 있는 내부 ID를 0번으로 치환한다. 그 결과 문서 기본 10pt가 제어
한 칸을 소유하고 제목이 약4px 앞당겨진다. 일반 문자 겹침의 Composition 경로는
이미 원문 host를 구분하므로, 누락 fallback도 같은 host ID를 소비하게 한다.
명시적인 0번이나 유효한 저장 ID는 계속 보존한다.

소비 경로는 원본 제어 슬롯의 UTF-16 위치 → host_style_id → 삽입 ComposedTextRun의
char_style_id → 공통 한 칸 전진 → 저장 줄 재생·재조판·실제 paint다. 이후 backend에서
제목 x를 옮기지 않는다. 원문·내부 문자 배열·명시적 서식 ID도 바꾸지 않는다.
선행 기능은 PR #8019(`2fb1103d81`)의 일반 겹침 Composition과 PUA 별도 해독 경계다.
문서별 예외 대신 비어 있는 내부 참조와 명시적0번 참조를 나누는 반례로 검증한다.

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
