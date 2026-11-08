# 보정27 증거 — 셀 앞 간격과 중첩 이어받기 시작 띠

부모 HEAD `68af0105b0fd5d80addce5e711f0abf53a21fefc`, fixed base `2e3b4f880bd11d35398e6ea0100846611f13a72d`. source1032/fresh1034와 같은 런타임의 전쪽 결과다. 전체 PR 검증 완료가 아니다.

## 전쪽 결과

| 문서 | 쪽수 | Native 최저 | fresh WASM 최저 | 미달 |
|---|---:|---:|---:|---|
| legal | 74 | 96.05600% | 96.01774% | []/[] |
| prep | 140 | 94.75150% | 94.22354% | []/[] |
| rowbreak-hwpx | 18 | 92.05401% | 92.05433% | []/[] |
| rowbreak-hwp | 18 | 93.07060% | 92.96483% | []/[] |
| water-control | 3 | 71.85405% | 72.05077% | [3]/[3] |

법령·PrEP·RowBreak 두 형식은 양쪽 각각250쪽이며90% 미만/누락/쪽수 불일치0이다. 별도 수도시설 대조군3쪽은 기존 미달로 수용 범위에 합치지 않는다. 새로 렌더한 양쪽6PNG가1000/1002의 이전 결과와 byte 동일하다(ignored1046).

이전 브랜치848/962 대비500쪽의 입력/PDF/글꼴/도구/래스터라이저를 맞춰 대응 비교했다.0.1%p 이상 하락0쪽이며 고정 devel 전체 보존 검증의 대용은 아니다. PNG byte 비교는471쪽 동일/29쪽 변경이다. Native 변경은 법령36/37쪽이며 WASM의 다른 변경 페이지·실제 pixel 차이는 ignored1047 감사에 별도 기록했다. 모든 대응 raw 하락은0.1%p 미만이다. 세부값은 [대응 TSV](paired-previous-branch-500-pages.tsv)에 있다.

## 대표 판독

37쪽 Native99.98294/fresh WASM99.98291%,38쪽100/100%다.37쪽의 중첩 표 종료·후속 두 행의 baseline과37→38쪽 이어받기를 판독했다.36쪽은96.05600/96.01774%로 앞 간격 개선을 확인했으나98% 미만이므로 px 핀을 추가하지 않았다.36쪽 아래 부분표·일부 자형·선에 차이가 남는다.

- native 36쪽: [review](legal-native-p036-review.png) · [overlay](legal-native-p036-overlay.png)
- native 37쪽: [review](legal-native-p037-review.png) · [overlay](legal-native-p037-overlay.png)
- native 38쪽: [review](legal-native-p038-review.png) · [overlay](legal-native-p038-overlay.png)
- wasm 36쪽: [review](legal-wasm-p036-review.png) · [overlay](legal-wasm-p036-overlay.png)
- wasm 37쪽: [review](legal-wasm-p037-review.png) · [overlay](legal-wasm-p037-overlay.png)
- wasm 38쪽: [review](legal-wasm-p038-review.png) · [overlay](legal-wasm-p038-overlay.png)

## 입력 신원

| 입력 | SHA256 | 독립 PDF | SHA256 |
|---|---|---|---|
| `samples/issue6776/78494-virtual-convergence-industry-decree.hwpx` | `5090e9d20ce75814febeb4b79e106ea6abfd2546e511f229900d137d42623617` | `pdf/issue7642-legal-table-origin/78494-virtual-convergence-industry-decree-2020.pdf` | `674ef63b51adc926c7c89f530bd142a45bb64cb655a7117bc20285d7a6e5a8d2` |
| `samples/issue2006/1790387_prep_final_report.hwpx` | `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f` | `pdf/issue2006/1790387_prep_final_report-current-print-2024.pdf` | `9580bb332813b70f27a367e79ad735b238bcd716ff51b849e716f7ddd1e60af0` |
| `samples/rowbreak-problem-pages.hwpx` | `4ccc321e7eb146f28b768654502a128ba1ae408238fd1ed37d51e963732eb5b0` | `pdf/rowbreak-problem-pages-hwpx-2020.pdf` | `e3dc7bb21707f92e42e83808023bf155f62475fc4eb0e0498709d03986c5afe5` |
| `samples/rowbreak-problem-pages.hwp` | `10b6ab6548610e18c82ba78a1c844a00107fedbb28c195cb05e6fd20626d33ed` | `pdf/rowbreak-problem-pages-hwp-2024.pdf` | `2c49bda9cc21dc8b93b554d2607da241dd7e894657eb308ed823e7d44654e85c` |
| `samples/issue6776/36367506-water-facility-approval.hwpx` | `62ee54efc7fd9f5fd06cbfdb5843edcdbf84074b05ede5571c15078d4f1b14ab` | `pdf/36367506-water-facility-approval-2020.pdf` | `fe82e237046659fdd770811dbd30c8f199d155d9737f1a8d0732f19f2049deda` |

