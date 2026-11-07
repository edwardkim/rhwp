# #8733 한컴 표 배경 저장 수정

- 최초 기준: devel b8dd67c5001dfb0410e9e192528676396cdd6a4e
- 작업 브랜치: task/8733-solid-fill-hancom
- 확인일: 2026-11-07 UTC
- 대상: https://github.com/edwardkim/rhwp/issues/8733
- 비교 자료: https://github.com/edwardkim/rhwp/issues/8733#issuecomment-6702261909

## 원인과 수정

표/셀 배경 대화상자와 공통 BorderFill 생성기가 새 단색 배경의 무늬 없음에 0을 사용했다. 첨부 HWP와 신고자의 한컴 출력에서는 이 값이 가로 무늬로 나타난다. 새 단색의 무늬 없음은 -1로 저장한다.

- 두 표 배경 대화상자의 무늬 없음 선택값을 -1로 변경했다.
- 원본에서 읽은 0은 숨김 보존 선택지로 유지한다. 테두리만 바꾸거나 확인할 때 NaN 또는 -1로 치환하지 않는다. 미리보기에는 가로 무늬를 표시한다.
- 배경이 없는 대상으로 대화상자를 재사용하면 무늬 선택을 -1로 초기화한다.
- 공통 생성기는 부분 편집에서 생략한 기존 배경색, 무늬색, 무늬 종류를 유지한다. 기존 배경이 없는 새 단색만 흰색/검정색/-1을 기본값으로 쓴다.
- HWP 바이너리 writer와 양수 무늬 매핑은 변경하지 않았다. 이 변경은 모든 무늬 형식의 전면적인 매핑 수정이나 기존 손상 문서의 자동 복구가 아니다.

## 너비와 간격

첨부 HWP의 일곱 표에 저장된 공통 FLAGS 0x04200317은 #7606의 이전 Table.attr 전체 덮어쓰기 결과와 일치한다. 이 값에는 PAPER 크기 기준과 affect_line_spacing 비트가 포함된다. #7608은 현재 devel에 이미 반영되어 있으므로 compositor나 크기 계산을 중복 변경하지 않는다.

https://github.com/edwardkim/rhwp/pull/7608

기존 #7606 회귀에 새 TAC 표의 줄 간격 영향 비트가 꺼져 있고 빈 속성/위치 편집 후에도 유지된다는 검사를 추가했다. 오래된 손상 문서의 기존 FLAGS를 임의로 정규화하지 않는다.

## 검증

- UI 수정 전: 새 회귀 9건 모두 실패.
- UI 수정 후: 새 회귀 9건 모두 통과.
- Rust 수정 전: 새 저장 회귀 3건 모두 실패. 새 단색의 0/-1 불일치와 부분 편집의 무늬색 소실을 재현했다.
- 고정 base 대비 Rust suite manifest 및 unit-tier 정책: 통과.
- fresh WASM dev 진단 빌드 및 최종 optimized release WASM 빌드: 성공. wasm-pack 0.15.0, Rust 1.93.1, locked wrapper, wasm-opt -O, exit 0.
- 실제 Chrome 155 + 최종 optimized release WASM: 셀과 표 대화상자 모두 새 단색 -1, 취소/재열기, HWP/HWPX 저장/재열기, 원본 0 보존, 명시적 무늬 제거, undo/redo 통과. browser-results.json의 success=true 및 브라우저 오류 0건을 확인했다.
- Native root Clippy: 통과. TypeScript noEmit: 통과.
- 중단 복구 시 이전 focused Native 실행이 잘못된 파생 shard를 선택해 0건만 실행했음을 확인했다. 이를 통과 근거로 사용하지 않았으며 현재 파생 shard에서 #8733과 #7606을 각각 3건씩 재실행해 통과했다.
- 전체 Studio 최종: 설치된 Node 24.22.1에서 2,756건 중 2,754 통과, 2 skip, 실패 0건(108.42초, exit 0). 기존 #8102 fixture의 URL.pathname이 Windows에서 /C:/ 경로를 생성하는 문제를 fileURLToPath로 최소 보정했다. 해당 fixture의 12건과 TypeScript도 별도 통과했다. 제품 코드나 테스트 기대값은 바꾸지 않았다. 초기 Node 22 및 Windows 경로 실패 기록은 진단 로그로 보존한다.
- WASM Clippy: 통과(exit 0, 3분 33초). #8733 focused Native: 3건 통과(exit 0), raw HWP -1 저장 및 HWP/HWPX roundtrip 확인.
- #7606 focused Native: 3건 통과. 빈 속성과 위치 편집 후 절대 크기 기준 및 줄 간격 플래그 보존을 확인했다.
- workspace build: 통과(exit 0, 4분 20초). workspace all-target Clippy: 통과(exit 0, 3분 20초).
- 전체 release-test: 55개 바이너리, 12,767건 통과, 45건 skip, exit 0. 명령 전체 40분 33초(컴파일 15분 12초 포함).

