# #7998 P4 열두 번째 절편 — 각주·미주·바탕쪽의 클립 저장 소유

2026-10-19 다음 절차 지시에 따라 승인된 `62a820490` 뒤에서 계속한다.
범위는 원래 TextBox/Paragraph를 다른 실제 소유 문단 목록에 둔 경우의 rhwp
저장·재열기 보존이다. 회전/resize·캡션·한컴 native 비사각 가림은 별도 경계로 둔다.
전체 Rust/lint/package·최신 devel 비교와 원격 제출은 P5 단계에 유지한다.

수정 전 계약: 실제 문단 목록의 source clip/원문/ID는 두 writer가 같은 소유자로
보존해야 하며, 본문과 다른 목록에서 같은 ID를 발견하면 결합을 거절해야 한다.
현재 `text_clip::collect/visit`은 Header/Footer까지만 순회하고 각주·미주와
SectionDef의 canonical master 목록을 방문하지 않는다. Section에는 authoritative
`section_def`와 본문 Control::SectionDef의 복제 표현이 있어, 둘을 무조건 방문하면
같은 바탕쪽을 두 번 세게 된다. 기존 serializer·identity 순회와 실제 저장/로드를
대조하고 source의 유일 소유와 실제 paint 경로를 먼저 실행한다.

원시 증거: ignored `output/pr-review/issue7998/p4-text-clip-lists/`.

## 실패 전후와 실제 소비

공개 `pdf/issue7998/text-clip-placement/sources/first-hole.pdf`의 원문 MERGED·clip·
글꼴/metric을 그대로 가진 TextBox를 각주/미주 또는 바탕쪽의 실제 문단 목록에 둔다.
임의 LineSeg/vpos는 작성하지 않는다. 원래 Paragraph/Shape를 보존하며, 두 쪽 대조군은
제품 `insert_page_break_native`로 실제 쪽 나누기를 넣었다.

`probe-before.log`·`before/result.json`에서 세 소유자의 live IR clip은 1개지만 두 저장
형식 재열기의 IR clip은 모두 0개였다. 미주·바탕쪽의 live paint에는 clip이 1개였다.
`probe-after-final.log`·`after/result.json`에서는 각주/미주/바탕쪽과 추가 진단 경로 모두
두 writer를 거쳐 원래 IR clip 1개를 보존한다. `before-visual-results.json`의 같은
`master-hole.hwpx`는 이전 immutable CLI에서 83.33333%, 현재 CLI에서 100%다.
현재 코드에서 만든 입력을 이전 코드로 읽어도 실패하므로, 생성본의 외형을 정답으로
삼거나 helper 내부 결과만 재인용한 증거가 아니다.

| 경계 | 생산 → 저장/로드 → 실제 소비와 판정 |
| --- | --- |
| 각주·미주 소유 | PDF frontend의 source-local `TextBox.paint_clip` → `text_clip::encode`의 collect → CFB/HWPX writer의 보조 stream/entry → parser 마지막 decode의 visit → 같은 각주/미주 문단의 TextBox. 실제 두 adapter에서 원문·경로·winding/ID를 보존했다. |
| 바탕쪽 소유 | `Section.section_def.master_pages`의 canonical 목록을 collect/visit의 기존 전역 ID 공간에 추가한다. 본문 `Control::SectionDef`는 그 복제 표현이므로 별도 방문하지 않는다. 이중 방문으로 정상 ID를 충돌로 만들지 않으며, 다른 실제 본문/각주 소유자와의 중복은 거절한다. |
| 최종 바탕쪽 paint | canonical master → `layout.rs::build_master_page`의 물리 용지 원점 → `layout_textbox_content`의 source HU→page px clip → 공통 Layer ClipPath → Native Skia/WebCanvas/실제 Studio CanvasKit. 같은 모델 소유자 하나를 두 페이지에 반복한다. 표시 횟수를 모델 중복으로 잘못 세지 않는다. |
| 원점/측정 | 이번 변경은 metadata 문단 목록 순회다. 글줄 높이·bbox·정렬·분할 컷·회전·가림 경로 값은 바꾸지 않는다. stage12의 실제 내용 이동 공통 결과도 그대로 소비한다. |

