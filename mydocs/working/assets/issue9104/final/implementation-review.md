# #9104 보완의 공통 계약 검토

제품 생산자는 `1da7afbd64bbc59d9338a187ebcd21860ddc0842`, 기준은
`4f44355f5ff234abd1289d340cb4a3fe13297982`다. 최종 검증 완료 여부와 별도로 기록한다.

| 값/계약 | 생산 → 소비 → 최종 적용 |
| --- | --- |
| 가시 글줄 폭 | `paragraph_layout.rs::VisibleTextLineMeasure::new/width`가 조합된 줄과 전체 글자모양 소유를 확인 → 같은 문단의 fit/자간 계산이 이 값을 소비 → 확정된 `TextLine`에서 `fit_inherited_decoration_tail`이 남은 줄 예산에 말미 SP 전진만 배분 |
| 실제 전진 위치 | `TextRunNode.layout_positions` → `render_tree.rs::replay_positions_for`, 공통 paint의 text placement/장식 → Native text replay, Canvas replay, SVG 출력이 같은 전진 배열을 소비 |
| 실제 대체 관측 | `font_lookup`의 기존 후보 순서/선택을 보존하며 선언 후보 여부만 함께 생산 → `text_replay/fonts.rs::for_text`가 실제 선택 후보와 최종 Font의 family를 관측 → Native exporter의 명령별 snapshot → CLI의 JSON 복사본에만 병합 |
| 진단 신뢰 경계 | 요청 family는 문서 파생 데이터 → 기존 `nativeFontSupply` 전체 불신 계약과 신규 provenance 검사 → 소비자는 진단 문자열을 지시로 사용하지 않음 |

저장 HWP/HWPX 원문·LineSeg를 바꾸지 않는다. 별도 모양의 입력란, 음영/테두리,
공백 전용 줄, 탭/글자취급 개체, 원고지 격자는 이 공통 규칙에 섞지 않는다.
가시 글자의 원점·원문·장식 소유를 보존하며 Paint에서 밑줄이나 공백을 삭제하지 않는다.
분할 컷·소유 유닛·행 예약 높이·표 이월/종료·페이지 높이는 변경하지 않아 해당 분할 계약은 비해당이다.

Native 관측은 기본 비활성이며 PNG의 `--json`에서만 활성화한다. 관측 때문에 공급
`mappings`나 `FontEnvironment`를 채우거나 face를 다시 고르지 않는다. 최대 256개 항목,
이름 합계 64 KiB를 넘으면 누락 수와 `partial`을 기록한다. 본문/문자 값은 보관하지 않는다.
`complete`는 해당 범위의 관측 누락이 없다는 뜻이며 glyph 잉크/전체 문서 일치를 보증하지 않는다.
선택하지 않은 쪽과 이미지 내부 SVG/메타파일의 별도 선택은 범위 밖이다.

실행 근거:

- 실제 Native 12건: 원본 face와 명시 공급 우선, 진단 on/off PNG 동일,
  캐시 재생 중복 제거, 명령 사이 초기화, 미지원 문자/부분 관측/상한을 확인했다.
- 네트워크/외부 글꼴을 숨긴 CJK 전용 CLI: 실제 8개 대체 선택, 원본 10문단 표시,
  metadata-only의 `notRendered`와 PNG 미생성을 확인했다.
- 같은 조건의 정확한 devel: PNG는 바이트 단위로 같지만 실제 대체 진단은 없어 새 요구를 검출한다.
- 정확한 devel Native 라이브러리를 이용한 관계 검사 음성 대조: 11건 중 말미 공백의
  본문 압축 경계 1건이 실패했다. 후보의 정식 전체 회귀 결과는 별도로 기록한다.
- production layer SVG의 핵심 5개 출력도 실제로 생성했다. Native/fresh WASM
  전쪽 시각 검증을 SVG 변환으로 대신하지 않는다.

이 검토는 현재 로컬 작업의 근거다. 새 PR/원격 CI/머지 완료를 뜻하지 않는다.
