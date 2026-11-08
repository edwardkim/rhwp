# 보정28 증거 — Paper 표와 저장 본문 축의 독립 소유

부모 `6e9ef166320b381c0342a1252fd0f8e017499fee`, fixed base `2e3b4f880bd11d35398e6ea0100846611f13a72d` 위의 런타임1055/fresh1057 결과다. 전체 PR 검증 완료를 뜻하지 않는다.

## 원인·공통 결과

원 PR#7406을 통합한 #7437/merge `8baae33464bce7e92cbbd07e715397dca141f836`와 이슈#7390의 저장 host 줄 보존 계약을 대조했다. PrEP 첫 빈 host의 저장 vpos4312+LH1200+LS720은 다음 문단 vpos6232를 증명한다. 종이 기준 떠있는 표의 bottom141HU는 이 텍스트 축에서 빼는 값이 아니다.

처음1050에서 typeset만 고쳐도 최종 출력은 그대로였다.1054의 실제 dispatch/block trace로 원 경로가 실행됨을 확인했고, layout의 별도 최종 원점 분기가 같은 바깥여백을 다시 빼고 있음을 찾았다. 임시 trace는 hash 확인 후 제거했다. 새 `stored_host_line_end_hu`가 합성되지 않은 현재/다음 저장 줄의 인접 관계를 인증하고 vpos+LH를 발행한다. 기존 typeset 예약과 최종 layout이 같은 끝점을 소비한다. caller의 저장 배치/빈 host/Paper·배경 제외 조건은 유지한다. 문서명·쪽수 예외나 px 오프셋은 없다.

## 전쪽 결과

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 90% 미달 |
|---|---:|---:|---:|---|
| 법령 | 74 | 96.05600% | 96.01774% | 0/0 |
| PrEP | 140 | 94.75150% | 94.22354% | 0/0 |
| RowBreak HWPX | 18 | 92.05401% | 92.05433% | 0/0 |
| RowBreak HWP | 18 | 93.07060% | 92.96483% | 0/0 |
| 별도 수도시설 대조군 | 3 | 71.85405% | 72.05077% | 3쪽/3쪽 |

양쪽 각각250쪽은90% 미달·쪽수 불일치·누락0이다. 수도시설 기존 미달은 별도 대조군이며 이 통과 범위에 합치지 않는다. 각 `*-all-pages.tsv`에 전쪽을 보존한다. canonical96DPI/PDFium smooth_rgb_v1/raw2px90% 그대로이며 시각 캡처 총8/배타 lock을 유지했다. 완료SVG만 hash manifest 뒤 정리했고 PNG/TSV/로그/원본/PDF는 보존했다.

[Stage27과 대응506쪽 TSV](paired-stage27-506-pages.tsv): 입력/PDF/글꼴/도구/래스터라이저를 맞췄고0.1%p 이상 하락0이다. 양쪽PrEP1쪽 PNG만 변경됐으며 나머지504PNG는 byte 동일하다. fixed devel 전체 보존 검증의 대용은 아니다. #8064 Native-only 하한의PrEP140쪽은 통과한다. 법령2/13쪽0.15331/0.18072%p 하락은 미해결이며 하한을 완화하지 않았다.

## 대표 판독

PrEP1쪽은 양쪽raw100.00000%다. 한국어·영어 제목, 연구기관·질병관리청의 순서/배치와 첫 표의 독립 소유를 직접 읽었다. 본문 텍스트 축이 복원됐고 첫 표 위치는 유지된다. 표 선의 색상/두께와 일부 자형의 미세 차이는 남는다. strictRGB는 약62.6%이므로raw100을 이미지 완전 일치로 표현하지 않는다.

- Native 전: [review](prep-native-p001-before-review.png) · [overlay](prep-native-p001-before-overlay.png)
- Native 후: [review](prep-native-p001-after-review.png) · [overlay](prep-native-p001-after-overlay.png)
- fresh WASM 전: [review](prep-wasm-p001-before-review.png) · [overlay](prep-wasm-p001-before-overlay.png)
- fresh WASM 후: [review](prep-wasm-p001-after-review.png) · [overlay](prep-wasm-p001-after-overlay.png)

