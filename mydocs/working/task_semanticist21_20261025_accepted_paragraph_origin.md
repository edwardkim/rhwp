# 배제 영역이 확정한 첫 글줄 원점의 공통 소비

## 독립 근거와 선행 해결

Issue #6970의 원 PR #7118은 `aff426a709c472d812b199ef41a0968f6a96b900`,
누적 적용은 `dd25132d2deb4495a096902f9d48f136cfb7f5bd`다. 통합 #7197의
기록과 [원 검토 증적](../pr/archives/pr_7118_review_impl.md)을 대조했다.
익명화 원본 `tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp`와 독립
`pdf/synth_no_ls_square_wrap-2020.pdf`의 같은 전쪽 비교에서 고정 base와 동등한
Native194는 최저 raw 98.37264%, 현재 `ca4420fad6`의 Native240은 60.85378%다.
원본은 이번 #7445 미완료 제외 목록에 없다. 현재 회귀이므로 정확한 기존 검사를
제거하거나 진단 허용 오차를 넓히지 않는다.

## 실제 생산·소비 경로와 원인

`computed_host_frame::resolve → ParagraphFloatPlacement::ComputedEmptyHost →
TypesetState::resolve_computed_float_first_line → FloatPushedFirstLine.origin →
LayoutEngine::layout_column_item`을 추적했다. 마지막 소비는 일반 글자가 있는
문단 분기 안에만 있어 빈 문단의 장식 개체·줄 전진 경로를 누락했다.

표는 논리 호스트 글줄과 병렬로 점유하고, 뒤 글줄은 표의 배제 띠 아래에서
계속할 수 있다. 이 두 상태를 표 전체 높이의 무조건 전진으로 합치지 않는다.
해당 표의 측정은 두 버전 모두 259.12px이고 실제 표 하단도 446.3px다. 현재
paint는 표 뒤 커서를 호스트 줄 끝 203.9px로 잇고, 앞서 수용한 뒤 문단 원점을
누락해 본문이 표 안에 겹친다. 측정 높이 부족이라는 첫 가설은 이 관측으로 기각했다.

공통 항목 진입점에서 전체 문단과 첫 부분 문단의 확정 원점을 적용한다. 같은
문단의 뒤 개체나 이어지는 글줄은 이를 다시 적용하지 않는다. 배치 결과를 clamp하거나
원본 좌표로 맞추지 않고 수용한 원점을 직접 소비한다. 기존 분기 안의 중복 적용은
제거한다. 변경 뒤 기존 #6970·공통 배제·부동 표·빈 문단 대조군과 독립 PDF를
다시 확인하며, 전쪽 Native/fresh 90% 이상과 확대 판독 전 새 회귀를 추가하지 않는다.

작업 로그와 중간 트리는 ignored `output/pr-review/semanticist21-20261019/`의
`remaining-origin-3876/`, `logs/remaining-origin-*-diag-3878.log`,
`logs/remaining-origin-current-flow-3879.log`, `remaining-visual-3877-result.json`에 있다.
아직 보정 뒤 통과를 선언하지 않는다.

## #7158의 이동된 Square 띠와 접두 줄

기존 #7158 → PR #7163(`da7ec5c090`)의 줄 간격 계약을 유지한다. 기존 실패를
원본 여백 mirror 불일치로 설명한 가설은 실제 속성 setter·양쪽 mirror 변경에서도
같이 실패하므로 기각했다. 원본 HWPX의 75번 문단 표 아래 여백 숫자 하나만
+750/+1500 HWPUNIT 바꾼 대조군을 만들고, 바이트 변경 0인 정상 대조군도 확인했다.
변경 0은 원래 4줄, 두 여백 대조군은 Native240에서 3줄로 바뀌었다.

