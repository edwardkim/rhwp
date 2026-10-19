# #7998 stage28 — 병합된 셀 이웃의 경계 상속

2026-10-19 다음 절편 승인으로 착수했고 2026-10-20 검증을 이어간다. 기준 커밋은
`09bee1ff6c633e1be13a461212a870593a6204b9`다. 이번 범위는 **이미 colspan/rowspan이
있는 원본 표의 경계 소유와 행·열 편집**이다. 보존 paint 자체의 병합/나누기는 stage27의
원자 거절을 유지한다. R2·P5·B23 예외는 바꾸지 않는다.

## 규칙·원인·실제 소비

독립 한컴 3셀 표의 논리 셀에는 2구간짜리 긴 변과 실제 내부 경계가 없는 병합 영역이
있다. 기존 compound 가져오기는 1×1 셀만 수용해 실제 3셀 구조를 찾고도 소유를
발행하지 않았다. 구조 편집은 이를 소유 없는 일반 그림으로 보고 성공해, 경계를
새 셀에 상속하지 못했다. 새 `final/before-after.json`은 **같은 최종 PDF**를 실제
stage27 WASM과 현재 Native에서 열어 세 입력 모두 `3셀/소유 없음 → 3셀/소유 있음`을
확인한다. 새 Rust 검사 전체를 구버전 lib에서 실행했다는 주장은 하지 않는다.

- `parser/pdf/table_paint/compound.rs::plan`에서 blanket 1×1 조건을 제거한다.
  실제 segment의 셀 perimeter 피복, root/source closure와 자식 표 접촉 배제는
  유지한다. 병합 셀 내부를 가로지르는 잉크를 새 경계로 확정하지 않는다.
- `commands/border_paint_structure.rs::grid/side_keys`는 실제 Cell의 시작·끝 span과
  완전한 비중복 tiling을 검증하고 변의 물리 interval 집합을 만든다. `Proof::add`와
  `add_rectangle`이 같은 interval에 실제 source 잉크 피복을 증명한다. 4,096 슬롯·
  200,000 연산 등 기존 예산을 유지한다. 논리 행 컷을 병합 셀 내부 선으로 만들지 않는다.
- `border_paint_bands.rs`는 생존 Cell·모델이 실제 선택한 새 셀 template의 pen을
  옮긴다. 단일 셀 변 중간에 서로 다른 pen이 있으면 첫 구간/최댓값으로 합치지 않는다.
  편집 후 다시 tiling을 확인하고, 실제 외곽은 `row+row_span/col+col_span`으로 정한다.
  셀 안/밖 반쪽을 같은 폭으로 발행한다. 최근접 native BorderFill로 바꾸지 않는다.
- `ensure_banded_layout_grid`는 복제 후보의 **실제 HeightMeasurer → 최종
  LayoutEngine::resolve_column_widths/resolve_row_heights** 결과와 band의 선언 격자를
  비교한다. 실제 배치 `renderer/layout/table_layout.rs`의 root/depth 0·non-TAC 경로는
  같은 row resolver와 `relaxed_pad=false`를 소비해 누적 row_y를 만든다. 별도 중첩 fit,
  fragment/continuation 경로는 이 paper-anchored root 증명 범위가 아니다. 셀 줄/문단
  원점을 줄이거나 높이를 clamp해 동등성을 만들지 않는다. fresh engine으로 후보의
  pointer-keyed cache를 live engine에 남기지 않고, 발행 전에 실패를 반환한다.
- 성공한 후보만 원문 parent·BorderFill에 발행한 뒤 기존 reflow/event 경로로 이어진다.
  실패는 paragraph model·event·실제 SVG·HWP/HWPX를 그대로 보존한다. 문단/글꼴/저장
  LineSeg·pagination 규칙을 변경하지 않는다. 현재 source span의 소비와 단일 쪽 root
  배치 검증이며, 다쪽 continuation 전체를 검증했다고 확대하지 않는다.

## 입력·반례와 철회한 접근

새 입력은 공개 `pdf/issue7998/cell-regions/{colspan,rowspan}.pdf`의 텍스트·clip과
3셀 구조를 사용하되, 선 폭을 0.359→0.5pt로 바꾸고 독립 subpath를 compound stroke로
묶는다. 원래 Print의 join용 끝점 확장을 clip 축에서 명시적으로 끝낸 **별도 계약 PDF**다.
원래 Print와 바이트/픽셀 동일 또는 원래 overshoot도 편집 가능하다고 주장하지 않는다.