새 Rust 검사는 HWP DocInfo의 BorderFill 원시 바이트와 HWP/HWPX 재열기 결과를 검증한다. 기존 가로 무늬 0 및 양수 무늬의 HWP 저장 보존, 배경색만 바꾸는 편집, 무늬만 없애는 편집을 포함한다. 후속 사용자 승인으로 한컴 2020(11.0.0.9166) 실행과 실제 Print 확인을 완료했다. 새 단색 HWP/HWPX 및 명시적 무늬 제거 결과는 단색이고, 기존 무늬 0은 가로 무늬를 유지했다. 승인 우회나 원격 업로드는 하지 않았다. 전체 양수 무늬 형식 간 대응도 이번 수정 범위가 아니다.

## 제출 후보와 증적

사용자가 남은 검증과 PR 제출을 승인한 후 최종 optimized WASM 및 Studio 전체 검증을 완료했다. 코드 commit은 `cf88d4c70`, Windows 테스트 경로 보정은 `e551fcc76`이며, Rust 전체 검증 이후 제품·Rust 테스트 소스는 동일하다. 이후 추가되는 파일은 보고서와 검증 입력뿐이다.

- 공개 가능한 합성 HWP/HWPX, 대화상자 PNG, 파일 해시와 검증 집계: [검증 입력](../pr/assets/issue_8733/README.md).
- [한컴 Print PDF](../pr/assets/issue_8733/hancom-print.md)와 [입력·출력 해시 및 대응 근거](../pr/assets/issue_8733/hancom-print-manifest.json)를 보존한다. 초기에는 명시적 100% 인쇄 배율이 미확인이었으나 이후 출력자가 7개 모두의 100%·원래 용지·전체 페이지·한 쪽씩 설정을 확인했다. 최초 표 HWP PDF는 덮어쓰기되었으며 재인쇄본을 별도로 보존했다. 이를 전체 시각 일치로 주장하지 않는다.
- 생성된 `rhwp-studio/public/` WASM glue와 WASM 바이너리는 제출에서 제외한다.
- 이 보고서는 제출 후보의 로컬 검증 기록이다. 원격 CI 결과는 PR의 정확한 head에서 별도로 확인하며 merge는 사용자 승인 없이 수행하지 않는다.

## 후속 독립 Print 확인

2026-11-07에 실제 한글 2020과 Hancom PDF 1.3.0.550 Print 경로로 요청한 7개 합성 입력의 채움·무늬 동작을 확인했다. 현재 보존된 PDF는 후속 표 HWP 재인쇄를 포함해 7개이며 모두 A4 1쪽이다. 두 HWPX 출력의 원본 확장자는 출력자가 직접 확인했다.

- 새 단색 셀·표 HWP/HWPX: 배경 RGB(170,187,204), 가로 무늬 없음.
- imported-raw-zero.hwp: 가로 무늬가 유지됨.
- cell-cleared-zero.hwp와 table-cleared-zero.hwp: 명시적 무늬 제거 후 단색 유지.
- 새 셀 HWP와 HWPX Print의 240 DPI 래스터가 바이트 단위로 동일함. 다른 파일 간 완전한 픽셀 일치는 주장하지 않음.
- 최초 표 HWP PDF는 뒤의 HWPX 수동 인쇄 중 덮어쓰기되었으며, 최초 원본 래스터와 SHA 기록은 남아 있음. 07:34:16 UTC에 실제 HWP를 다시 인쇄한 별도 PDF도 보존함. HWPX PDF는 덮어쓰기 전에 별도 사본으로 보존함.
- 최초 셀 인쇄의 전체 페이지·기본 인쇄·한 쪽 도식은 관찰했으나 명시적 가로·세로 100% 배율은 확인하지 못함. 배율 확인 시 창이 닫혀 있었고, 이후 HWP 재인쇄 때도 확장 탭 활성화 실패로 배율을 확인하지 못했음. 과거 설정은 추정하지 않음.