같은 편집본을 새 MCP 2020 Print로 각각 출력했다. 두 출력 모두 9쪽의 81번
문단을 4줄로 유지했다. PDF의 실제 글자 원점을 합친 기준선은 +750에서
422.9·452.9·482.8·512.9px, +1500에서 433.0·462.9·492.9·522.9px다.
앞 여백으로 원점이 이동해도 첫 줄의 Square 띠 소속은 사라지지 않는다.
+1500의 독립 Print는 전체 18쪽으로 바뀌므로 전체 쪽수를 과거 17쪽에 고정하지 않는다.

- +750 MCP job: `f9fb8704-89d4-4c83-b92c-f5d9bb5d6d34`, PDF SHA-256
  `eb6d71e41df2f419506c8b9a343654ce22b8014efef30e628b02f8cad4be9b7a`.
- +1500 MCP job: `df49aef9-ab48-4b4a-b792-ccccc2bb5da0`, PDF SHA-256
  `c6637def9fdb63b9f2cd7034ab6953413cd6ef2ffa6508aafc1727ff2d496d7c`.
- 실제 엔진: `11.0.0.9136`, 두 출력 모두 `pdf_print_method=0`,
  `hancom2020_pdf_driver_one_up`. 원본은 바꾸지 않았다.

`wrap_match::classify → wrap_flow의 prefix format → suffix_metrics →
commit_wrap_absorption/commit_wrap_tail → layout_wrap_around_paras/부분 문단`을
대조했다. 실제 활성 Square 띠를 prefix format에 전달하지 않는 것이 원인이라는
가설로 known-square-band 경로를 실험했으나, Native244에서도 같은 실패가 남아
이 가설을 기각하고 해당 실험 변경을 제거했다. 올바른 기존 관계 검사는 유지한다.
독립 Print·현재 전쪽 TSV와 확대 판독을 완료하기 전 새 픽스처나
회귀를 등록하지 않는다. 임시 편집본·Print·로그는 `margin-7158-3880/`와
`mcp-7158-margin{750,1500}-3881/`에 있다.

## #8228 병합 후 리베이스와 재검증

사용자 지시에 따라 #8228의 머지 `3e02fe2749866a8453af3eb3901b9c1a674cccaf` 위로
기존 320개 커밋을 리베이스했다. 이전 HEAD는 `dd25d5e71dec421399629cbb60aec883ef468bdd`,
리베이스 직후 HEAD는 `b90b0741b69c2da4eddb03552d998848fbc5e251`이다.
미커밋 원점·Square 보정과 이 기록은 별도로 보존한 뒤 복원했다. 충돌은 두 동작을
함께 유지하고, 양립하지 않는 기존 프레임 판정은 사용자 지시에 따라 #8228을 우선했다.

전체 fallback 별칭·원 프로그램 굵기, 수학 기호·보충 문자 메트릭, 명시적 전체/부분
문단 소유, 재조판 셀 프레임 무효화, 완결 셀 슬롯과 첫 조각 바깥 여백을 함께 보존했다.
현재 글줄 메트릭 투영은 #8228의 구판 HWPX 여백 문맥을 유지한 단어 꼬리 투영으로 전달한다.
컴파일에서 드러난 다섯 호출 호환성 오류는 소유한 표·셀의 재조판 상태와 수용 구성을
실제 호출부에서 전달하는 방식으로 보정했다. 기대값·허용치는 변경하지 않았다.

삭제된 sample16·편람 검사는 복원하지 않았다. 실제 Rust 검증 worktree에서 파생 참조
1,770개를 확인했고, 삭제된 검사 52개로의 참조·누락 원본·sample16 이름 참조·삭제 파일
복원은 모두 0건이다. 남아 있는 정상 소스만 suite에 연결했다. 파생 파일은 PR에 포함하지 않는다.