원래 399pt 용지에서는 열 추가로 표가 종이 밖으로 늘어나 오른쪽 문구가 잘렸다.
한컴 Print도 같은 상태였으나 이를 내용 보존 통과로 쓰지 않는다. `narrow-paper/`에
입력·출력·Print·판독을 보존하고 **셀 크기·글줄·본문은 유지한 600×360pt 용지**의 새
입력을 최종 대조군으로 만들었다. 편집 결과 30개도 그 정확한 새 바이트로 다시 Print한다.
같은 제품 Native/fresh WASM을 재사용하며 용지 변경의 영향 캡처만 갱신한다.

가운데 정렬한 원본 rowspan의 첫 줄 vpos와 spacingBefore는 각각 686HU다. 첫 후보는
행 위 삽입에 성공했지만 실제 줄 점유 때문에 행이 늘어 band와 달라졌다(91.97991%,
직접 판독 약 5px 차이). 새 빈 셀의 저장 줄 원점만 0으로 바꾼 후보는 해당 문단 간격과
불일치해 새 독립 Print 대비 75.97147%였다. `rejected-empty-origin/`과 patch를 보존하고
`Cell::new_from_template` 수정은 전부 철회했다. 현재는 입증되지 않은 행 편집을 원자
거절하며, 이 결과를 조판 일치 개선이라고 보고하지 않는다.

반례로 원래 정상 HWPX의 실제 셀 정렬 속성만 Top으로 편집해 새 한컴 Print를 만들었다.
그 PDF에서 같은 compound 계약을 파생한 `rowspan-top-retained.pdf`는 위 행 삽입과
끝 행 추가가 모두 실제 격자를 유지한다. 단순 rowspan blanket 거절과 구별한다.
`colspan-varying-side.pdf`는 긴 변의 절반만 0.9pt여서 단일 own pen으로 표현할 수 없다.

| 최종 원본 | 허용하는 편집 | 원문 보존 거절 |
|---|---|---|
| colspan | 위/아래/끝 행 추가, 왼쪽/중간/끝 열 추가, 원래 첫 행·첫/둘째 열 삭제 | 원래 둘째 행 삭제의 불일치 격자 |
| rowspan(원래 문단 간격) | 왼쪽/중간/끝 열 추가, 원래 첫 열 삭제 | 행 추가·행 삭제·원래 둘째 열 삭제의 격자/실제 높이 불일치 |
| rowspan-top | 위 행·끝 행 추가를 추가 대조 | 다른 미실행 조합을 완료로 세지 않음 |
| merged side 중간의 다른 pen | 없음 | 네 방향 추가·둘째 행/열 삭제 6건 |

## 검증 증거

안정 증거는 `/mnt/e/tmp/issue7998-stage28-evidence/`이며 ignored
`output/pr-review/issue7998/p4-merged-neighbor-inheritance/`에서 연결한다. **`final/`만
최종 제품 후보**이며 narrow-paper/철회 후보·첫 빌드의 점수를 최종 합계로 쓰지 않는다.

- 공유 `target/pr-review`에서 jobs 14로 `cargo build --locked --profile release-test
  --features native-skia --bin rhwp --lib`를 실행했다. Native SHA는
  `429dc330f5c1f81a3f3ea27294025acd840eafd981e48ff7fcec4f5df59310ce`다.
- canonical Docker `scripts/wasm-pack-locked.sh --target web --out-dir pkg`를 jobs 14·
  codegen-units 16으로 실행했다. fresh WASM 19,782,115바이트의 SHA는
  `269b7ef7a8c3d1c04da02f5dec1ef9ad793da3f84abc3e1bfea3ca9e15ce20c4`다.
  기존 glue `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`는
  동일하다. 제품 3개 소스·Native/pkg/public·실제 Studio 해시를 `source-runtime.json`에
  고정했다. 이후 제품 코드는 바꾸지 않았다.
- 기존 자동 수집 9개 case module의 집중 **80건 통과**(`guards.log`). 새 검사는
  전쪽 Native/fresh WASM과 직접 판독 후 같은 기존 case source에 새 3건을 등록했다.
- 실제 Studio의 성공 **15종 SnapshotCommand·undo/redo**와 양 저장 **30개 Native
  바이트 동일**, 미지원 **13종**의 render/event/양 저장 무변경을 확인했다
  (`studio-result.json`, `verify-studio.log`). 이는 모델/명령/직렬화 증거이며, 역편집 결과
  모두의 별도 한컴 Print를 확보했다는 뜻은 아니다.
- 최종 입력 4개는 `generate_public.py`로 별도 디렉터리에서 바이트까지 재현했다.
  공개 fixture/Print 출처·전쪽 TSV·대표 compare/overlay/review와 직접 판정은 아래에
  최종 결과를 연결한다. private 실물 PDF 두 개는 로컬 대조 전용이며 커밋하지 않는다.

## 최종 로컬 결과 — 2026-10-20 시각·편집 구조 통과