바탕쪽 대조군과 정식 검사는 canonical 목록 외에 **실제 `Control::SectionDef` 복제도
명시적으로 둬서** 같은 ID를 두 번 세지 않는지 실행한다. 최초 PDF import에는 이 복제가
없었으므로, 초기 authoring 결과를 복제 존재의 증거로 오인하지 않고 보완했다.
그때의 자료는 `pre-mirror-inputs/`·`pre-mirror-native/`에 보존하고 최종 입력으로 다시
Native/fresh WASM을 출력했다. 제품 source는 그대로라 빌드는 반복하지 않았다.

## 빌드·시각 자료와 예외

제품 변경은 `src/model/text_clip.rs` 한 파일이다. depth/record/byte 제한과 private
metadata version은 바꾸지 않았다. `source-runtime.json`의 2,796개 runtime 파일과
아래 빌드를 고정했으며 이후 추가는 검사/문서뿐이다.

| 대상 | SHA-256 |
| --- | --- |
| runtime manifest | `1bd10e6713a5670f0f1e586e802e41a6bc1836b2ef881225e72c7631d39f1af7` |
| immutable Native | `545de67902c7745ad65532218e825666ab5e6801b3ac493b0a4619482e2e793f` |
| fresh Docker WASM, 19,365,088 bytes | `63e0cc4ce5444a83ef87303ffcccad86db5d0bcb61345e9ec7c27084ff49752c` |
| JS glue, 476,233 bytes | `2f715f0325d1292c52a84165d08477868c979eb94900174231a6a02aa6a8de11` |

Native는 기존 shared target의 `--locked --profile release-test --features native-skia
--bin rhwp`로, WASM은 기존 `rhwp` Docker Compose project/named cache로 만들었다.
`native-build.log`·`docker-wasm.log`·`build-artifacts.json`에 결과를 보존했다.
JS glue는 직전 절편과 바이트가 같으며 실제 Studio가 새 WASM SHA를 instantiate한
것을 따로 확인했다. 메인 Studio 7700은 변경하지 않는다.

바탕쪽 기준 PDF는 기존 공개 한컴 Print에서 유래한 같은 PDF paint를 **물리 용지
원점 그대로** 사용한다. 두 쪽은 같은 페이지를 반복한다. 정상 unclipped 입력만
W/W*를 명시적으로 제거한다. `generate_masters.rs`·`contexts-origin.json`·
`reference-origin.json`의 생성 과정/해시를 보존했다. 이것은 한컴이 native 비사각
clip을 저장·재현했다는 증거가 아니다. native-format 미지원은 content-loss에 남긴다.

| 이번 최종 전쪽 비교 | Native | fresh WASM |
| --- | --- | --- |
| master-hole·두 쪽 반복·unclipped: 9건/12쪽 | 최저 99.78308%, 미달 0 | 최저 99.79798%, 미달 0 |
| master-curve: 세 형식 각 1쪽 | 77.77778% | 77.50000% |
| master-parent: 세 형식 각 1쪽 | 0% | 0% |
| 전체 원시 합계: 15건/18쪽 | 최저 0%, 미달 6쪽 | 최저 0%, 미달 6쪽 |

`native/visual/*/silhouette.tsv`, `wasm/visual/*/silhouette.tsv`는 원시 값과 미달을
그대로 남긴다. raw gate를 통과로 바꾸지 않는다. 각 쪽수는 기준 PDF와 같다.
compat2022·96dpi·기존 Arial 공급·PNG 조건의 실제 명령은 `validate.py`와 각 결과 JSON에
기록했다. 대표 `review/{native,wasm}/master-hole*`에 compare/overlay/review를 생성하고
같은 원점의 4배 확대 crop과 실제 원본 페이지를 판독했다.

곡선의 작은 노출 영역은 PDF 잉크 19픽셀/Native 40픽셀, parent 교집합은 PDF에 회색
2픽셀/Native 0픽셀이다(`<245`, source 원문은 모두 보존). 같은 PDF의 **직접 열기**도
이전 `62a820490` CLI와 현재 CLI의 Native PNG가 바이트까지 같다. 현재 저장 보정의
회귀로 분류하지 않으며 정확한 원인은 미확정이다. 2026-10-19 작업지시자가
**“저장 보존 범위 예외·경계 차이는 백로그”**로 결정했다. B21에 source·해시·실패값과
원본/이전/현재 그림을 연결한다. 원본 path를 이동하거나 픽셀을 감춰 수치를 맞추지 않는다.
해당 두 입력을 새 rendering golden/px fixture로 등록하지 않았다.