Native243 빌드는 `target/pr-review`, `CARGO_BUILD_JOBS=8`과 `release-test`를 사용해
265.80초, exit0으로 종료했다. 빌드 중 소스 불변을 확인했고 실행 파일·라이브러리를 보존했다.
Studio 글꼴·AI·패널·장치 최소 선폭 집중 29건은 모두 통과했다. 새 잠금 파일 의존성을
설치한 뒤 TypeScript 검사도 exit0이다. 현재 전쪽 Native 비교와 Rust 집중 검사를 실행하며,
이전 fresh WASM068 결과를 리베이스 후 후보의 검증으로 재인용하지 않는다.

이관 #8230은 사용자 대기 요청 전 생성·#7445 하위 연결까지 된 상태다. 활성 검사 삭제나
추가 이관은 하지 않았으며 새 기준의 결과를 먼저 재검토한다. Native/fresh WASM의 새 기준
대조·쪽수·직접 판독과 전체 필수 게이트 전에는 PR 준비 완료로 판정하지 않는다.

로그와 입력·출력 해시는 ignored `output/pr-review/semanticist21-20261019/`의
`rebase-8228-3899/`, `rebase-native243-build-3904-result.json`,
`logs/rebase-8228-focused-rust-3906.log`, `logs/rebase-8228-visual-3907.log`에 기록한다.

### 리베이스 후 첫 실행의 실제 판정

- Rust 집중 검사: 144건 중 142 PASS·2 FAIL, exit100. 기존 ClickHere의 80행
  물리 분할 검사와 #7158의 이동 앵커 검사가 남았다. #7158 known-square-band 변경만으로
  실패를 해소하지 못했으며 해당 변경은 실험 중인 미커밋 변경으로 남긴다.
- Studio 전체 단위 검사: 2,597 PASS·실패 0, exit0.
- Native 12조건·26쪽: #1510 HWP/HWPX·#1535·#6133은 전쪽100%, #6970은
  3쪽 최저 98.22185%. #8228의 field-reset·field-plain·saved-host·omitted-after·
  symbol-slot 두 Print 조건도 전쪽100%다.
- #8228의 `titled-host.hwpx`는1/3쪽43.94386%/40.24697%로 후퇴했다.
  2/4쪽은100%다. 새 PR·이관의 완료 조건을 충족하지 못했다.

동일 입력을 Native240·241·243으로 대조한 결과 T3/T4와 뒤 본문 원점은 같다.
따라서 이번 #6970의 공통 첫 줄 원점 적용이 이 차이의 원인이라는 가설은 기각했다.
#8157·#8159의 원 검토 기록과 독립 Print의 실제 선 좌표를 읽었다. K1의 T3 상단은
356.569 CSSpx인데 현재492.9px다. 앞 문단 오른쪽 T2와 뒤 문단 왼쪽 T3는 세로 구간이
겹쳐도 독립 Print에서 보존된다. 본문의 배제 띠와 수용된 흐름 표의 원점을 같은 조건으로
다시 미는 경로를 우선 대조한다. 현재144건에는 #7687 모듈 자체가 빠져 있었으므로
후속 집중 실행 계획에 기존 두 검사를 명시적으로 추가했다. 이 범위를 통과로 확대하지 않는다.

### 2026-10-26 흐름 표 원점 보정의 Native 중간 확인

`HostPlacementConstraint::constrain`에서 이미 흐름이 소유한 `CoanchoredTable`을
본문 배제 띠로 다시 밀지 않도록 했다. 같은 문단의 앞 표가 소유한 흐름 바닥값은 유지한다.
수식·문서명·페이지 번호·좌표 예외를 추가하지 않았다. 현재 계산 흐름 소유와 떠 있는
개체를 피해야 하는 본문 줄의 조건을 구분한다. #7687과 #1510·#1535·#6133을 함께 대조했다.

Native244 빌드는271.01초·exit0·소스 불변이다. 주 작업공간과 검증 worktree의 실제
런타임1,604개 경로가 같은지 확인했다. 후속 Native12조건·26쪽은 모두90% 이상이며
최저 98.22185%다. `titled-host`는1/3쪽을 포함해 전 4쪽100%로 회복됐다.
3쪽의 canonical review·독립 overlay를 직접 열어 T3·T4와 뒤 본문 원점을 확인했다.