## 제품과 소비 경로

canonical96DPI/PDFium smooth_rgb_v1/raw2px90% 문턱을 유지했다. 한 번에16쪽을 묶되 시각 캡처 총8을 배타 lock으로 제한했다. 완료 SVG는 SHA manifest 후 정리하고 PNG/TSV/로그·원본/PDF를 유지했다. 삭제 SVG를 재사용 가능한 증거로 취급하지 않는다.

Native1032 CLI SHA `2bf4b63f44ba13828c5a12d20edbcaf077396ed0ff6b8f1bb67ad87d5b3e311f`; fresh1034 JS `63c587e4b072f0a65416e65a0e86bf00b2971cdfb0f46cfda7f150449dd30524`; WASM `f646c427da6515092c63d02f31135edc138d999e58c9a6fbfbd96fcd729169fd`. Mac locked web/pkg/no-opt232.589초 exit0 로컬 대체 빌드이며 Docker 최적화 통과가 아니다. root pkg/Studio public/1041의 실제 Studio7736 JS/WASM 응답 hash가 같다. 초기화 true/오류0이며 문서 Actual36은 별도 미완료다.

소스 hash:

- `src/renderer/cell_paragraph_spacing.rs`: `442e025b4beb51c9b2275cc24798a1c1e05ec7c0d1099cb66406d42acce47ada`
- `src/renderer/layout/paragraph_layout.rs`: `8e9dea778360bc7e860b7551b6e670365855cd8435271c1debdbe285085ca0e9`
- `src/renderer/layout/table_layout.rs`: `11b67535f5d716a6be2c3eb80806587bd1c6f012c760b2f45d4395476010f020`

## 회귀 및 잔여

같은 최종 case SHA `b211d5919a7f24587e5bfe2a9215f81b631aa95df8c7241935b1c283606f6e03`: frozen1000에서2PASS/1FAIL→1032에서3PASS.1043의 정식 자동 nextest는3PASS/262선택제외(실행1.653초·전체390.243초)다. source-unit 증가 없이 tests/cases에 수집하며 실제 suite009를 실행했다. 원문 before400→물리200HU 관계·메타데이터 중립·중첩 닫힘 원본 여백·후속 두 셀의 원문 및37/38쪽 조각 소유를 검사한다.37쪽 독립 Print 원시 text matrix의 x348.4511533, baseline609.893472/903.9720853CSSpx를 최종 Print runToPage와 ±2px 대조한다. 기대값을 후보 bbox에서 역산하지 않았다.

#8064 Native-only 하한 감사1038에서는 법령2/13쪽 각각0.15331/0.18072%p,PrEP1쪽0.62299%p 하락이 남는다. 하한이나90% 문턱을 변경하지 않았다. 최신 전체 corpus/devel 대응 비교·전체 nextest/필수 gate·Actual36은 별도 남는다.

원시 로그/producer/query/독립 인쇄 대조는 ignored `/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019`의1032~1046 폴더에 보존한다. 생성 JSON/글꼴 바이너리·로그는 제출 자산에 추가하지 않았다.

정상 대조1044의 기존8case35PASS/0FAIL과 신규1043의3PASS/0FAIL, 집중 합계38PASS다. 전체 nextest 결과는 아니다.1045의 manifest/check·unit-tier/check(base2e3)·수정 Rust 파일 fmt/check·git diff/check는 모두 exit0이다.
