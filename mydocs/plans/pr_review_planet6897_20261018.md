# planet6897 열린 PR 11개 체리픽 통합 계획 — 2026-10-18

## 요청·기준·작업 경로

사용자는 열린 PR들의 **체리픽 머지와 후속 처리**, 리뷰어 지정 선행과 필요한 메인터너 보정을 명시했다.
원 PR을 각각 직접 merge하지 않고 최신 devel 위 한 브랜치에 기능 commit을 누적 적용한다.

- 기준 devel: `8766fb47d8ec11547f0d2ae941c53cc3c9780ab2`.
- 작업 브랜치: `review/planet6897-20261018`, 기본 작업공간 `/home/tsjang/rhwp`.
- 11개 모두 non-draft. 먼저 `jangster77` reviewer 지정을 완료했고, 원 author·공동 저자·fork와 `-x` 출처를 보존한다.
- 원 PR source 47 commit의 Git cherry 검사에서 기준 devel과 patch-equivalent인 commit은 없었다. PR 안의 merge commit도 없었다.
- 기본 경로: collaborator_external_pr 9.1.1. 보조: intake/local_validation, multi_pr_update_branch, visual_fixture_evidence, renderer backend, 대형 PR(#7965), CI 후 review-only trailing, post_merge.

## 적용 순서

| 원 PR | 고정 source SHA | commit 수 | 변경 |
| --- | --- | ---: | --- |
| [#7934](https://github.com/edwardkim/rhwp/pull/7934) | `a60c275e8abbce2c6cb9e4277422b7084b13e830` | 4 | fix(layout): 병합 격자 표 열 경계를 저장 칸 폭 누적으로 푼다 (#7933) |
| [#7956](https://github.com/edwardkim/rhwp/pull/7956) | `dfafbbbf50c9c2cf778174aa3c7c208e99e91c8b` | 9 | fix(조판): #7840 저장 프레임 되감김 오판·이어받기·꼬리말 TAC 줄 상자 — 148738070·1150200·156465345 쪽수 PDF 일치 |
| [#7965](https://github.com/edwardkim/rhwp/pull/7965) | `f95faf3a57cccc1304ab253fa8db40b3e8442ea4` | 20 | fix(조판): RowBreak 표 분할·이어받기를 저장 쪽 경계 기록대로 — 1342000 쪽 수 415 일치·21761835·1480000·1613000 (부분 개선) |
| [#7967](https://github.com/edwardkim/rhwp/pull/7967) | `2004d296003b0a75c823a00c4ac09486ed672385` | 4 | fix(renderer): 문단 테두리 묶음 아래 변에 2px 기본 여백을 더하지 않는다 (#7888) |
| [#7968](https://github.com/edwardkim/rhwp/pull/7968) | `4fde7ae18c1c5ded4ed7c645e72db682ce58b088` | 4 | fix(text): KoPubWorld 계열을 KoPub 전진폭 표로 재고 바탕체 한글을 936/1000 으로 둔다 (#7702) |
| [#7973](https://github.com/edwardkim/rhwp/pull/7973) | `6df857e2c1b164a6016f6ea50f0299da6bbe5587` | 1 | fix(renderer): 낡은 저장 host 줄이 자란 중첩 TAC 표를 못 담으면 초과분·재조판 줄간격을 측정과 배치가 함께 센다 (#7872) |
| [#7975](https://github.com/edwardkim/rhwp/pull/7975) | `dbdbdd8b3a7b7510e047be23979b8366b84536fc` | 1 | fix(equation): rm·it 는 수식 끝까지 지속되는 글꼴 상태이고 뒤 공백 명령은 간격이다 (#7873) |
| [#7976](https://github.com/edwardkim/rhwp/pull/7976) | `7f85ab9b0198edbd68d848e43b370831407c33bd` | 1 | fix(equation): 수식 스크립트의 +-·-+ 축약을 ±·∓ 명령 토큰으로 읽는다 (#7865) |
| [#7979](https://github.com/edwardkim/rhwp/pull/7979) | `7377b745bce39aa28dca444b1df1b8c90a4598a2` | 1 | fix(search): 찾아 바꾸기가 새 글자에 지운 글자의 글자 모양을 자리대로 잇는다 (#7866) |
| [#7982](https://github.com/edwardkim/rhwp/pull/7982) | `7ffd13b64aa90110070d5c47eb6043b1d83af4df` | 1 | fix(layout): 자리차지 표 항목이 조판기가 독립 항목으로 내보낸 같은 문단의 TAC 표를 다시 그리지 않는다 (#7330) |
| [#7987](https://github.com/edwardkim/rhwp/pull/7987) | `fe5d9ebd5260eca29c23e913d8b81cc1c209ccac` | 1 | fix(equation): 전부 대문자로 쓴 그리스 문자 이름을 대문자 그리스 문자로 읽는다 (#7390) |

## 프로젝트 적합성과 구현 검토

표·문단/수식/글꼴·편집의 한컴 호환 개선이라는 프로젝트 범위에 맞는지를 주장별로 검토한다.
단순 원문 존재·쪽수·종전 CI를 시각 일치나 측정/배치 계약의 증거로 바꾸어 쓰지 않는다.
현재 devel의 Unicode source-span 검색/치환, PDF 입력·paint 및 공개 WASM API 변경을 충돌 해결에서도 보존한다.

- #7934: 저장 칸 폭의 차분 제약과 비적용 반례, 측정/배치의 실제 폭 소비자를 추적한다. 합성 1쪽 통과로 실문서 미달을 면제하지 않는다.
- #7956/#7965: 저장 프레임·물리 높이·여백·컷/이월·최종 배치까지 실제 일반/특수 경로를 확인한다. 작성자가 명시한 기존/새 하락을 별도 확인한다.
- #7967/#7968: 문단 테두리와 폰트 메트릭의 독립 근거를 확인한다. 온새미로의 빈 쪽/폰트·저장 줄 문제와 편람의 최신 보정을 함께 살핀다.
- #7973: 변형 fixture와 실제 제보 원본을 구분한다. 저장 정보 유효성·낡은 host 줄 간격·둘째 표 내용 보존과 뒤 내용까지 확인한다.
- #7975/#7976/#7987: 독립 한컴 수식의 명령·글꼴 상태·공백·기호·분수/첨자 경계를 확대 대조한다. 수정 구현을 재인용한 기대값이나 미달 문서의 golden 잠금을 수용하지 않는다.
- #7979: 한컴 실제 치환 쌍의 모양 상속과 Unicode source-span을 본문/칸, 실제 편집·저장·재열기에서 함께 확인한다.
- #7982: 독립 PageItem 소유와 paint 한 번 계약을 실제 문단/구역 경계에서도 대조한다. 노드 수뿐 아니라 순서·뒤 내용·겹침을 검증한다.

## 현재 알려진 검증 주의점

원 PR 본문은 #7965의 다수 90% 미달, #7956의 55025 p5 하락, #7968의 새 미달/하락,
#7976의 fixture 점수 하락, #7975의 합성 증적 51~61%, #7967의 온새미로 47/46쪽 등을 명시했다.
이는 **작성자 기록이며 이 서버 최신 통합 head의 실행 결과가 아니다**. 전체를 체리픽한 결과로 다시 확인하고 필요한 범위의 메인터너 보정을 별도 commit에 담는다.
대표 쪽 상승·기존 이슈·평균·CI 녹색을 90% gate나 devel 대비 0.01%p 하락 조건의 면제로 사용하지 않는다.

## 실행·검증 순서

1. 기준 Native Skia CLI와 fresh WASM을 기본 작업공간의 같은 고정 공용 target에서 순차 빌드하고 producer를 ignored output에 보존한다. 누적 체리픽과 보정은 사용자가 그래프를 볼 수 있는 주 작업공간에서 진행한다. 초기 임시 integration-source worktree는 기준 빌드 후 제거했고 현재 별도 worktree는 없다. 모든 Cargo 작업은 공용 target에서 순차 실행한다.
2. 위 순서의 기능·증적 commit을 -x 누적 체리픽한다. 충돌·적용 SHA와 원 기여/메인터너 변경을 구분한다.
3. 원본/PDF는 기존 저장본을 먼저 찾고 SHA·저장 제품·Print 경로를 확인한다. 비공개 원본은 korea_downloads에서 찾는다. 50MiB 이상 PDF는 local-only 보존 계약을 따른다. 글꼴은 실제 시스템 등록/명시 경로를 공급하며 복사·이름 변경·배포하지 않는다.
4. 고정 devel과 통합 후보의 전쪽 Native/fresh WASM raw TSV를 같은 입력/PDF/글꼴/profile/compat/DPI로 산출한다. 작은 경계와 대표/미달 PNG 직접 판독을 먼저 하고 필요한 보정 뒤 갱신한다.
5. 각 페이지 raw 90%·쪽수/내용 보존과 devel 대비 0.01%p 이상 하락 0을 확인한다. 실제 결함과 부족한 증거는 완료로 쓰지 않는다. 통과한 범위에서만 관계형 회귀를 정비하고, 완료 98% 이상 페이지의 핵심 경계에 독립 ±2px 보호를 높은 점수 순으로 추가한다.
6. 필수 lint·workspace·focused·전체 nextest·Native Skia 3종·fresh WASM·해당 편집/브라우저 검증을 최종 source에서 순차 완료한다. 새 fixture의 7개 래칫·보안 입력 계약을 확인하고 CI 개별 호출을 추가하지 않는다.
7. 준비된 후보를 upstream 임시 head의 통합 PR로 제출한다. 정확한 code CI가 성공한 뒤에만 원 PR별 archive 리뷰/구현 기록과 오늘할일을 같은 PR trailing 문서로 추가하고 merge-tree/링크/기존 기록 보존을 확인한다.
8. trailing CI·실제 fast-pass·최신 head·MERGEABLE/CLEAN을 확인해 정상 merge한다. source PR별 검증·해결 범위만 안내하고 실제 완료한 원 PR을 통합 완료로 close한다.
9. duration 결과·devel sync·이슈 종료/부분 해결 구분·댓글을 확인한 뒤 이번 소유 임시 branch/ref와 사용 종료한 root output 전체를 정리한다. 원 fork·기본 작업공간·공유 target/pr-review·golden SVG는 보존한다. 추가 문서 PR을 만들지 않는다.

작업 로그와 중간 JSON/TSV는 `output/pr-review/planet6897-20261018/`에 보존한다. 최종 TSV·대표 PNG·입력/producer 근거는 정식 증적 경로와 PR 본문에 연결한다.

## 작업지시자 지적 영역 — 추가 보정 필수

사용자가 아래 원 PR 이미지를 직접 지정하고 추가 메인터너 보정을 요청했다. 같은 immutable SHA의 Git blob에서 이미지를 꺼내 확인했다. 이 이미지는 **원 PR의 과거 캡처**이며 최신 통합 head의 증거로 재사용하지 않는다.

| 원 PR·페이지 | 사용자 지정 이미지 | 직접 관찰과 이후 확인 |
| --- | --- | --- |
| #7965 / 1342000 p202 | `devel_table_scan_regression/1342000-p202-overlay-wasm.png`, source f95faf3a5 | 마지막 행의 글줄 겹침·가로 경계 차이. 해당 셀의 컷/예약·최종 행 높이·줄 소속·내용 보존을 독립 PDF와 대조한다. |
| #7982 / 야간방호일지 p1 | `float-sibling-tac-7330/36374873-p001-after.png`, source 7ffd13b64 | 표 뒤 붙임 문단이 PDF보다 아래. 중복 Table 제거에 더해 뒤 문단의 저장 줄·빈 줄·표 점유 끝과 실제 원점을 확인한다. |
| #7934 / 80168 p17 | `table_merged_grid_column_boundaries/80168_p017_overlay_after.png`, source a60c275e8 | 아래쪽 행 높이와 가로 경계 어긋남. 열 경계 외에 중첩 표/텍스트 점유·바깥 행 높이까지 확인한다. |

해당 페이지의 기존 점수가 높아도 지적 영역은 완료로 판정하지 않는다. 통합 후 실제 Native/fresh WASM의 같은 페이지·확대 영역을 다시 출력하고 원인 보정 전후를 비교한다.

## 누적 적용 완료와 1차 메인터너 보정 후보

- 11개 PR / 47개 원 커밋을 모두 `-x` 체리픽했다. 적용 완료 head: `3a8f25685830e214ee49b11cfd5a38b64af4dfa8`. 원 PR head는 적용 직전 GitHub에서 재확인했고 모두 고정 intake SHA와 같았다.
- 충돌 1건: #7956 `a8359e632e6f938672c3af88d5dee865e5fe8216` → `d8eef6185`. 최신 devel의 `paragraph_line_indent_for_source`와 `frame_owns_inline_table` 우선순위를 유지하고, 저장 column_start가 자기 문단 여백일 때만 추가 들여쓰기를 적용했다.
- 기준 Native 빌드 완료: `8766fb47d8ec11547f0d2ae941c53cc3c9780ab2`, 실행 파일 SHA-256 `deadff7c0273b7f3b9a8a3716dfc48cb93b190c11697aaa3face9e78f3b6f848`. 기준 fresh WASM과 후보 검증은 아직 진행 중이다.
- 기존 1342000 PDF 2종의 p202 raster SHA가 같음을 확인했다. 실제 current-source Print는 원문 대응이 기록된 415쪽 자료(SHA-256 `82fe8eefecbf3ad5176549976a8cd486ac0bb9c113e7ac34510a69ee6ccbf17f`)를 재사용한다. 새 MCP 변환을 하지 않았다.

### 야간방호일지의 측정 → 배치 경로

독립 PDF의 붙임 글줄 원점은 저장 vpos 71077HU와 같은 쪽 원점에서 약 1023.3px이다. 기준 Native는 1052.0px에 놓인다. `typeset/section/vpos.rs`의 다음 문단(pi5) 진입은 246.29 → 926.36px의 저장 스냅을 수용하지만, paint는 앞 TAC 캐리어(pi4)의 저장 위치에 표를 그린 뒤 현재 흐름 978.17px에 다시 52.47px을 더해 1030.64px로 진행한다. 그 뒤 빈 줄·본문이 같은 차이만큼 내려간다.

1차 후보는 순서대로 `vpos_snap_current_height`의 확정 경계 → `record_stored_paragraph_flow_end` → `ColumnContent.stored_paragraph_flow_ends` → `layout.rs`의 마지막 문단 항목 완료 분기로 동일한 끝을 전달한다. 새 좌표 상수나 clamp를 추가하지 않는다. 원본 저장 줄의 연결·완전한 앞 문단 소유·공백 TAC 캐리어·가용 쪽 예산을 확인하며, 편집/합성/되감김·소유가 다른 부분 문단은 비적용이다. 여러 TAC가 있어도 같은 줄이라고 추정하지 않는다. 이 후보의 빌드·실행·Native/fresh WASM 확대 판독과 반례는 아직 **미검증**이다. 다른 두 지적 페이지도 완료로 판정하지 않았다.

### 공통 로직으로 교체 — 사용자 지시 반영

`cd753eb26`의 공백 TAC 캐리어 전용 판정은 폐기했다. 해당 후보 빌드는 공통 로직 교체를 위해 중단했고, 빌드 실패나 검증 통과로 해석하지 않는다. 문서·공백·TAC 조합에 따른 새 판정 helper를 제거했다.

공통 `vpos_snap_current_height`가 이미 결정한 흐름 경계를 같은 단의 직전 완료 문단 소유자에게 전달한다. 기존 `record_stored_paragraph_flow_end` → ColumnContent → 마지막 문단 항목의 배치 완료 경로를 재사용하며, paint에서 저장 좌표나 줄 높이를 다시 추정하지 않는다. 다른 단·부분 문단은 완료 소유자가 아니므로 해당 경계로 덮어쓰지 않는다. 문단 앞 간격은 기존 HeightCursor가 이미 처리한 값을 그대로 소비한다. 이 공통 후보도 실행 전이므로 **미검증**이다.

원 PR p202 마지막 칸을 확대하면 동일한 두 문단의 PDF/RHWP 위치가 위아래로 갈린다. 초기의 '글줄 겹침' 관찰은 overlay의 두 출력이 갈리는 현상이며 RHWP 내부 중복으로 확정한 것이 아니다. 실제 후보의 컷·세로 정렬과 다음 쪽 소유를 함께 확인한다.

### 공통 후보의 실행 결과와 후속 보정

`d70578a320ddee1dc278be51a36dceb4aa135eec` Native 빌드는 종료 0이며, 지침만 추가한 `7f837e4b2`의 fresh WASM 빌드도 종료 0이다. Native 실행 파일 SHA-256은 `803e985a21a2af446d7dc33c94fb3cee3ced2e0ea4db5e20863bd0a4724501e0`, WASM은 `b3cf99c41f03687b67a3f7ba8d93815cd367fb0db441b3fcd3bff5368dcc4323`이다. 두 backend의 시각 검증은 별도 기록하며 빌드 성공을 시각 통과로 해석하지 않는다.

| 범위 | 기준 Native → 공통 후보 Native | 직접 관찰·남은 작업 |
| --- | --- | --- |
| 야간방호일지 p1 | raw 95.19973 → 97.73177%, 1/1쪽 | 붙임 글줄 y=1023.3px, 마지막 빈 줄 y=1044.6px로 본문 안에 복귀. fresh WASM·관계형 검사 필요. |
| 80168 p17 | raw 89.33934 → 98.17791%, 157/157쪽 | 아래 세 칸의 빈 다음 행 띠·가로선 및 정렬 차이는 남음. 점수로 면제하지 않음. |
| 1342000 p202 | raw 86.32839 → 97.81029%, 419→415/PDF415쪽 | row94 col8의 실제 컷은 0..2, 다음 쪽은 2..3으로 내용 보존. 첫 조각은 Top 강제, 다음 조각은 Center라 첫 프레임 정렬 소유가 누락됨. |
| 156465345 전9쪽 | 기준10/PDF9쪽 → 후보9/PDF9쪽; 후보 최저 raw 84.58908% p1 | 나머지8쪽은 90% 이상. p1의 문단 끝 되감김과 겹침을 먼저 보정하며 기대값을 변경하지 않음. |
| 148738070 / 1150200 | 후보7/7쪽·최저96.62296% / 4/4쪽·최저94.43547% | Native 전쪽 TSV 확보. fresh WASM·확대 판독 필요. |

공통 흐름 후보는 `y != current_height`의 부동소수점 끝자리 차이까지 실제 이동으로 취급했다. 156465345 p1의 빈 pi11은 저장 successor 계약 대상이 아닌데도 끝 750.933333px가 전달되어, 실제 그린 pi11 시작 894.16px보다 앞인 864.31px로 paint 흐름이 되감긴다. 좌표 비교를 원본 HU 단위로 통일해 같은 원본 위치의 수치 오차가 새 소유 경계를 만들지 않게 한다. 이 수정의 새 빌드·전후 실행은 아직 미검증이다.

병합 칸은 이웃 단일 행의 원본 높이 합이 병합 칸의 선언 높이를 닫고, 그 원본 행 prefix가 선택한 컷과 같은 위치에서 저장 원점으로 복귀할 때 프레임 소유를 인증한다. 양수 앞 간격도 원점에 포함한다. `stored_span_row_owner`의 공통 결과를 기존 컷 산출과 `stored_plain_spanning_frame_owns_alignment`가 소비하며, 실제 수용한 행 상자 높이는 바꾸지 않는다. 자동 높이·선언 불일치·편집/재조판·중간 로컬 재시작·진행 없는 반복 원점은 인증하지 않는다. 1342000 원본의 row94/95 이웃 높이 4173+2782HU=병합 칸6955HU, 컷 뒤 원점200HU·이전1700HU가 이 근거이며, 반례·수정 후 Native/fresh WASM은 미검증이다.

전체 TSV·명령·producer·PNG는 ignored `output/pr-review/planet6897-20261018/`에 보존한다. 미달 페이지와 전쪽 검증이 남아 있으므로 PR 제출·승인·merge 단계에 도달하지 않았다.