기존 #7687 검사를 실행하면 K1의 실제 배치 실패는 사라졌지만 K2 첫 글줄 기대값에서
실패한다. 원본 XML의15~18번 문단은 `K2 제목`, `K2 글 1`, `K2 글 2`, `K2 글 3`이며,
27~30번은 같은 K4 순서다. 독립 Print의 K2 첫 본문 위끝은696.377 CSSpx이고
Native244의16번 문단은696.7px다. 기존 검사는17번·29번의 **두 번째** 본문을 첫 본문으로
지정했다. 같은 문제는 수정 전 Native243의 K2에도 있지만 K1에서 먼저 실패해 드러나지 않았다.

정식 검사·기대값은 아직 바꾸지 않았다. ignored output의 임시 검사 사본에서 문단 소속을
16/28로 교정한 진단은48/48 PASS다. 교정 전 정식 소스 재실행은47 PASS·1 FAIL이므로
둘을 구분한다. 이 진단은 전체 nextest나 fresh WASM 수용 증거를 대체하지 않는다.
전쪽 fresh WASM·직접 판독 선행 조건을 마친 뒤 필요한 정식 검사 교정을 진행한다.
기존 ClickHere·#7158 실패와 이관 대기는 계속 남아 있다. PR 준비 완료로 판정하지 않는다.

원본·진단 소유 비교는 `rebase-8228-3899/titled-host-diagnostic/`, 전체 Native TSV는
`remaining-visual-3911/`, 대표 그림은 `rebase-8228-3899/titled-host-after/`에 있다.
`rebase-host-flow-3910-result.json`은 정식 소스 재실행,
`rebase-host-flow-3912-result.json`은 임시 문단 소속 교정 진단이다.

## 원점 이동과 문자 경계의 출처 분리

2026-10-26 확인 범위를 현재 보정 대상과 #8228 리베이스의 기능 보존으로 고정했다.
다른 HWP 파일로 확대하지 않는다. 삭제된 sample16·편람 검사와 자동 수집 참조도
다시 생성하지 않는다. 검증 worktree의 실제 파생 참조 1,770개에서 삭제·누락 경로의
참조는 0건이며, 새 base에서 이미 삭제된 경로도 복원하지 않았다.

#7158의 실제 원인은 `compute_render_normalized`의 인라인 상자 높이 투영 뒤에 있었다.
앞 표의 바깥 여백이 커지면 후속 문단은 세로 원점만 이동하지만, 이 경로가 모든 줄에
`TAG_IMPLEMENTATION_PROPERTY`를 붙이고 `publish_object_projection`을 호출했다.
그 결과 문자 경계가 그대로인 4줄을 `para_has_no_stored_line_segs`가 NO_LS로 판단해
일반 전폭 3줄로 다시 채웠다. 원점만 바뀐 문단의 공백 메트릭·문자 축도 재발행됐다.

소비 경로는 `compute_render_normalized` → `recompose_stored_lines_in_frame`의
`fresh_body` → `wrap_tail::suffix_metrics` → `wrap_absorption::prefix` →
`commit_wrap_tail` → 부분 문단 paint다. 지역 재조판만 막은 Native245 진단은 4줄을
복원했지만, 합성 태그를 제외하는 prefix 끝점 계산에서 원점을 잃어 첫 줄 간격이
5.47px로 줄었다. 이 지역 우회는 제거했다.

최종 보정은 생산 지점에서 세로 위치만 이동하고 기존 줄 태그·문자 경계·가로 폭·공백
메트릭을 유지한다. 세로 위치 snapshot만 무효화한다. 문자 편집의 무효화 계약,
실제 새 줄 생성의 구현 태그, 셀의 개체 투영 계약은 그대로 소비한다. 문서별 수치나
출력 좌표를 추가하지 않았다.