## 입력·소스·제품 신원

PrEP `samples/issue2006/1790387_prep_final_report.hwpx` SHA `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`; 독립 Hancom2024 Print140쪽 `pdf/issue2006/1790387_prep_final_report-current-print-2024.pdf` SHA `9580bb332813b70f27a367e79ad735b238bcd716ff51b849e716f7ddd1e60af0`. 다른4입력/PDF는 Stage27 README와 동일하고1058 producer에 고정했다.

- float_placement.rs: `beee2d6dae111e7ccdb98bafd166105a2cd06036568091f5f193d3488dc4f058`
- layout.rs: `a161a63f722fd7742d4eb952ca5e6413ae02f591e38d5fc4efdc7f1d00ec6e16`
- typeset/table/block/entry.rs: `9b9e2e60cdd9e6ced13585b34d01e5e3778fc7d0563214c53d4a9b1746b6b766`
- Native1055 CLI: `84fc2188f085934096a23775e0674683b18942a1ffbf85b8cc626685ec024aff`
- fresh1057 JS: `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`
- fresh1057 WASM: `7d0b82c5e81e9fa490a75e6823e15a3c4143659fe9be76b62c7e2f1cff98810a`

Mac locked web/pkg/no-opt256.322초 exit0은 로컬 대체 빌드다. root pkg/Studio public hash와1061의 실제 Studio7736 초기 JS/WASM 응답을 대조했고 초기화true/오류0이다. JS는 Vite의WASM URL rewrite/source-map원문까지 확인했고WASM은byte 동일하다. Docker 최적화나 문서Actual36/CanvasKit 수용으로 주장하지 않는다.

## 정식 회귀·게이트·남은 실패

새 case `issue_7445_paper_host_source_frame` SHA `954238dff2335ea14564959e9fa92e402c98ecbc7895b33de7caf76466a5f6a2`는 같은 최종source를 frozen1032에 실행하면1FAIL→1055에서1PASS다.1064의 --prepare 후 실제 자동suite025는1PASS(실행1.594초/총174.392초)다. source-unit 수는 늘리지 않았다. 원문 두 제목·첫 글자 소유·순서·누락/중복·140쪽, 원본/같은margin/top0/bottom0/both0에서 텍스트축 불변을 검사한다. 양쪽100%와 직접 판독 후 독립PDF 한국어제목x89.8712933333/baseline307.823592, 영어x90.0312066667/baseline415.8655173333CSSpx±2와 실제Native Print를 대조한다. 좌표만으로1.88px오류를 놓치지 않도록 원문소유/여백독립관계를 함께 유지한다.

1065/1068 기존10case는30PASS/1FAIL이다. 실패는 #7751 `master_raster_print_frame_preserves_paper_border_and_unrelated_source_frames`의Print 이미지 선택조건이며1067의이전1032에서도같은source로1PASS/1FAIL이 재현된다. 현재 실제Print에 전체바탕쪽이미지는존재하고,Print가crop을소비한뒤원래crop필드를비교하는선택조건이맞지않는다. 기존검사는아직수정하지않았으며 handbook전383쪽 Native/fresh 자격확인과독립기준으로후속처리한다. 신규1건을포함한집중결과는31PASS/1기존FAIL이며전체회귀통과가아니다.

1068 Native/WASM lib strictClippy, manifest/check·unit-tier/check(base2e3), cargo fmt all/check, git diff/check는모두exit0이다. jobs8/threads8/Cargo순차를유지했다. 필터미선택수는전체회귀skip으로합산하지않는다.

원시source/trace/producer/JSON/log는 ignored output `rebase-4a07d3ae-20261019/repair1049`~`repair1068`에 보존한다. TSV는 원시와parsed records 동일성을 확인하고 제출LF로 정규화했다. 글꼴바이너리·생성JSON·로그는제출하지않는다. 최신전체corpus/devel TSV·전체nextest/필수build/Skia등gate·Actual36, 법령하한·과학기준선·기존7751실패는남았다. PR은사용자승인후진행한다.
