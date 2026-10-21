# 단계13: 원본 HWP3 수식의 저장 속성과 최종 paint 회귀 검사 재검토

## 대상과 원인

원본 `samples/hwp3-sample16.hwp`(SHA256 `559fd94860cf836d5800436d56055c23a73e59eedef3da273a5ba365e6ed9e16`)의 기존 `inline_equations_keep_source_size_baseline_and_single_line_ownership`를 재검토했다. #7642 단계1017 / 원 구현 `fd42be4af17e452e7b5587b36b15b43620a3cf9a`는 HWP3 사양 표40의 공통 상자 속성을 수식 IR에 보존하고 원본 길이 기준선을 백분율로 환산한다. HWP3 수식 직접 열기에서 크기·글꼴·글자처럼 취급이 사라지던 문제의 보정이며, HWP5 그림의 `flow_with_text` 계약을 HWP3 수식에 추가하는 변경이 아니다.

기존 로컬 검사에서는 `treat_as_char && flow_with_text`와 기준선67을 동시에 요구했다. 원본 압축 본문에서 두84바이트 상자를 직접 확인한 결과 기준 위치는0(글자), 수식 종류2, 셀1개이며 baseline465/높이702(1800dpi)다. 이 형식의 상자에는 HWP5 `flow_with_text` 비트가 없다. 기준선은66.239…%이므로 공통 IR의 반올림 정수는66이며 기존 주석의 올림67도 잘못됐다. 원본 상자 기록·사양 offset·독립 PDF 관측은 같은 증적 폴더에 기록했다. 원본·렌더러 구현을 바꾸지 않았다.

## 기존 출력의 선행 검증

정확한 원본에 대응하는 독립 한컴2020 Print `pdf/hwp3-sample16-current-print-20261017-2020.pdf`(SHA256 `8f0bfd27ae567481e244f39de6f7b596b3afa8b227dce895d4ebf9cebc416bd0`)를 사용했다. 기존192DPI/compat2022, 현재 정본 PDFium smooth_rgb_v1/raw2px90% 조건을 유지했다.

| 범위 | Native | fresh WASM |
|---|---:|---:|
| 전쪽·쪽수 | 64/64 | 64/64 |
| 전쪽 raw 최저 | 94.87079% (40쪽) | 94.87154% (40쪽) |
| 90% 미달 | 0쪽 | 0쪽 |
| 수식6쪽 raw | 99.77898% | 99.79231% |
| 기존 좌표 검사33쪽 raw | 99.90769% | 99.91034% |

실제 runtime은 Native507/fresh WASM509이며 단계12 구현과 동일하다. 이후 제품 소스 변경은 임시 PDF 보관 위치를 알리는 주석뿐이다. producer와 각 산출물 hash를 보존했다. Mac WASM은 locked wrapper의 web/pkg/no-opt 로컬 대체 빌드다. Docker 최적화 통과를 뜻하지 않는다.

Native·WASM6/33/40쪽 review를 직접 읽었다. 6쪽 두 수식, 표와 뒤 본문 및33쪽 SAN 표·글머리표의 소유가 유지된다. 수식 괄호·연산자의 미세 차이와40쪽 연구개발 문단의 일부 줄 간격/래핑 잔여를 완전 일치로 주장하지 않는다.

## 검사 보완

원본의 글자처럼 취급·1200HU 기본 크기·HYhwpEQ·66% 저장 기준선·한 줄 계약을 보호한다. 전64쪽 실제 render tree에서 두 수식의 원문 section/paragraph/control 소유, 스크립트 보존, 누락·중복, 독립 Print6쪽 소속, 유일한 원본 글줄, 원본 크기와 글줄 포함을 검사한다.

6쪽 양쪽 raw99%대와 직접 판독을 근거로, 실제 Equation SVG의 `평점`과 두 `예정가격의` 글리프 원점을 독립 Print 관측값과±2CSSpx로 비교한다. 외부 슬롯·자체 측정값을 기준으로 고정한 것이 아니다. 기존33쪽 표·글머리표·본문 원점의±2CSSpx와 소유 관계 검사는 그대로 유지한다.

직접 실행은 기존2 PASS/1 FAIL(잘못된flow 비트)에서3 PASS/0 FAIL로 바뀌었다. 수정 도중 가려져 있던67% 기대값 오류도 별도로 확인·정정했다. 진단 프로그램의 최초u32/usize 컴파일 오류는 보정했으며 최종 증거로 사용하지 않는다. 정식 Native 묶음522는19건 실행18 PASS/1 FAIL,983선택 제외, exit100이다. 수정한 원본 source-frame3건과 원본 Print 경계4건·HWP3→HWP5 대조군10건은 모두 통과했다. 별도 native bold의 실제 glyph 비신축 검사1건도 통과했다. 남은1건은 `issue_265.hwp`의 저장·재열람 후 그림 다음 본문 baseline686.74667/기대708.15999이며, 원본만 실행한 진단519의 PASS로 저장 경로를 통과했다고 주장하지 않는다. 전체 묶음 성공이 아니며 원인 추적을 계속한다.

## 증적과 잔여

- [전쪽 TSV·PNG·독립 관측값·hash](../pr/assets/issue7445/completed-20261009/rebase-4a07d3ae-20261019/hwp3-equation-source/verification.md)
- [직접 실행 로그](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair521-hwp3-equation-source-contract/after.log)
- [정식 묶음 로그](/Users/tsjang/rhwp/output/pr-review/7445-completed-20261009/rebase-4a07d3ae-20261019/repair522-hwp3-source-contract-gates/nextest.log)

이는 기존 검사 오류와 수식 소유·paint 보호의 보완이다. 전체 PR 수용은 미완료다. Science 원본3쪽 Native의 fixed base 대비−0.32810%p 하락은 여전히 보류 사유이며, 다른 실패와 최종 전체 회귀·필수 게이트·실제 소비 경로 검증도 계속한다.