Native246 빌드는 251.50초·exit 0이고, 원래 #7158 검사 3개는 모두 PASS다.
이동 앵커의 4줄 소유·저장 줄 간격·후속 문단 이동과 기존 13개 이어받기 문단을
같은 소스로 검사했다. 테스트 함수·기대값·허용치는 바꾸지 않았다. Native245의
2 PASS·1 FAIL과 구분한다. 로그는 ignored 경로의
`logs/rebase-native246-build-3921.log`, `logs/object-projection-wrap-after-3922.log`다.
전쪽 Native/fresh WASM·새 base 비교와 필수 최종 검증은 이어서 완료한다.

동일 Native246의 현재 대상·리베이스 대조군 12조건·26쪽은 전쪽 90% 이상이며
최저 98.22185%를 유지했다. #7158의 기존 원본 17쪽과 이미 만든 두 편집 대조군도
별도로 측정했으나, 원본 최저 56.45846%, +750 최저 58.30695%, +1500
최저 58.13220%로 미달했다. 관계형 3 PASS를 시각 통과로 해석하지 않는다.
새 base의 같은 입력·Print 대조로 기존 문제와 이번 수정의 후퇴를 구분한다.
사용자 지시에 따라 다른 HWP로 범위를 확대하지 않는다. 전쪽 원문은
`remaining-visual-3925-result.json`과 해당 폴더의 Native TSV에 보존했다.

동결한 원 PR 45개도 재조회했으며 head 변경은 0건이다. 기존 개별 리뷰 8건 외 37건은
원문 주장·적용 커밋·현재 검사 소스를 연결한 임시 초안을 output에 준비했다.
실제 최종 검증과 직접 판독 전에는 승인이나 개별 리뷰 완료로 판정하지 않는다.

## #6970의 배제 경계와 Native/fresh WASM 차이

새 base `3e02fe2749866a8453af3eb3901b9c1a674cccaf`의 Native247과 후보
`7e0be68ddc200bde3a29414deb3c0e870b8db357`의 Native246을 같은 입력·Print로 대조했다.
#6970의 2쪽은 98.37264% → 98.22185%로 0.15079%p 하락해 작업지시자의 0.1%p 기준을
초과했다. 다른 위치·구조는 같고 235개 노드 중 90번 문단의 한 TextRun 폭만
305.1px → 306.2px로 달랐다. 독립 Print의 한글 전진 12.96px는 후보의 자연 전진과
맞으므로, 과거 말미 공백 압축을 복원해 점수 하락을 숨기지 않는다.

fresh WASM069의 실제 전쪽 값은 100%·63.13384%·91.53306%다. Native의 통과를
fresh 통과로 해석하지 않는다. WASM은 같은 90번 문단의 마지막 `롽`을 셋째 줄로
이월해 후속 문단을 16px 이상 밀었다. 다른 HWP를 추가하지 않고 같은 3쪽을 대조했다.

원인은 `synth_square_wrap_rects`가 원본 HWPUNIT 바깥 여백을 이미 포함한 사각형을
발행하는데, `section/flow::place_paragraph_flow`가 좌우 감폭과 줄 단위 감폭에
고정 3.8px을 다시 더한 데 있었다. `WrapAnchorRef`의 cs/sw →
`synthetic_wrap_column_width`의 측정 폭 → `paragraph_layout`의 `line_cs_offset`과
가용 폭으로 이어져 양쪽 backend가 같은 좁은 경계를 소비했다. Native와 WASM의
폭 판정 잔차가 이 잘못된 경계에서 서로 다른 줄 수로 드러났다.

