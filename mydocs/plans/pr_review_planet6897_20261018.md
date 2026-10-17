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
