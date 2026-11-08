# #7998 비표준 테두리 조각의 공통 소유·저장 계약

- 날짜: 2026-10-19 (Asia/Seoul)
- 지시: [stage14](task_m100_7998_stage14.md)의 다음 절편 진행 승인.
- 시작 head: `38a230bac`. 이번 경계는 원래 경계 조각의 공통 IR 소유와 양 저장 보존이다.
  스타일 교체·새 행열 상속 완료나 P4/P5 완료로 보고하지 않는다.

## 구현 전 계약과 반례

현재 `table_paint::partition`은 실제 물리 조각을 셀 문단의 Shape/Picture로 옮기지만
경계의 source 근거와 셀 변 소유는 폐기한다. 같은 물리 소유 안에도 내용 그림·배경·
가림 paint가 있으므로, 후속 편집에서 bbox 근접만으로 경계를 삭제할 수 없다.

이번 producer는 source 경계로 채택된 path의 단일 축 근거가 있는 불투명 직사각형
stroke 조각에만 변 소유를 붙인다. 원래 축과 해당 셀 변이 일치하고 실제 partition의 물리
조각이 그 셀에 속해야 한다. compound bitmap·혼합 방향·내용 그림·면 배경·표 밖 잔여는
승격하지 않는다. 같은 source가 공유 변 양쪽으로 나뉘면 원래 source ID를 공유하되
서로 다른 실제 개체 ID·셀 소유를 유지한다. 원래 paint/좌표/순서는 바꾸지 않는다.

공통 IR의 소유 표지는 실제 조각 개체에 둔다. HWP/HWPX 보조 metadata는 실제 table·
cell·개체 ID와 현재 물리 속성 지문이 함께 맞을 때만 연결한다. 중복 ID·잘못된 소유·
stale 속성·과도한 metadata는 적용하지 않는다. 기본 한컴에는 도형 자체를 보존하고,
rhwp 전용 편집 소유 보존과 한컴의 편집 기능을 구분해 보고한다.

독립 기대는 기존 공개 phase/underreach/occlusion 반례의 명시적 source 선분과 셀
partition이다. 자격 확인 native 실선/Dot는 중복 소유를 붙이지 않고, 일반 내용/배경·
원본 외부 paint도 별도 소유로 남겨야 한다. 실제 export/reload와 원문·개체·paint의
동일성을 먼저 확인한다. 신규 rendering golden/좌표 핀은 추가하지 않는다.

## 실제 생산·저장·복원의 소비

- `table_paint::partition`은 source boundary의 `paintKind=stroke`·단일 축과 실제 grid
  기여를 확인한 뒤, 변과 일치하는 **이미 분할한 조각**의 `CommonObjAttr.cell_border_paint`에
  `CellBorderPaint { side, source_id }`를 넣는다. 절단·좌표·크기·z-order·paint는 그대로다.
- `source_id`는 같은 원래 paint에서 갈라진 조각을 묶는 불투명한 출처 ID다. 현재 개체를
  찾는 주소나 완전한 PDF stream/operator 참조로 사용하지 않는다. 현재 개체 ID와 물리
  셀 소유는 별개로 보존한다. 전자는 공통 모델의 출처/역추적 일반화와 구분한다.
- `model::border_paint::encode`는 실제 table/cell/직접 사각형 소유를 순회하고, 모서리·
  내부 포함·앵커·불투명 채움과 현재 속성 지문을 확인한다. 중복 개체/table ID는 거절한다.
  옛 변에서 이동해 더 이상 맞지 않는 표지는 저장하지 않으며 도형 자체는 유지한다.
- CFB writer는 `/RhwpCellBorderPaint`, HWPX writer는
  `META-INF/rhwp-cell-border-paint.bin`에 기록한다. 두 parser는 실제 개체·소유·지문이
  맞을 때만 표지를 복원한다. 그 뒤 같은 native 도형을 기존 layout/paint가 소비한다.
  한컴의 native 셀 테두리 의미와 다르다는 content-loss는 두 저장 보고에 명시한다.

renderer·줄/높이·원점·분할·측정 값은 바꾸지 않았다. private 소유 정보는 paint를 대체하거나
삭제하지 않는다. style API와 새 행열의 stencil 상속은 다음 절편의 실제 소비 대상이다.

## 원본으로 드러난 경계와 보정

가림 원본의 흰 면을 얇게 바꾼 **별도** 대조군을 만들었다. 원래 4pt 면과 새 1pt 면을
서로 다른 입력으로 보존한다. 새 면은 기존 검출에서 경계 후보에 기여할 수 있지만 stroke가
아니다. 처음의 모양/축만 확인하던 코드에서는 흰 면 1개가 잘못 표지됐고, 원래 paintKind를
소비한 뒤 0개가 됐다. source 원본과 개체 자체는 모두 유지한다. 색으로 배제하는 구현이
아니며, 이 입력이 흰 fill임은 원래 PDF 명령으로 독립 확인한다.