보정은 원본 여백을 포함한 배제 사각형의 좌우 끝을 그대로 사용한다. 추가 상수·문서
식별자·출력 backend 분기는 없다. 새 base WASM070은 사용자 중단 뒤 실제 핸들·PID가
모두 사라진 것을 확인했으며, 완료로 재사용하지 않았다. 수정한 Native248을 먼저
확인하고 필요한 fresh WASM과 새 base 비교를 순차로 실행한다.

#7687은 같은 기존 입력의 전 4쪽이 Native246/fresh069 모두 100%이며, fresh 2쪽의
review·overlay에서 T3/T4·첫 본문·뒤 두 본문 순서를 직접 판독했다. 잘못된 문단 17/29
지정은 16/28로 고치고, 새 이름의 `verified_titled_host_first_body_ownership`에서
실제 `K2/K4 글 1` 내용과 뒤 `글 2/3`의 순서를 추가 보호한다. 기존 빈 host H1~H6 검사는
유지하며, 올바른 수치 기대값·허용치는 바꾸지 않는다. 기존 범위의 두 검사 중 잘못된 검사 하나를 새 이름으로 분리했으며 신규 HWP/PDF는 추가하지 않았다.

#7158의 기존 min HWPX는 BinData 23개 중 11개 JPEG가 동일한 332바이트로 들어 있으며
PIL 확인과 Canvas decode가 실패했다. 원본·기존 독립 PDF와 실패 증거는 보존한다.
현재 관계형 3 PASS와 전쪽 시각 미달을 구분하고, 새 base 확인 없이 검사를 이관하거나
해당 문서를 시각 완료로 선언하지 않는다. 확인 파일은 ignored 경로의
`7158-image-entry-audit-3939.json`, `6970-native-wasm-node-difference-3940.json`,
`titled-host-test-correction-3944.json`과 해당 실행 로그다.


### 고정 간격 제거 후 확인

Native248은 291.07초·exit 0으로 빌드됐으며, 기존 경계와 새 #7687 이름의 검사
총 51건이 모두 PASS다. #6970 전 3쪽 raw는 99.85355%·99.79447%·99.90413%다.
새 base Native247의 99.85351%·98.37264%·99.90410%와 대조해 각 쪽을 유지·개선했고,
0.1%p 초과 하락은 없다. 다른 기존 대조군도 전쪽 100%를 유지했다.
새 fresh WASM의 같은 범위 확인과 필수 최종 검증은 계속 진행한다.

새 #7687 검사의 K1 assertion 블록은 이전 검사와 바이트가 같다. Native243에서
그 K1 배치 결함을 실제 검출했던 기존 로그와 연결하고, Native248의 동일 경계 PASS를
확인했다. 오래된 rlib의 별도 재실행 시도는 의존 crate를 찾지 못해 컴파일 단계에서
종료했으므로 제품의 보정 전 FAIL로 인용하지 않는다. 기존 확정 실행·동일 assertion의
연결은 `titled-host-before-after-proof-3949.json`에 보존했다.

실제 검사 로그는 `logs/rebase-boundaries-native248-after-3945.log`이며, 전쪽 Native
원문은 `remaining-visual-3946/`의 TSV와 `remaining-visual-3946-result.json`에 있다.

### 최신 fresh WASM과 삭제 범위 확인

#8231의 문서·증적 변경을 반영한 base `238de6de41a208648f4ade6b494fe433a81c1fce`로
리베이스했다. 후보 `9504cf250a211077a2f6c7f9137a890f8f5db8eb`는 기존 324개 커밋을
보존하며, 리베이스 전 후보와 Rust·Studio 실행 소스의 차이는 없다. Native248의 실제
소스 일치와 별도로, 같은 후보에서 표준 release·wasm-opt를 실행한 fresh WASM071을 만들었다.
절대 `--out-dir` 때문에 wrapper의 Studio 자동 복사가 생략된 뒤 검증 driver가 실패한 것은
빌드 실패와 구분한다. 실제 빌드 exit 0 이후 동일 바이트를 Studio에 복사하고 pkg·public·
고정 package의 SHA-256 일치를 재확인했다. 최초 driver의 exit 1은 기록에서 지우지 않았다.

