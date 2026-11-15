# johndoekim #9081·#9084 메인터너 검증 증적

#9081은 글상자의 숨긴 누름틀 안내문 뒤 표 줄 구성을, #9084는 값이 든 누름틀의 HWP 저장 순서를
수정한다. 두 원 기여 SHA를 merge history로 보존했다. 메인터너 보완은 회귀 주석뿐이며 제품과 assertion은 같다.

| 대상 | SHA |
| --- | --- |
| #9081 원 head | `3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738` |
| #9084 원 head | `81dd920099bae78d8d66148ddf9614785760deac` |
| 최초 검토 base | `7ee4fe0327d0cdfc8dec12c450e5dd5b5369b826` |
| 두 기여 merge 코드 | `3843f897ae410fdf67347651b39214d9dd9cafff` |
| Native·최적화 WASM 생산 | `47177a9bfc560ee52a12ae5588f6518b7950e4b9` |
| 최신 devel | `67b4ff8d3a55e5bb50c46a44315de749a4ef07a7` |
| 주석 보완 제출 코드 | `e958314dc9078fdb55e7cde04227a1a22054a72d` |

제품/Cargo/asset/fixture/실제 캡처 입력은 생산 source와 실제 검증/제출 코드 사이에 바뀌지 않았다. 검증 당시 로컬 임시 SHA와 제출 SHA의 동일성도 기록했다.
실제 빌드·프로그램 해시·글꼴 복구·실행 명령은 [검증 기록](validation-record.md)에 있다.
원시 로그·MCP 응답·중간 JSON·페이지 raster는 커밋하지 않는다.

## #9081 전쪽 비교

한/글 2022 `12.0.0.535`의 실제 저장본 2종과 Microsoft Print to PDF를 사용했다.
입력·기준 해시는 [개별 review](../../pr_9081_review.md)에 있다.
두 입력 모두 독립 PDF와 전체 1쪽이 같으며, 색상 경계 보정 전 2px `silhouette_raw_match_percent`다.
영역 제외·글꼴 예외·시각 기준 변경은 없다. Native Skia PNG와 최적화 완료 WASM Canvas를 96dpi로 비교했다.

| 입력 | base Native | base WASM | 후보 Native | 후보 WASM |
| --- | ---: | ---: | ---: | ---: |
| 안내문 있음 | [29.43](visual/base-native/guide/silhouette.tsv) | [29.43](visual/base-wasm/guide/silhouette.tsv) | [100.00](visual/candidate-native/guide/silhouette.tsv) | [100.00](visual/candidate-wasm/guide/silhouette.tsv) |
| 누름틀 없음 | [100.00](visual/base-native/no-field/silhouette.tsv) | [100.00](visual/base-wasm/no-field/silhouette.tsv) | [100.00](visual/candidate-native/no-field/silhouette.tsv) | [100.00](visual/candidate-wasm/no-field/silhouette.tsv) |

| 입력 | Native review / overlay | fresh WASM review / overlay |
| --- | --- | --- |
| 안내문 있음 | [review](visual/candidate-native/guide/review.png) / [overlay](visual/candidate-native/guide/overlay.png) | [review](visual/candidate-wasm/guide/review.png) / [overlay](visual/candidate-wasm/guide/overlay.png) |
| 누름틀 없음 | [review](visual/candidate-native/no-field/review.png) / [overlay](visual/candidate-native/no-field/overlay.png) | [review](visual/candidate-wasm/no-field/review.png) / [overlay](visual/candidate-wasm/no-field/overlay.png) |

[Print·base·Native·WASM 확대 비교](guide-table-zoom.png)에서 표의 행·열 구성과 두 표의 순서·상하 간격·
글상자 내부 포함을 직접 판독했다. base의 둘째 표 가로 이탈·누락이 해결됐으며 단순 점수만으로 판정하지 않았다.

## #9084 실제 편집·반복 저장

한 글자 값·네 글자 값·안내문 3입력에서 공개 CLI로 실제 본문을 편집하고 HWP 저장·재열기를 두 차례 실행했다.
이어 HWPX 최초 저장·재저장도 다시 열어 확인했다. 원본 raw stream을 재생해 직렬화 검사를 대신하지 않았다.