Native/fresh WASM 시각 측정은 보조값·직접 판독·인쇄 조건을 구분해 보고한다. 현재 확정 7쌍의 fresh optimized WASM 전체 1쪽 raw silhouette는 모두 100.00%이며 누락·unmatched pixel은 0이다. `silhouette_only` 결과의 gate는 `not_evaluated`이므로 이 값만으로 시각 검증 완료를 선언하지 않는다. WASM 대표 이미지 판독에서는 새 단색·명시적 제거 6개는 채움·배치가 일치하고 raw0 대조군 1개는 내부 가로 무늬 누락이 확인됐다. Native 스키아 빌드·7개 측정·전체 리뷰도 exit 0으로 완료했다. Native 각 1쪽의 raw silhouette 역시 모두 100.00%이고, 직접 판독에서는 같은 raw0 가로 무늬 누락이 확인됐다. 다른 6개는 채움·배치가 일치하고 경계 래스터 차이는 남는다.

## 원격 CI

제출 head `0352c9deee13130f374cfc8015384512ecc6256c`에서 [메인 CI](https://github.com/edwardkim/rhwp/actions/runs/42349923309), CodeQL, Adapter inter-diff, Proptest roundtrip이 성공했다. 별도 Render Diff도 성공해 5개 workflow가 모두 성공했다. 후속 증적 commit은 코드 변경 없이 만들고 해당 head 상태를 별도로 확인한다. Draft PR #8738은 병합하지 않았다.

## 기존 렌더링 차이의 범위

고정 devel의 `src/renderer/style_resolver.rs`는 `pattern_type > 0`에서만 패턴을 생성하므로 0은 단색이다. `web_canvas.rs`는 1~6, `skia/fill_shader.rs`는 0~5 번호로 패턴을 그린다. 이 파일들은 현재 PR과 고정 base 사이에 diff가 없다. 이 저장 수정이 렌더러의 전체 무늬 매핑을 고쳤다고 주장하지 않는다. 기존 raw0는 값과 한컴 출력에서 보존되지만 rhwp의 페이지 표시에서는 해치가 누락된다.

[Native/WASM 원본 TSV와 직접 판독 자료](../pr/assets/issue_8733/README.md#페이지-시각-비교)를 보존했다. 이 잔여 차이 때문에 7개 모두의 시각 일치로 표시하지 않는다. 이후 출력자가 인쇄 조건을 확인했으며, 고정 base 비교와 self-review의 범위별 판정은 [PR 검토](../pr/archives/pr_8738_review.md)에 기록한다.

Native 빌드는 Rust 1.93.1의 locked release native-skia 기능으로 완료했고 재연결 후 재개 빌드는 23분 30초였다. 실행 파일 SHA-256은 2284ff6defecbd40754af052d47d6bd63c2b2ab56a28b3ee3276552b8b5e15ec이다. pdftotext의 bbox 추출은 설치된 Xpdf 버전의 옵션 미지원으로 생략됐으며, 글자 없는 합성 셀의 PNG/PDFium 비교와 직접 판독을 수행했다.

## 인쇄 조건 후속 확인

출력자가 2026-11-07에 보존된 7개 PDF 모두의 인쇄 당시 가로·세로 100%, 원래 용지, 전체 페이지, 한 쪽씩 설정을 명시적으로 확인했다. 최초 도구 관찰의 미확인을 이 출력자 확인으로 보완했다. UI 직접 관찰이나 PDF metadata로 입증한 것으로 표현하지 않으며, 파일을 다시 인쇄하거나 대체하지 않았다. raw0 렌더링 잔여 차이와 base 비교는 별도 판정한다.


## Self-review 후속 비교

최초 b8dd67c5뿐 아니라 최신 devel e4ffabea38062e716f6c333cc950fa31b42b4dd7의 Native/fresh WASM 비교를 완료했다. 같은 7개 입력의 전체 14개 page/backend 쌍에서 raw 100.00%, Decimal 최대 하락 0.00%p, 실제 렌더 PNG 바이트·RGB 픽셀 동일을 확인했다. 대표 compare/overlay/review 18개도 후보와 바이트 단위로 같았다. [최신 비교](../pr/assets/issue_8733/latest-base/comparison.json)와 [실제 명령](../pr/assets/issue_8733/latest-base/REPRODUCE.md)을 보존한다.

이 결과와 출력자의 Print 조건 확인으로 저장·단색 채움·부분 편집 보존 범위의 self-review를 승인했다. 기존 raw0 페이지 해치 누락, 전체 무늬 매핑과 이미 손상된 원 문서의 너비·간격 복구는 완료 주장에 포함하지 않는다. Native 기본 dev와 후보 release, WASM codegen 기록 차이는 실제 producer 정보에 명시했으며 동일 옵션이라고 주장하지 않는다. 원격 ready·merge는 문서 trailing head CI 및 최신 상태 확인 뒤 진행한다.