기존 대조군 12조건·26쪽의 fresh WASM071 전쪽 TSV는 모두 raw 90% 이상이다.
#6970의 각 쪽은 100.00000%·95.39371%·91.53306%이며, 잘못된 배제 경계로 3행이 되던
2쪽은 fresh WASM069의 63.13384%에서 개선됐다. 1·3쪽의 실제 PNG는 보정 전과 SHA-256이
같다. 2·3쪽의 review와 standalone overlay를 함께 직접 확인했고, 3쪽의 기존 글자 폭 잔차를
새 보정의 개선으로 보고하지 않는다. 새 base fresh WASM070과의 페이지별 하락 대조는
별도 실행 중이며, 아직 기능 보존 검증 전체가 완료됐다고 판정하지 않는다.

같은 package를 사용하는 Studio `npm run build`와 실제 브라우저의 command-error 계약
검사는 각각 exit 0이다. 브라우저 로그의 주입된 명령·커서 오류는 오류 복구 검사의 입력이며,
성공한 종료 결과와 분리해 해석한다. 실행 명령·결과는 `studio-current-3960-result.json`,
전쪽 TSV는 `remaining-visual-3955-result.json`, 이미지 비교는
`6970-fixed-boundary-wasm-review-3961/`, 전후 PNG 해시는 `6970-before-after-png-3963.json`에 있다.
모두 앞서 지정한 ignored 검증 경로 아래에 보존하며 실행 로그는 커밋하지 않는다.

검증 worktree의 Rust 소스 1,601개가 후보와 같은 것을 확인하고, 최종 형식의 새 #7687 검사만
동기화한 뒤 그 worktree에서만 `--prepare`를 실행했다. suite의 파생 참조 1,771개에서 삭제된
검사·누락 소스·sample16·편람 모듈 참조는 0건이다. 주 작업 트리의 파생 파일은 재생성하거나
stage하지 않았다. sample16·편람의 기존 삭제와 자동 수집 제외를 유지하며, 해당 문서의 검사나
별도 보관본을 복원하지 않는다. #7158의 이미지 디코딩 실패와 ClickHere 축약 입력의 미달은
위 12조건의 통과와 구분하고 새 base 근거를 확인한 뒤 기존 처리 절차를 이어간다.

### 새 base 대조와 WASM 원문 주소 검사 보정

base fresh WASM070은 표준 release·실제 wasm-opt를 거쳐 1,051.54초, exit 0으로 종료했고
빌드 중 소스 불변을 확인했다. Native247/248과 fresh WASM070/071의 기존 12조건을 각각
대조했다. 후보의 전 26쪽은 양쪽 모두 90% 이상이며, 직접 대응 가능한 페이지 중 작업지시자의
0.1%p 기준을 넘는 하락은 0쪽이다. #6133은 base의 1쪽과 독립 Print의 2쪽이 달라 baseline
TSV를 만들지 못했다. 후보는 두 backend 모두 독립 Print와 같은 2쪽·raw 100%로 개선됐다.
미산출 baseline을 임의 수치로 채우지 않는다. `rebase-page-preservation-3974.json`에 원본·
PDF SHA와 페이지별 전후 값을 연결했다. #7158의 세 조건은 양쪽 WASM에서 같은 이미지
디코딩 실패가 재현됐고, ClickHere는 base 쪽수 불일치와 후보 시각 미달을 별도로 기록했다.

기존 WASM 글꼴 trace E2E는 공개 런의 `charStart + local`을 무조건 확정 원문 주소로 보아
실패했다. 같은 범위의 Native 검사는 이미 실제 원문 문단과 글자 겹침 이후 경계로 소유권을
판정한다. 원본 CFB의 `BodyText/Section0`를 독립적으로 해제·레코드 해석한 결과 문단 5의
선두 `0x0017`과 `tcps` 개체를 확인했다. 추정 주소를 확정하지 않는 현재 trace의 null 반환이
의도된 계약이며, 원문 경계를 무시한 WASM 기대값을 보정했다. 새 HWP나 renderer 변경은 없다.