`underreach-dot`는 바깥 후보가 보류되고 native 자식 표만 생성된다. 최초 검사의 ‘모든
비표준 입력에 표지 필요’ 기대를 버리고 정상 보류 대조로 정정했다. 보류된 원본을 표로
강제하거나 검사 통과를 위해 source를 고치지 않았다.

초기 JSON 코덱은 실물의 **43,767개** stroke 조각에서 문서 한도에 걸렸다. source ID
충돌은 0건이었다(`audit-model.log`). 입력 한도를 늘리지 않고 레코드당 고정 65바이트
바이너리로 바꿔 **2,844,867바이트**에 전체를 담았다. 새 코덱은 다음 계약이다.

- 8바이트 magic/version `RBPO 00 00 00 01`, LE u32 개수의 12바이트 header.
- 각 65바이트: 개체 ID 4 + table/cell/크기/안여백 소유 24 + 변 1 + 출처 ID 4 +
  현재 물리 속성의 SHA-256 32. 변은 Left/Right/Top/Bottom = 0/1/2/3이다.
- 기존 4MiB 제한을 유지하고 최대 개수는 그 바이트 한도에서 유도한다. decode는 길이·
  개수·버전을 할당 전에 확인하며 잘린/중복/모르는 변·stale 소유/지문은 적용하지 않는다.

초기 JSON 빌드/진단과 원본은 `initial/`·`json-stage/`에 구분해 보존했다. 그때의 원본
저장 오류·누락 결과를 최종 검증에 합산하지 않는다. 최초 집중 검사 준비 후 test source를
수정해 suite 배정이 바뀌면서 0건 선택된 실행도 통과로 세지 않았다. 최종 내용에서 prepare한
뒤 정식 suite에서 최종 5건 선택·통과를 확인했다.

## Native에서 확인한 실제 왕복과 출력

`native-roundtrip.json`에서 phase 280개·다른 pen 280개·가림 288개·얇은 fill 가림 288개,
보고서 43,767개의 표지/실제 소유/출처 ID가 **HWP/HWPX adapter 왕복에서 바이트까지 동일**하다.
이미 native Dot인 입력·보류된 바깥 후보 및 fill 경계 중심의 견적서는 불필요한 표지 0개다.
compound·내용 그림/면 배경은 별도 대조군에서 확인한다.

두 실물과 세 공개 입력의 변경 전/후 HWPX `Contents/*`·`BinData/*` **47개 native part는
모두 바이트 동일**하다(`native-payload-identity.json`). 새 private 편집 소유만 추가한다.
이것을 모든 한컴 버전의 열기/편집 검증으로 확대하지 않는다.

Native의 두 실물과 6개 대조군에서 직접 PDF·HWP·HWPX **24건/57쪽**을 원본 PDF와 대조해
최저 **97.57194%**, 누락/쪽수 차이/90% 미달 0건을 확인했다. 내용 그림·배경 대조군
2개×세 형식 6쪽은 각각 99.98114%·100%다. fresh WASM·실제 Studio 저장 및 최종 기록은
후속 결과로 구분해 연결한다. 이번 Native 결과만으로 P4/P5 완료를 선언하지 않는다.

## 최종 fresh WASM·실제 Studio·정식 검사

| 같은 최종 제품의 전쪽 범위 | Native | fresh Docker WASM |
|---|---|---|
| 두 실물·8개 공개/독립 대조군 × 직접/HWP/HWPX | 30건/63쪽 | 30건/63쪽 |
| 전쪽 raw 최저 | 97.57194% | 97.13390% |
| 90% 미달·누락·쪽수 차이·실행 오류 | 0 | 0 |
| 두 저장 재열기의 PNG 대 직접 열기 | 전쪽 byte 동일 | 전쪽 byte 동일 |

원시 TSV는 `native-all-pages.tsv`·`wasm-all-pages.tsv`, 실제 명령은 `validate.py`와
`{native,native-controls,wasm}/results.json`에 있다. 같은 source의 이전 코덱 결과와
최종 binary 결과를 섞지 않는다. `validation-summary.json`·각 `*-format-identity.json`에
쪽별/형식별 집계를 보존했다.

대표 thin-fill과 보고서 5쪽의 compare/standalone overlay/review를 기존 래스터에서
산출해 직접 판독했다. 흰 가림 구간·표 외곽·자식 표·뒤 문장·실물 사진과 글자 표시가
보존된다. 기존 선 농도·글꼴 차이는 남아 있으며 이번 소유 정보로 개선됐다고 보고하지
않는다. 이 변경은 표준 native 데이터 동일성과 저장 보존을 검증한 범위다.