| 입력 | 원본의 FIELD_END / 표1 / 표2 | base 반복 저장 | 후보 첫·둘째 저장 |
| --- | --- | --- | --- |
| 값 한 글자 | 9 / 17 / 25 | 표 9·17 뒤에 끝 25 | 원본과 동일 |
| 값 네 글자 | 12 / 20 / 28 | 표 12·20 뒤에 끝 28 | 원본과 동일 |
| 안내문 | 9 / 17 / 25 | 원본과 동일 | 원본과 동일 |

필드 이름·값도 HWP 최초/재저장과 HWPX 최초/재저장에서 유지됐다. HWPX XML 6개는 base·후보 모두
`fieldBegin → 값 → fieldEnd → tbl → tbl` 순서다. HWPX의 기존 정상 동작을 새 개선으로 세지 않는다.
실행한 명령과 raw 토큰 대조 방법은 [검증 기록](validation-record.md)에 정리했다.

## 기존 #9058의 진단 대조군

입력과 독립 Print는 [공개 issue 댓글](https://github.com/edwardkim/rhwp/issues/9058#issuecomment-6885514057)의
원본을 사용했다. `issue9058/`에 공개 입력·PDF를 보존하며, 바이트가 같은 값 한 글자 입력은
기존 `samples/issue9082/textbox_filled_field_tac_tables.hwp`를 재사용한다.
진단 전용이며 이 낮은 출력을 정상 rendering fixture·golden·자동 회귀로 새 등록하지 않았다.

| 실제 한컴 저장본 | Native base / 후보 | WASM base / 후보 |
| --- | ---: | ---: |
| 안내문 없는 빈 필드 | 64.87 / 64.87 | 64.87 / 64.87 |
| 값 한 글자 | 57.23 / 57.23 | 57.24 / 57.24 |
| 값 네 글자 | 58.36 / 58.36 | 58.40 / 58.40 |

전체 1쪽씩의 TSV는 `visual/<base-or-candidate>-<backend>/<empty-no-guide-or-filled-one-or-filled-four>/`에 있다.
Native/WASM 각각 base·후보 PNG 3쌍은 모두 바이트 단위로 같다. 저장기 수정의 해결 범위와 구분해
#9058은 OPEN을 유지한다. 이 기존 미해결 동작을 #9081 또는 #9084가 해결했다고 기록하지 않는다.

## 공통 검사와 특별 보호

- 최소 글꼴 집중 57건: 통과. 원시 로그 대신 [실행·최종 검사 기록](validation-record.md)을 보존한다.
- 공개 공통 gate: 4문서 × Native/fresh WASM 8개 실행 모두 raw 최저 100.00%·passed.
  `contributor-gate/<document_id>/<backend>/`에 전쪽 TSV를 보존한다. private 글꼴을 가린 환경이다.
- 실제 편람 Native 383쪽 `--handbook-gate`: passed·하락 0건. [전쪽 TSV](handbook-gate/silhouette.tsv).
- 구형 수식 글꼴은 독립 Print로 복구해 원본 77개 프로그램 해시가 기존과 같음을 확인했다.
  사설 프로그램과 문자표 전체 PDF는 커밋하지 않는다. 로컬 설치 위치·실제 새 컨테이너 해시는 실행 기록에 있다.
- [특별 보호 전쪽 결과](protected-validation.md): 18문서 × Native/fresh WASM 전체 3,542쪽, 공식값 하락·누락 0건. 예외 없이 통과했다.
- 최종 전체 회귀 13,614 passed·0 failed (기존 ignored 45개), Native 공식 4,073 passed·0 failed, 모든 lint·workspace build·suite manifest 검사 통과.

## 원격 처리

[처리안](remote-action-plan.md)은 승인 후 실행할 범위를 구체화한 문서다. 현재 원격 push·새 PR 생성·공개 리뷰·
댓글·머지는 수행하지 않았다. 필수 로컬 검증을 완료했으며 새 후보의 원격 승인이 필요하다.