새 원본 경계 fixture는 입력 SHA·문단 텍스트 및 개체 헤더의 실제 바이트를 고정한다.
WASM 검사는 해당 경계에서만 `unavailable`·null을 요구하고, 나머지 일반 본문은 `complete`와
정확한 원문 위치·구역·문단·중첩 소유 관계를 모두 검사한다. 선언한 경계의 실제 실행도 확인해
전부 null인 출력으로 통과할 수 없다. 같은 새 검사에서 base는 추정 주소 0을 게시해 FAIL,
현재 후보는 2 PASS / 0 FAIL이다. `font-trace-source-contract-3975-result.json`에 원본 바이트,
수정 전후 및 base의 실제 로그·검사 SHA를 연결했다. 검증된 code와 별개인 전체 Rust·lint·
개별 PR 최종 판정은 해당 실행을 마친 뒤 확정한다.

Native Clippy는 중첩 표의 `nested_y` 선택에 같은 `treat_as_char &&
table_host_line_only_fragment` 분기가 두 번 있음을 검출했다. 앞 분기와 뒤 분기가 호출하는
원점 계산도 같고, 조건이 참이면 앞 분기에서 이미 끝나므로 뒤 분기는 실행 불가능하다.
앞 분기의 저장 줄 소유·원점 소비를 유지하고 뒤 중복만 제거했다. 기대값·허용치·입력은
바꾸지 않았다. 재실행한 형식 검사와 Native Clippy는 모두 exit 0이며, 원 실패와 재검증 로그는
각각 `logs/final-rust-native-clippy-3973.log`와 `logs/final-rust-native-clippy-3976.log`다.

### 공통 제외 helper와 최종 suite 준비

WASM Clippy와 workspace build는 exit 0이다. all-target Clippy가 발견한 공통 제외 helper의
중복 모듈 로딩은 자동 suite 안에 여러 기존 검사 모듈이 함께 묶이는 경로의 문제였다.
같은 helper 파일을 외부 `mod`로 거듭 로딩하던 세 소비자를 private inline 모듈의 `include!`로
바꿨다. helper 함수 본문과 제외 목록의 바이트는 그대로이며, sample16·편람의 제외 범위도
유지한다. #1949의 숫자 구분자 한 곳은 값 변경 없이 표기만 바로잡았다.

보정 후 형식 검사와 workspace all-target Clippy가 모두 exit 0이다. helper 길이 변경으로
달라진 파생 harness 배정은 검증 worktree에서만 `--prepare`로 갱신했다. 이후 고정 base의
suite policy와 unit tier policy도 각각 exit 0이다. 원본 함수 본문 동일성·제외 목록 SHA와
파생 참조 1,771개 중 삭제된 검사·sample16·편람 참조 0건은
`rust-test-preparation-3984-result.json`에 연결했다. 주 작업 트리의 파생 파일은 생성하거나
커밋하지 않았다. 정책 실패를 이유로 삭제된 소스나 예전 참조를 복원하지 않는다.

workspace 검증 뒤 공간 부족은 종료된 중간 라이브러리 복사본·과거 WASM package와
24시간 이상 지난 미사용 WASM 증분 캐시 한 곳만 정리해 4.09GiB를 확보했다.
열린 파일·메모리 매핑·worktree symlink 참조를 먼저 확인했으며, 현재/기준/보정 전 생산자와
입력·PDF·TSV·PNG·로그 및 target의 deps·실행 파일은 보존했다. 정확한 삭제 목록·해시는
ignored `disk-cleanup-3985.json`에 기록했다.