실제 CanvasKit Studio 7793은 최종 WASM SHA를 instantiate했고 page error 0이다.
공개 6개와 두 실물을 실제 PDF로 불러와 양 저장 형식으로 재열었다. 공개 5개는 실제
`insertTextInCell`/`deleteTextInCell`의 X 삽입·삭제도 실행했다. 바깥 후보가 보류된
underreach는 저장/재열기만 확인했으며, 로그의 일괄 편집 성공 문구보다 `studio-result.json`의
실제 outputs 구분을 따른다. 모든 실제 저장본 **26개**를 Native로 재소비해 source와
같은 개체·소유·변·출처·지문을 확인했다(`wasm-native-readback.log`). 편집 문서의 새
독립 한컴 Print까지 검증했다는 의미는 아니다.

최종 `issue_7998_pdf_border_paint_ownership` **5건**은 실제 두 writer 왕복,
공유 source의 다른 셀/변, native/보류 정상 대조, 내용 그림·넓은 배경, 잘린/오래된/
중복/한도 초과 metadata와 4만 개 이상 모델의 codec 보존을 검사한다. 순수 metadata
용량 검사를 160쪽 조판 일치로 승격하지 않는다. 새 thin-fill fixture는 Native/fresh WASM
세 형식의 90% 이상과 직접 판독 뒤 등록했다. 기존 source PDF 5개는 generator 변경 전후
바이트 동일이다. [입력 근거](../../pdf/issue7998/border-paint-ownership-sources.md)를 따른다.

기존 periodic border 4건·border editing 3건·text clip/placement/owned lists 각 3건,
**기존 16건 + 새 5건 = 총 21건 통과**했다. 자동 suite prepare 뒤 실제 선택 건수를
확인했다. 새 test/fixture 추가 외에 runtime은 바뀌지 않아 기존 16건을 중복 실행하지 않았다.
전체 회귀·Clippy·Worker/package·최신 devel 비교는 승인 계획의 P5에 유지한다.

## 빌드 신원·검토 경로와 남은 범위

| artifact | SHA-256 |
|---|---|
| 최종 runtime manifest, 2,797개 파일 | `d2af0bfe983415c02dcf4956073f30a587a434299395e655f3e581591b3bfa0c` |
| immutable Native | `5069d8ad9828d1bf0f0f553127ce3f5882e8279bcea46f13146b094856129874` |
| Docker WASM, 19,298,594 bytes | `734620a54b19b18bed517b85acd971f8ff6eec8f995b828eeb2548d74616ba5c` |
| JS glue, 476,233 bytes | `2f715f0325d1292c52a84165d08477868c979eb94900174231a6a02aa6a8de11` |

Native는 shared target의 `--locked --profile release-test --features native-skia --bin rhwp`,
WASM은 기존 `docker compose -p rhwp --env-file .env.docker ... wasm` 경로다.
`native-binary-build.log`·`docker-binary-wasm.log`·`build-artifacts.json`에 명령/성공과
내용 신원을 연결한다. 최종 source와 빌드 뒤의 runtime 2,797개 파일은 동일하고 이후
수정은 검사/fixture/문서뿐이다. JS는 이전 glue와 바이트 동일이며 pkg와 worktree public의
실제 WASM/JS도 같다. 메인 Studio 7700의 산출물은 수정하지 않는다.

증거 root는 저장소 밖
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-border-paint-ownership`이며
ignored `output/pr-review/issue7998/p4-border-paint-ownership`로 연결했다.
이동 가능한 검토 화면은 `E:\tmp\issue7998-border-paint-ownership-review\index.html`이다.
이미지·TSV·공개 PDF와 양 저장 형식·실제 WASM 셀 편집본을 상대 경로로 함께 복사했다.

이번 완료는 **분리 가능한 opaque stroke 조각의 소유·rhwp 저장 보존**이다. fill·compound의
여러 변/내용 혼재 및 표 밖 잔여까지의 편집 closure는 아직 미완료다. 이어서 명시적 스타일
교체·행열 상속/복원을 이 공통 소유의 실제 소비로 연결한다. 역할만 보고 남은 그림을
무조건 삭제하거나 가림/실제 틈을 native Solid로 바꾸지 않는다. 확정 문단 메트릭·P4 전체·
P5의 완료와 원격 push/PR 등록은 이번 절편에서 선언하지 않는다.
안여백·resize로 기준 프레임이 바뀔 때의 소유/배치 갱신도 후속 편집 소비에서 대조한다.
이번 본문 문자 편집 통과를 모든 크기·테두리 편집 경로의 검증으로 대신하지 않는다.

변경 Rust 파일의 rustfmt 검사·`git diff --check`, 계획/백로그/보고서/입력 근거의 상대
링크, 복사한 검토 HTML의 상대 asset/link 22개를 확인했다. broad gate를 미리 반복하거나
원격 상태를 변경하지 않았다.
새 PDF의 xref 끝 공백은 원본 형식의 일부이므로 기존 `.gitattributes`의 PDF 바이트 보존
규칙을 `periodic-borders/`에도 적용했다. PDF 내용과 검증 SHA는 바꾸지 않았다.