최종 Native/fresh WASM은 각각 **66입력·77쪽**이다. 범위는 기존 수용 24쪽·새 source
3종의 PDF/양 저장 9쪽·실제 편집 양 저장 30쪽·원문 보존 반례 1쪽·private 실물 2개
13쪽이다. 전체 raw 최저는 **97.57194% / 97.13390%**, 새 편집 30쪽은
**99.90587% / 98.85019%**다. 90% 미만·측정 누락·전체 쪽수 차이는 모두 0이다.
기존 수용본 24쪽과 실물 13쪽의 PNG **각 37쪽은 이전 같은 backend와 바이트 동일**하다
(`baseline-identity.json`). 최신 devel의 P5 비교로 바꾸어 부르지 않는다.

- 정확한 입력/기준 PDF·해시·명령·page gate는 `final/{native,wasm}/results.json`과
  `visual/<key>/{silhouette.tsv,silhouette_manifest.json}`에 있다. silhouette-only의
  `not_evaluated`는 자동 승인 판정이 아니며 다음 직접 판독과 구분한다.
- 실제 최종 HWP/HWPX **30개 각각**의 독립 한컴 Print를 확보했다. 600×360pt 원본
  용지·method 0·one-up·한컴 2020 11.0.0.9136이다. `remote-print/<filename>/`에서
  요청 때 고정한 입력과 최종 저장본 SHA가 같음을 확인했다. 이전 좁은 용지의 Print를
  새 저장본의 Print로 대신하지 않았다. 서비스 자격 증명/endpoint는 커밋하지 않는다.
- `E:\tmp\issue7998-merged-neighbor-review\index.html`에 Native/WASM의 같은 위치·배율
  compare·standalone overlay·review를 준비했다. 긴 merged 변·내부 선 부재·새 빈 행/열·
  원문 순서·삭제된 셀·종이 내부 문구를 직접 읽었다. `native-key-zoom.png`는 철회 전
  좁은 용지 진단이며, **`wasm-key-zoom.png`와 현재 gallery만 넓힌 최종 용지**다.
  Native 최종도 gallery의 같은 key에서 확인했다. 기존 Helvetica 대체·fine-line/글꼴
  smoothing 농도 차이는 남으며 수치만으로 새 결함을 면제하지 않는다.
- 같은 fresh runtime의 사용자 CDP 4개 새 탭에 실제 HWPX를 열었고, 모두 1쪽과
  5/4/5/5개의 실제 셀을 확인했다(`user-cdp-tabs.json`). 기존 탭·7700 서버는 유지했다.
  `issue7998-merged-neighbor-{colspan-row,colspan-column,rowspan-column,rowspan-top-row}`다.
- 전쪽 90%와 대표 직접 판독 후 기존 자동 수집 case에 관계형 **3함수**를 추가했다.
  source의 3셀/span·실제 네 변 소유, 지원 15종 편집의 본문 유일 소유·cell/paper 포함·
  양 저장 topology와 소유, 불일치/단일 변의 다른 pen 13종 거절의 원자성을 검사한다.
  새 3건 PASS(`formal-tests.log`), 기존 동일 제품 집중 80건 PASS(`guards.log`의 최초
  실행 기록)다. 80개 기존 본문은 변경하지 않아 다시 실행하지 않았다. `ci.yml` 호출·
  별도 runner/수집 경로를 추가하지 않았다. 공식 case의 소스/실행 바이너리 SHA는
  `formal-added.json`에 연결했다.
- 공개 입력 4개·정상 Top source HWPX/Print·편집 Print 30개·재현기·출처/입력 SHA·
  runtime·점수는 `pdf/issue7998/merged-neighbor/`에 보존했다. 생성기를 별도 출력으로
  실행해 네 입력의 바이트 동일을 확인했다. private 두 문서는 커밋하지 않는다.

2026-10-20 메인테이너가 **“시각 판정 통과 / 편집 구조 통과”**로 이번 절편을 수용했다.
판정 대상은 위 비교 화면과 실제 Studio 네 탭의 동일 source/runtime이다. 수용 기록만
갱신하며 제품 코드·검증 입력·공식 검사 소스가 같은지 해시로 확인해 빌드·검사·캡처를
반복하지 않았다. 현재 절편을 로컬 커밋으로 마무리한다. 현재 지원 격자의 상속 증거를 일반 병합 paint
재배분·지역/가림/dash/alpha·미증명 배치·R2/P5 전체 완료로 확대하지 않는다. 실제 줄
점유로 물리 격자가 늘어나는 행 편집은 원문 보존 거절이며 조판 일치 보정 완료가 아니다.
전체 suite·remote push·PR 상태 변경은 수행하지 않았다. `parser_allocation_bounds_proven=false`다.
