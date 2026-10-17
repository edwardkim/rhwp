# planet6897 열린 PR 11개 체리픽 통합 계획 — 2026-10-18

## 요청·기준·작업 경로

사용자는 열린 PR들의 **체리픽 머지와 후속 처리**, 리뷰어 지정 선행과 필요한 메인터너 보정을 명시했다.
원 PR을 각각 직접 merge하지 않고 최신 devel 위 한 브랜치에 기능 commit을 누적 적용한다.

- 기준 devel: `8766fb47d8ec11547f0d2ae941c53cc3c9780ab2`.
- 작업 브랜치: `review/planet6897-20261018`, 기본 작업공간 `/home/tsjang/rhwp`.
- 11개 모두 non-draft. 먼저 `jangster77` reviewer 지정을 완료했고, 원 author·공동 저자·fork와 `-x` 출처를 보존한다.
- 초기 원 PR source 47 commit의 Git cherry 검사에서 기준 devel과 patch-equivalent인 commit은 없었다. 이후 #7965의 재작성된 기존20개는 stable patch-id가 모두 일치함을 확인하고 새2개만 추가해 총49개를 적용했다. PR 안의 merge commit은 없었다.
- 기본 경로: collaborator_external_pr 9.1.1. 보조: intake/local_validation, multi_pr_update_branch, visual_fixture_evidence, renderer backend, 대형 PR(#7965), CI 후 review-only trailing, post_merge.

## 적용 순서

| 원 PR | 고정 source SHA | commit 수 | 변경 |
| --- | --- | ---: | --- |
| [#7934](https://github.com/edwardkim/rhwp/pull/7934) | `a60c275e8abbce2c6cb9e4277422b7084b13e830` | 4 | fix(layout): 병합 격자 표 열 경계를 저장 칸 폭 누적으로 푼다 (#7933) |
| [#7956](https://github.com/edwardkim/rhwp/pull/7956) | `dfafbbbf50c9c2cf778174aa3c7c208e99e91c8b` | 9 | fix(조판): #7840 저장 프레임 되감김 오판·이어받기·꼬리말 TAC 줄 상자 — 148738070·1150200·156465345 쪽수 PDF 일치 |
| [#7965](https://github.com/edwardkim/rhwp/pull/7965) | `4c83c6112a038cf0e30ee164e12c4c0a98f4e6af` | 22 | fix(조판): RowBreak 표 분할·이어받기를 저장 쪽 경계 기록대로 — 1342000 쪽 수 415 일치·21761835·1480000·1613000 (부분 개선) |
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

### 원문·전쪽 대조와 다음 공통 보정 후보

`a5437f9c773a164f56d1e7100f9faf482852afa1`의 Native/fresh WASM 빌드는 모두 종료0이다. Native 지적 페이지는 야간방호97.73177%, 80168 p17 98.17791%, **1342000 p202 99.89483%**다. p202의 두 문단을 독립 PDF와 직접 대조해 첫 조각 가운데 정렬 복귀를 확인했다. 꼬리말156465345는 전9쪽 최저88.71602%로 개선됐지만 미달이므로 완료가 아니다. fresh WASM 시각·최종 전쪽 gate는 별도로 확인 중이다.

앞 공통 후보의 80168 전157쪽은 최저35.27497%·미달17쪽이다. 고정 devel Native 전쪽과 Decimal로 비교한0.01%p 이상 변화는15쪽 개선·하락0쪽이다. 1342000 전415쪽은 최저28.52187%·미달140쪽이다. 단순 정렬 차이 외에 p84처럼 PDF 이전 쪽의 내용이 현 쪽에 남는 분할 위치 차이도 직접 확인했다. 따라서 대표 세 쪽의 통과를 전쪽 통과로 쓰지 않는다.

#7872/#7873의 gist 원본은 PR의 변형 fixture와 다르다. 고정 gist 버전·입력 SHA와 XML 차이를 ignored output에 보존했다. 원본 nested 입력 SHA-256은 `b9f285f0ce01773c9c64240392477a374e5c67bef00eabe0ace36af66ccae0ba`, 수식 원본은 `351d4b8b482dadffa2cf3a79037ad9397563ab670ce33c3770254c35d5c20a93`이다. 두 원본에 대응하는 기존 Print PDF가 저장소에 없어 같은 원본을 engine2020 PrintMethod0으로 출력했다(job `f47c14ee-999a-4fc5-adbe-e8055ae2cdb0` / `cbb76bf4-e11e-4ec0-9a71-ef85a0d0e58c`, 각각1쪽). 실제 PDF는 KoPubWorldDotumMedium을 내장하며 설치 글꼴 hmtx의 한글872/1000·공백290/1000도 직접 확인했다. 앞 후보 Native 원본 raw는21.90060% /30.65664%로 미달이다. 변형 fixture의100%·99%를 원본 해결 근거로 바꾸지 않는다.

다음 후보는 다음의 공통 계약을 보정한다. 아직 새 실행 전이므로 모두 미검증이다.

- **비활성 셀 여백**: #7972의 제보 원본과 현재 `Cell::effective_padding` 계약이 같은 원인임을 확인했다. `resolve_row_heights_with_common_fit`의 단일 행 성장 경로만 raw cell.padding을 부활시키며 병합 행 경로와 측정은 이미 유효 여백을 쓴다. raw 복원과 선언 크기별 예외를 제거하고 기존 공통 유효 여백을 모든 행 성장에서 소비한다. 기존 #3386 선언 높이·#5301 aim=false/true 대조를 함께 확인할 예정이다.
- **완전 행 prefix의 물리 끝**: 80168 p17의 컷은 row8의 모든 내용만 소비하고 row9 이후 유닛은0이다. 기존 `SelectedBlockCut::end_row`는 컷의 물리 높이51.1px가 row8의44.1px를 넘는다는 이유로 미소비 row9를 열어6.9px 빈 칸과 괘선을 만든다. 기존 완전 행 경계 인증으로 원본 끝 row9(반열린 끝)를 고르고, 같은 컷의 물리 높이를 마지막 소유 행의 override로 전달한다. `complete_row_boundary_frame`의 예약 높이도 물리 컷 이상으로 보존해 paint만 커지는 불일치를 만들지 않는다. 부분 유닛 컷·이어받기는 이 완전 prefix 계약으로 바꾸지 않으며 #7336 등 기존 부분 컷 대조가 필요하다.

80168 p93은 `keep_with_next=false` 제목 pi782가 PDF p92 하단에 있지만 rhwp p93 상단으로 밀린다. 기존 `native_hwp5_circled_rowbreak_table_heading_requires_fresh_page`가 동그라미 글자·빈 줄·1×1 표 조합으로 묶음을 추정하는 경로를 확인했다. 이 가정의 독립 근거와 기존76076 대조는 조사 중이며 아직 코드를 변경하지 않았다.


### 체리픽 선행 순서 재정렬과 완전 행 커서 후속 보정

사용자 지시에 따라 추가 source 두 개(`27bd84bf3`, `4c83c6112`)를 #7965의 기존20개 뒤로
옮겨 **기여자49개 → 메인터너 보정5개** 순서로 interactive rebase했다. 계획 commit은 맨 앞에
유지했다. 재정렬 head는 `dd846c62e9f3191cc65be84d8df0f7b479f675c8`이며, 이전
`ffdf1c478d64cd667396855fd21938ec92d32b8a`와 최종 tree가 동일하고 원저자·저자일·출처
trailer49개가 보존됨을 확인했다. `upstream/devel`은 다시 fetch해도 `8766fb47d8`로 동일했다.
ignored output의 `source-order-rebase-result.json`과 `cherry-pick-mapping.json`에 전후 SHA가 있다.

이 head(candidate5)의 Native 빌드는6분3초·exit0이며 fresh WASM도 exit0으로 종료됐다.
Native SHA-256은 `0803fbf729e1e469ed34b33589a1063c816e0148c2b1fc7665bb164e445046e4`다.
Native 지적 쪽 raw는 야간방호97.73177%, 80168 p17 **99.47393%**, 1342000 p202 **99.89483%**다.
그러나 80168 p18은 직전98.38128%에서 **44.22378%**로 내려갔다. p17만 개선됐다는 이유로
완료 처리하지 않는다. 원본 nested의 유효 여백 보정은21.90060→89.50846%로 개선됐지만 미달이다.

실제 호출 연결은 `SelectedBlockCut::end_row`의 완전 prefix(끝9) → block_step의 양수
`split_end_limit`·잔존 cut vector → emit의 `continuation.advance(..., partial=true)` → 다음 조각
시작8이다. trace에서 `FRAGMENT rows=0..9 ... cut=[2,2,1,1,1,0,0,0,0,0]` 뒤
`rows=8..24`가 확인됐다. 즉 물리 끝만 바꾸고 부분 컷 커서를 남겨 끝난 행을 다시 열었다.

후속 보정은 기존 `row_block_cut_is_complete_row_boundary`의 원본 유닛 인증을 통과한 컷의
공통 `complete_row_boundary_frame` 결과를 소비한다. 최소 orphan 높이 미만인 경우뿐 아니라
인증된 완전 prefix 모두에서 물리 행 상자와 **빈 끝 컷·0 부분 limit**을 함께 전달해 다음 행에서
재개한다. 기존 저장 병합 상자의 쪽 끝 밴드 인증은 우선 유지한다. 부분 유닛·hard break·편집·재조판
경로는 이 완전 행 계약으로 바꾸지 않는다. 같은 source의 Native/fresh WASM, p17→18 내용 보존과
다른 source 경계는 새 코드에서 재검증할 예정이며 아직 **미검증**이다. 기대값을 바꾸지 않았다.

#7873 원본의 빈 pi8은9pt/100% 스타일인데 높이13878HU·간격540HU를 가진다. 한컴2020
재저장 대조본(job `ab136fd2-90f5-4233-b4a4-11b3893f59a7`, 11.0.0.9136,
`input_preprocess=none`)은 같은 빈 줄을900HU·간격0으로 다시 계산한다. 첫 메타 문단도
1000→900HU로 재계산됐다. 대조 HWP SHA-256은
`6febf8b23007179b4108494c45ae2917efc8119ccd357784dc585be4d7fc698a`다.
원본 HWPX의 보존 줄 좌표는 모두0이며 일부 본문은 줄 배열도 없다. 이 혼합 입력의 수용과
재조판 계약은 조사 중이다. 대조본 통과를 원본 통과로 대신하지 않으며 원본·PDF를 교체하지 않는다.


### 완전 행 경계의 소비 유닛 보존 — 수용 공간 장부와 분리

`95b63a0541c07a3d07bd418c4e1faef72843f42e`(candidate6)의 Native 빌드는5분39초·exit0,
실행 파일 SHA-256은 `2be39c50db74374295c2a75c46ebea8b4f1d1250b826939a61bd58ace0494936`이다.
p17/p18 raw는99.29537%/98.75684%로 위치가 복구됐지만, p18 overlay를 직접 판독하면 PDF에
없는 앞쪽 병합 칸의 `적정성 여부`가 다시 나타난다. 높은 점수로 내용 중복을 면제하지 않는다.
앞 후보의 빈 컷 전환은 폐기한다. p92/p93은97.99810%/35.27497%로 별도 제목 이월 미달도 남는다.

다음 공통 후보는 블록 컷과 블록 시작 커서를 원래대로 보존한다. `complete_row_boundary_frame`
이 확정한 행 상자와 끝 컷을 emit의 `StoredFragmentEnd`에 함께 넘긴다. `record_stored_fragment_rows`
는 같은 완전 prefix 인증과 실제 마지막 행 override가 모두 있을 때만 해당 행들의 수용 높이를
`accepted_row_boxes`에 기록한다. 다음 `stored_fragment_row_plan`은 **완료 컷과 실제 수용 장부를
둘 다 확인**하고, 이미 앞 조각이 사용한 prefix의 현재 조각 높이를0으로 만든다. 나머지 행은
원래 높이를 유지한다. `fragment_measurement`가 이 행 계획을 스캔에 넘기고 같은
`fragment_row_heights`를 PageItem에 실어 배치가 소비한다. 병합 칸의 cut vector는 보존되므로
선택 유닛이 다시 그려지지 않는다.

이는 내용 컷만 보고 빈 공간을 없애거나 paint를 감추는 처리가 아니다. 실제 수용 상자가 없는
내용 완료 컷·미완료 유닛·hard break·편집/재조판은 새 완료 prefix 경로가 아니다. 같은 공간을
두 쪽에서 다시 세던 오류를 공통 수용 결과로 바로잡는다. 새 후보의 빌드·p17→18 확대 판독·
Native/fresh WASM·부분 컷 반례는 아직 미검증이며 정식 회귀의 기대값은 변경하지 않았다.


### 공통 후보7 실행과 다음 원인 계층

`80bc01897735c589042ead8a6e47f8428779d744`(candidate7)의 Native 빌드는5분7초·exit0이다.
p17/p18 raw는99.47393%/98.38128%이며 overlay에서 빈 띠·앞 조각의 병합 라벨 중복이 제거됐다.
p18 render tree의 `적정성`은 candidate6에만 있고 candidate7과 독립 PDF에는 없다. 이 내용 소유
진단은 ignored `candidate7-page18-content-ownership-diagnostic.json`에 기록했다. 전체157쪽은
같은 동결 CLI로 다시 산출 중이며 아직 전쪽 통과나 fresh WASM 완료로 판정하지 않는다.

다음 후보는 현재 미달의 두 원인 계층을 보정한다. 아직 실행 전이다.

- **제목 이월**: 80168 pi782의 `keep_with_next=false`와 한컴 PDF p92의 실제 소유가
  동그라미 번호·빈 문단·1×1 표 조합으로 강제 이월하는 기존 가정과 모순된다. 기존 가정의
  주석이 든76076의 동일 원본2024 Print p55/p70과 candidate5 Native는98.07848%/99.85341%이며,
  두 문서의 같은 제목들도 해당 속성이 false다. 내용 모양에 따라 묶는 helper와 호출을 제거하고
  공통 문단 fit 경로를 사용한다. p92→93과76076 p55/p70을 같은 입력·PDF로 전후 대조한다.
- **미계산 XML 본문 줄 캐시**: #7872/#7873 제보 원본과 #7865 샘플은 본문 저장 세그먼트의
  위치가 모두0이며 일부 문단은 세그먼트도 없다. 한컴2020 재저장 대조본은 첫 빈 메타 문단의
  1000HU를9pt 글자모양의900HU로, #7873 빈 pi8의13878HU·간격540HU를900HU·0으로 재계산한다.
  대조본은 원본 Print와98.48018%, 원본은30.65664%다. 문서나 특정 높이로 분기하지 않고,
  명시적 후속 쪽/단 나눔 없이 본문 전체가 미계산 원점인 XML import 구역을 공통 재조판한다.
  저장 위치가 전진하는 구역·한 문단만 있는 구역·명시적 경계·접힌0높이 줄은 이 판정 대상이
  아니다. HWP5-origin/HWP3 등 기존 XML 재조판 제외 계약은 기존 include_empty 경로가 유지한다.
  원본 파일과 IR 출처를 보존하며, 기존 reflow 함수가 만든 줄 결과를 reflowed_paras와 본문
  vpos 체인에 연결한다. 측정·페이지네이션·paint는 같은 재계산 LineSeg를 소비한다.

미계산 본문 판정은 새 작은 공용 모듈에 두고 본문만 재조판한다. 셀의 로컬 원점0은 정상이라
본문 판정을 셀로 재귀하지 않는다. 원본 제보2건·수식/중첩 표 변형 대조군·양수 저장 사다리와
명시적 경계 대조를 확인한다. 원본 문서를 변형 fixture나 한컴 재저장 대조본으로 교체하지 않는다.
새 구현의 Native/fresh WASM 및 필수 회귀는 미검증이며 현재 PR gate 미달은 남아 있다.


후보7의 전157쪽도 종료됐다. Native 최저35.27497%·90% 미만17쪽이며 candidate3 대비
0.01%p 이상 변화는 p17 개선만 있고 하락0이다. 고정 devel 대비15쪽 개선·하락0이다.
비교는 `candidate7-native-80168-comparison.json`에 보존했다. 전쪽 gate는 미달이다.

다음 후보의 미계산 본문 처리에서는 재계산 전에 기존 `replace_line_segs(Vec::new())`로 본문
캐시를 무효화한다. 낡은 개체 줄 간격과 tag를 template으로 재사용하지 않고 기존 reflow 함수가
생성 표식이 있는 LineSeg를 만든다. 원본 파일·문자·control·셀의 원본 캐시는 바꾸지 않는다.


### 최신 devel 재기반과 걸침 셀의 실제 수용 컷

`upstream/devel`의10개 새 commit을 받아 `4296fa045df2743a0eaa1b0cc71b2a3c1dd90bb6` 위로
리베이스했다. 후보9 head는 `39d85c284f14f91a1a7a1f819dfb9156209f184f`다. 코드 충돌은 없고,
59개 patch의 range-diff가 모두 동일하다. 원 기여49개가 메인터너 보정보다 앞서며 author·date와
미추적 PDF4개도 보존했다. 일시적인 index lock은 이미 해제된 것을 확인해 continue했으며
lock 파일을 제거하지 않았다. Native 빌드5분41초·fresh WASM6분19초가 exit0으로 완료됐다.
실행 명령·산출물 SHA는 ignored `candidate9-*-build-results.json`, 재기반과49개 출처 대응은
`upstream-4296fa045-rebase-result.json`·`cherry-pick-mapping.json`에 있다. 후보9 Native의
원본2건은99.89402%·98.48018%, 집중8개 중156465345 p1은88.71602%로 미달이다.
후보9 나머지 Native/fresh WASM 실행과 아래 변경 후 검증은 아직 완료되지 않았다.

직전 후보8 원본2건의 fresh WASM도99.91495%·98.24952%다. Native/WASM compare를 직접
판독해 중첩 표의 보기①–⑤와 후속 내용, rm 수식·분수와 본문의 대응을 확인했다. 이 수치는
리베이스 전 producer의 결과이며 최종 head 검증으로 대신하지 않는다. 리베이스 중 잠시
없어진 수식 입력2개의 ENOENT는 환경 오류로 구분했고 같은 동결 WASM으로 재실행해 통과했다.

1342000 구역1 pi10의 Sejong 걸침 칸(행17·열3)은 p82에서238.346667px를 수용했지만,
종료 RowCut `[1,1]`은 경계 행18의 **다른 일반 셀**을 가리킨다. 기존 코드는 이 컷의 짧은
내용 높이를 걸침 셀의 소비 공간으로 재사용해 유닛0..0을 선택했고, 다음 조각은 처음부터
다시 그렸다. 원본 저장 줄은 유닛13(문단11)에서0으로 되감기며 독립 Print p83의 소유와
대응한다. 원본·PDF·후보7 unit trace와 p82–85 독립 PDF 텍스트를 보존했다.

공통 `rowbreak_straddle_cut_units`는 원래 저장 프레임 owner 인증을 유지하되 일반 fallback의
종료 예산을 **자기 실제 수용 상자 높이**로 잰다. 이 선택 결과는
`accepted_straddle_fragment_units` → `StraddleCellCarry.fragment_cut/next_unit` → 다음 조각과
`table_partial`의 우선 컷 경로로 전달된다. 별도 장부가 없는 시작 행의 rowspan paint fallback도
같은 helper를 호출한다. 일반 셀 RowCut 높이로 다시 추측하던 중복 공식을 제거했다.
공통 메서드는 기존 작은 `stored_span_content.rs`로 옮기고 물리 높이와 내용 유닛의 역할을
같은 모듈에 둔다. 별도의 문서 ID·높이 상수·출력 숨김을 추가하지 않는다.

대조 경계는 #6803의 시작 행 걸침 칸(5쪽 전체), #5862의8쪽 끝 글줄 보존, 1342000
p82→83의 저장 되감김과 p84 후속 표, 기존 보정 p202→203, 80168 p17→18이다. 위 사례의
수정 전 동결 render tree를 보존했다. 새로운 구현은 아직 빌드·실행 전이다. 전쪽 gate 미달
문서의 기대값·golden을 갱신하거나 새 렌더링 회귀를 추가하지 않았다.

80168 p93의 별도 잔여 원인도 좁혔다. 통째 이월된 래퍼 표가 자식106.04px만 수용하고
밖의116.67px 최소 프레임을 소비하지 않는다. 실제 paint는194.15px까지 도달하나 수용
`occupied_bottom=109.8`을 다시 적용해 흐름이185.4px로 되돌아간다. 이 경로의 전체 프레임
fit·emit·paint를 같은 높이로 연결하는 보정은 다음 단계이며 아직 적용하지 않았다.


### 실제 수용 컷의 Native 관측과 통째 래퍼의 공통 프레임

후보10 `2786e299263258f61860bf8369c40201c0935c3b` Native 빌드는6분43초·exit0,
산출물 SHA-256은 `42c60c8923d1a4030aca21c69a1dd0f5ea52828370db8b1018740c58e2700d4a`다.
1342000은415쪽을 유지하며 위 걸침 칸의 첫 조각 컷이0..0→0..13, 다음 조각 시작이0→13으로
바뀌었다. 다음 조각의 나머지 요구는350.213333→124.88px다. #6803의 원본67문단이
기존과 같은 쪽·순서로 정확히 한 번씩 보존되고, #5862 p8 render tree는 수정 전과 byte-identical다.
이는 동결 CLI 출력의 진단이며 아직 공식 nextest 통과로 보고하지 않는다. 영향 페이지 시각
비교와 fresh WASM은 계속 검증한다.

별도 원인인 통째 래퍼 프레임은 같은 `TableWrapperVerticalFrame`을 사용하는
`whole_table_frame_height`로 조회한다. 최초 whole-fit 포맷과 행 스캔 후 whole 확정이 같은
공통 결과를 소비한다. 실제 컷·이어받기·행 높이 override가 있는 조각은 기존 조각 계약이다.
전체 미소비 행을 whole로 확정할 때 외곽 프레임이 현재 행 예산에 안 들고 현재 단에 앞 내용이
있으면, 내용·물리 커서를 소비하지 않은 상태로 이월한다. 상자를 실제로 놓는 분기에서는 같은
whole 높이를 흐름 소비와 `fragment_placement.occupied_bottom`에 전달한다. paint는 기존
공통 프레임으로 그리므로 별도 출력 좌표 보정을 추가하지 않는다.

독립 기준은80168 p93의 전체 래퍼와 뒤 빈 문단·③제목·다음 표다. 작은 자식의 행 합만
소비하면서 더 큰 부모 최소 상자를 그린 기존 경로를 제거한다. 더 큰 자식·명시적 부분 컷·
이어받기, 76076 p55/p70과 기존 보정 p17/p18이 반례다. 새 후보 실행은 아직 미검증이다.
50MiB 이상 PDF·원본·기준값을 새로 추가하거나 회귀 기대값을 완화하지 않는다.

찾아 바꾸기 #7979도 후보9의 실제 CLI 편집·HWPX 저장으로11개 원본/한컴 편집 대조쌍을
확인했다. 기준8766 CLI는9쌍에서 본문·셀의 글자 모양 상속이 다르고, 후보9는11쌍 모두
텍스트·bold·색상 서명이 일치한다. fresh WASM의 replaceAll11건과 본문의 replaceOne10건·
replaceText10건, 각각 저장 뒤 재열기31건도 같은 한컴 대조본 서명을 모두 보존했다.
대표4건(전체 굵게·표 셀·길어진 치환·두 모양 혼재)의 Native/WASM 저장 바이트가 동일하고,
그 저장본을 재개방한 양 backend의 전체1쪽 raw TSV는 각각100%다. 확대 이미지에서 앞뒤
보통 글자·첫 글자 굵기·후속 빨강과 셀 경계를 직접 대조했다. 증거는 ignored
`issue7866-native-cli-pair-results.json`, `candidate9-issue7866-wasm-edit-api-results.json`,
`candidate9-issue7866-edit-output-byte-equivalence.json` 및 `candidate9-*-issue7866-edited-visual/`다.
현재 코드는 표 분할 추가 보정이 있으므로 최종 source의 필수 검증은 별도로 이어간다.


### 통째 래퍼 실행 결과와 남은 저장 경계의 가정 제거

후보11 `abfbbd6a7ed4db09d9be54c2df442d9e050b0f36` Native 빌드는5분37초·exit0이다.
80168 p93 raw는57.24791%→99.98337%, p92는99.75569%로 유지됐다. 같은 독립 PDF와
review/overlay를 직접 판독해 표 뒤③제목·다음 본문과 두 표의 간격이 복구된 것을 확인했다.
전157쪽·fresh WASM은 아직 완료되지 않았고 code head의 cargo fmt --all -- --check는 통과했다.

후보10의1342000 p82/p83/p84 raw는48.17574%/49.68764%/48.58609%다. 직전 후보9의
같은 동결 CLI 비교는41.36844%/48.19388%/28.52187%다. p202/p203은99.99431%/99.99541%다.
개선됐지만 p82 맨 위에 PDF의 앞 쪽에 있어야 할 제6조 제목이 남고, p83·84도 앞 쪽의 마지막
한 줄이 남는다. 점수 상승으로 이 잔여 소유 차이를 면제하지 않는다. #6803 전5쪽 Native는
최저95.69907%·미달0이며 문단67개가 보존된다. #5862 p8은47.52852%·p9는99.03018%라
시각 통과로 보고하지 않는다. p8 tree는 이전 후보와 동일하며 이 변경에서 생긴 새 차이는 아니다.

다음 공통 후보는 두 추측을 제거한다. 1342000의 광주 row14는 문단3 원점0을 보고
강제 RowCut에서 멈추지만, 독립 Print는 그 제목을 앞 쪽에 싣는다. `original_stored_text_frame_cell`
의 표 전체 행 합이 선언보다 크다는 사실은 **각 문단 경계가 현재 물리 쪽의 끝이라는 증거가
아니다**. strict RowCut의 이 단독 문단 되감김 분기를 제거한다. 선언 프레임·실제 본문 끝·
공유 셀·문단 내부 원본 줄 등 개별 경계를 인증하는 나머지 경로와 일반 가용 높이 계약은 유지한다.
또한 이미 실제 수용 높이에서 선택한 걸침 셀 종료 유닛을 원점0 근처라는 이유로 다시 한 유닛
되감지 않는다. 독립 저장 owner는 먼저 선택하고, 일반 fallback은 수용한 물리 컷을 다음 조각에
그대로 전달한다. 수용 공간을 늘리거나 clamp·문서 분기를 추가하지 않는다.

이 후보의 동작은 아직 미검증이다. #6803/#5862의 내용 보존, 원본1×1 page-scale reset
대조(148738070), 1342000 p81–84·202–203과80168 p17–18·92–93을 우선 확인한다. 모든
필수 source별 증적과 전체 검증이 준비된 뒤 upstream/devel을 다시 fetch해 최신 기준 포함 여부와
리베이스 필요성을 확인하고, 필요한 재검증 후 PR·CI·같은 PR의 trailing 기록·merge·후속 정리를
계속한다. 현재 미달과 미검증이 남아 있으므로 아직 PR을 생성하거나 승인하지 않는다.


### 고정 px 휴리스틱 대신 실제 문단 유지와 수용 예산

후보12 `cab444202767441699ff5b72c883b64d4944f5f9` Native 빌드는6분29초·exit0이다.
앞의 가정2개 제거만으로1342000 p82–84의 점수는 바뀌지 않았다(p81=85.84021%). 해당
미달을 해결했다고 보고하지 않는다. 실제 ordinary RowCut에는 위 행의 rowspan이 현재 행을
가로지른다는 이유만으로 저장 hard reset을 강제하는 조건과 남은 공간32px 기준이 추가로
있다. 광주 row14 앞에는 내용 흐름과 무관한 앞쪽 병합 칸이 걸치며, 줄 하나가 들어가는
여유가 있어도 이 두 조건으로 멈춘다. 상단 병합의 존재는 그 문단의 페이지 유지 계약이 아니다.

다음 후보는 일반 RowCut의 **강제 break 사유**에서 prior-rowspan을 제거한다. 실제 break가
발생했을 때의 기존 문단 orphan 재검토에는 그 정보를 그대로 쓴다. 일반/블록/행 오프셋 블록
세 경로의 relaxed hard-reset fallback도 남은32px 자체로 멈추지 않고 다음 유닛의 실제
요구가 각 경로의 예산에 안 들 때 멈춘다. 독립 저장 프레임 인증과 일반/특수 경로의 기존
비완화 조건은 유지한다. 원본 프레임처럼 보인다는 이유만으로 컷 소유를 확정하지 않는다.

80168 p60→61(pi516)에는 별도 근거가 있다. row12 내용17.333px가 가용27.56px에 들어가고,
독립 PDF는 `1. 국제조약 … 되어야`라는 완결 문단을 p60에, 다음 문단을 p61에 둔다.
원본의 해당 문단·이웃 셀은 keepWithNext/keepLines/widowOrphan이 모두 false다.
현재 공통 문단 유지 helper가 저장 LineSeg 없는 문단을 제외해 25px 휴리스틱으로 넘긴 것이
기각 사유다. 저장 줄이 없거나 편집으로 재생성돼도 `CellUnit`의 실제 문단·글줄 소유와 현재
style의 유지 지시를 확인하도록 `table_paragraph_cut.rs`의 캐시 출처 게이트를 제거한다.
중첩 제어·세로쓰기·가시 문단이 없는 경우의 기존 제외와 실제 물리 예산 검사는 보존한다.
이는 저장 좌표 수용을 완화하는 변경이 아니며 line-seg·dirty tag를 페이지 경계 증거로 쓰지 않는다.

소비 경로는 row_step의 최소 조각 판단 → continuation/fragment의 시작 문단 계약 → emit의
실제 수용 밴드다. 모두 같은 CellUnit 컷/문단 유지 결과를 읽으며, emit의 독립 물리 상자 조건은
별도로 남는다. 실제 paragraph keep 지시가 켜진 반례와 재조판·기존 저장 경계를 함께 검사한다.
새 후보의 실행은 아직 미검증이다. 현재 전쪽80168 후보11은157쪽 중14쪽이90% 미만,
후보7 대비6쪽 개선·0.01%p 하락0이다. 대표93쪽 통과를 문서 전체 통과로 바꾸지 않는다.


### 후보13의 새 회귀 분리 — 통과로 처리하지 않음

후보13 `f49b50e367655e8fbbd67275e771078be7b1e3c4` Native 빌드는8분18초·exit0이다.
80168 p60/p61은98.72423%/97.88584%로, 완결 문단 컷 보정 방향은 개선됐다. 그러나
1342000의 전체 쪽수가415→417로 증가했고 같은 쪽81–84 raw도6.39742%/40.40355%/
44.25689%/18.18107%로 하락했다. 이는 실행으로 검출한 현재 보정의 새 회귀이며 수용하지 않는다.

원인 분리를 위해 후보13에서 함께 변경한 ordinary/두 block 경로의 hard-reset 예산 조건을
후보12 상태로 복구한다. 실제 문단 유지 계약 보정만 남긴 후보를 비교하며, 저장 정보·원본·
기준 PDF·회귀 기대값은 바꾸지 않는다. 다음 실행 결과 전에는 어느 조건이417쪽을 만들었는지
확정하지 않는다. 이전 후보의 좋은93쪽·원본 수식 점수를 이번 후보의 전체 통과로 사용하지 않는다.

반복 원인 분리는 공용 `target/pr-review`에서 rhwp 패키지만
`--config profile.release-test.package.rhwp.opt-level=1`로 진단한다. dependency·공유 캐시를
삭제하지 않고 jobs8·기본 incremental 정책을 유지한다. 명령·산출물 SHA·진단 전용 상태를
ignored build metadata에 기록하며, 이 실행은 최종90/98 gate·공식 회귀·lint·fresh WASM의
대체 증거가 아니다. 보정 방향이 확인되면 기본 release-test 최적화로 다시 검증한다.


### 후보14 결과와 독립 단일 행 최소 높이 보존

후보14 `595fc7b4c2864ffc9390b12f049ac75fcecdf7f8` 진단 Native는3분49초·exit0이다.
1342000은 다시415쪽으로 독립 PDF415쪽과 일치하고 p81–84 점수는 후보12와 동일하다.
80168은157쪽을 유지하며 p60/p61=98.72423%/97.88584% 개선도 보존됐다. 따라서 후보13의
새417쪽 회귀는 이번에 복구한 hard-reset 강제 분할 조건 변경에서 생겼다. 실제 문단 유지
계약 변경은 유지한다. 이 실행은 opt-level=1 진단이며 최종 head의 공식 검증을 대신하지 않는다.

80168 p89–90의 마지막 행에는 별도의 원본 최소 높이 소유가 있다. pi760 row30의 단일 행
셀91/92는 각각4679HU(96dpi에서62.386667px)를 선언하고, 독립 Print는 이 행을 p89의
31.1652px와 p90의31.1667px로 나눈다. 앞 row15 col0의 rowspan16은 다른 셀의 점유이며,
row30의 독립 최소 높이가 없어지는 근거가 아니다. 현재 `reflow_row_physical_minimum`은
이미 row_span==1의 원본 높이만 읽지만 prior-rowspan이면 전체 질의를 버려 p90 상자가23.3px로
줄어든다. 이 배제만 제거하고, rowspan 배분이 더해진 해석 행 높이는 여전히 사용하지 않는다.

생산/소비는 원본 단일 행 높이 → `reflow_row_physical_minimum` → scan/row_step의 실제
수용 상자 → fragment/emit의 full−실제수용높이 잔여 → 다음 fragment의 시작 높이와 paint다.
첫 조각이 끝낸 내용과 남은 물리 높이를 구별하며 이미 소비한 유닛을 반복하지 않는다.
범위는 기존 재조판 물리 프레임 경로이며 저장 프레임·중첩·비RowBreak 경로의 조건은 보존한다.
아직 수정 후 결과는 미검증이다. p89/90 직접 대조,157쪽 쪽수,1342000 쪽수/경계와 #6803/#5862
내용 보존을 진단한 뒤 기본 최적화 Native/fresh WASM과 정식 검증을 수행한다.