이전 116개 입력의 HWPX에서 이번에 새로 방문하는 note/master 목록이 없음을
`prior-scope-audit.json`으로 확인했다. 직전까지 검증한 381쪽/backend의 증거를
재사용하며 **이번 source의 전쪽 재출력이라고 합산하지 않는다.** 최종 devel 대비
전체 재검증은 P5에 유지한다.

## 실제 Studio·양 저장 형식과 정식 검사

`studio-result.json`은 실제 CanvasKit Studio 7793의 새 WASM SHA, page error 0과
5개 master 입력의 로드·두 export/reload·본문 X 삽입/삭제·초기 clip 복원을 기록한다.
한 canonical master가 두 쪽에 반복되는 경우도 확인했다. 바탕쪽 내부를 직접 타이핑한
검증과 구분한다. 각주/미주 2개도 실제 WASM 두 export를 만들고, `wasm-native-readback.json`
에서 실제 저장본 **24개**의 원문 MERGED 유일 소유·clip IR을 Native로 재소비했다.
fresh WASM TSV는 공통 layer WebCanvas이고, 실제 Studio CanvasKit 검증은 별도다.

`tests/cases/issue_7998_pdf_text_clip_owned_lists.rs`는 기존 공개 원본을 사용한
**model/metadata 보존 검사**다. 원문·경로와 winding의 실제 writer 왕복,
canonical/mirror의 구분·전역 중복 ID 거절·현재 문구가 다른 stale metadata 거절을
검사한다. 렌더링 좌표/숫자 점수/PNG hash를 기대값으로 등록하지 않는다.
자동 수집 suite를 prepare한 뒤 새 3건과 기존 `issue_7998_pdf_text_clip` 3건·
`issue_7998_pdf_text_clip_placement` 3건, **총 9건 통과**했다.
실제 명령/선택 건수는 `focused-final-results.json`에 있다. 초기 준비 검사와 최종
mirror 포함 검사를 구분하며, suite 준비 후 최종 소스로 실행했다.

진단 rustc는 공유 cache의 다른 serde_json variant와 연결되면 Serialize trait 오류가
발생했다. 진단 출력을 primitive/Debug로 좁혀 실제 제품 라이브러리를 그대로 재사용했다.
이 빌드 오류나 최초 mirror 없는 입력은 결함 재현/최종 통과 증거로 세지 않는다.

## 남은 결함과 완료 범위

각주의 직접 TextBox는 수정 전후 모두 live IR clip 1개지만 실제 paint에는 TextBox
원문이 나오지 않는다(Native/실제 WASM에서 확인). control-only 각주 문단의 번호/내용
소비를 별도로 추적할 **B22 실행 결함**이다. 단순한 metadata 보정으로 표시까지
해결됐다고 쓰지 않는다. 각주 TAC/표 시도는 사전 진단이며 독립 출력이 확인된 정상
fixture나 전체 지원 증거로 승격하지 않는다. 미주는 실제 paint clip이 살아남지만
전체 문서의 독립 한컴 Print 대비 시각 검증은 미완료다. 이번 바탕쪽 시각 판정으로
이 범위를 면제하지 않는다.

B22는 현재 PDF frontend가 note를 새로 발행해서 생긴 결함이 아니라, 공통 IR의
소유 목록 감사에서 드러난 기존 native 표시 결함이다. 발견한 세부 native 결함을
자동으로 이번 PDF 표/문단 작업의 다음 필수 구현 범위로 추가하지 않는다.
`E:\tmp\issue7998-text-clip-lists-review\diagnostic-notes\`에 같은 실제 진단 저장본도
복사했다. 바탕쪽 시각·구조의 직접 판정은 비교 화면/Studio 탭에서 요청한 상태이며,
판정 전 이를 최종 통과로 기록하지 않는다.

캡션·메모/숨은 설명 목록, 회전/resize·cache와 clipped table continuation,
한컴 native 비사각 표현/외부 재저장·strict glyph-only V2는 기존 B19 경계에 유지한다.
P4 전체 완료, P5 통합 검증/PR 준비/원격 반영 완료로 표시하지 않는다.
