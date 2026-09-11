# #7032 Stage 1 — 원본·IR·줄 구성·배치 계보 조사

- 이슈: https://github.com/edwardkim/rhwp/issues/7032
- 기준 source: `d408532ce9f333f7428bf91362829f2703f0c02a`
- 수행계획 승인 기록: `c2fef1c0e`
- 날짜: 2026-09-11 (KST)
- 상태: 조사 완료, 제품 코드·테스트 미변경. 구현계획 승인 요청으로 전환한다.

## 1. 결론

**HWP 원본의 빈 문단이 파서에서 삭제된 것이 아니라, 분할 표 렌더러가 줄 목록이 비어 있는
온전한 셀 문단을 비가시 문단으로 오인하여 건너뛴다.** 일반 문단 배치 함수에는 빈 문단의
글꼴·줄간격으로 진행하는 기존 처리까지 있지만, 이 분기 때문에 도달하지 못한다.

HWPX는 원본에 저장된 줄 정보가 있어 같은 문단이 건너뛰어지지 않는다. 메인테이너는 Studio에서
HWPX의 정상 조판과 HWP의 오조판을 직접 확인했다. 고칠 대상은 저장 줄 정보가 없는 실제 빈
문단의 가시성·배치이며, HWPX의 정상 경로와 파서의 원본 보존을 유지한다.

## 2. 입력 및 증적 재사용

| 입력 | 경로 | SHA-256 |
| --- | --- | --- |
| HWP | `samples/task2146/21761835_jeonjik_exemption_table.hwp` | `d8e7e38b206f6e53b7d2d260115396f8a0dd94bd8b712bbc4428dc11bf66e5dc` |
| HWPX | `samples/hwpx/21761835_jeonjik_exemption_table.hwpx` | `53b33bfca8d8a80534f27564ecee86403c6d1e3dc83cf405c7d6d158ae9625d2` |
| 기존 한컴 PDF | `pdf/task2146/21761835_jeonjik_exemption_table-hwp-2020.pdf` | `99d150b26805a29f3d3153528638f363e33f882d0e38ad11edb374baccdac127` |

HWPX는 메인테이너 제공 `/mnt/e/hwp/21761835_jeonjik_exemption_table.hwpx`를 바이트 그대로
복사했다. 기존 HWP와 PDF는 변경하지 않았다. PDF의 실제 metadata는 Hwp 2022/Hancom PDF다.
새 MCP 변환·코퍼스 전수 측정을 수행하지 않았다.

사용한 기존 보고서: [#7028 Stage 2 §6](task_m100_7028_stage2.md#6-메인테이너-피드백--선-확인-빈-문단의-텍스트-진행-누락),
[#2146 계획](../plans/archives/task_m100_2146.md),
[#2169 보고서](../report/archives/task_m100_2169_report.md).
과거 문서의 당시 수치와 현재 코드 계약은 구분했다.

## 3. 원본 → IR → 줄 구성 대조

대상은 section=0, paragraph=4, control=0, r0c0이다.

| 계층 | HWP | HWPX |
| --- | --- | --- |
| 원본 문단 | 빈 문단 + `직렬\r` 문단 | 빈 문단 + `직렬` 문단 |
| 저장 줄 정보 | 두 PARA_HEADER 선언 개수=0, 실제 LINE_SEG 없음 | 두 문단 각 1개, vertpos=0/1416 |
| parser IR | 두 문단 보존, 두 line_segs=[] | 두 문단 및 저장 줄 정보 보존 |
| 공통 스타일 | ParaShape53, CharShape37, Fixed raw=2832 | 동일 |
| 셀 정렬·둘째 문단 break | Center, Page(raw=4) | 동일 |
| compose_paragraph 줄 수 | 0개 / 1개 | 1개 / 1개 |

HWP 원시 셀 레코드는 24 이상 30 미만이다. 첫 문단 record25는 char_count=1이며
PARA_TEXT가 없고, 둘째 record27은 char_count=3, break=4다. record28은 UTF-16
`[51649,47148,13]`이다. 원본의 빈 문단 수 1개와 메인테이너가 설명한 빈 두 줄의 편집 의도를
동일한 수로 취급하지 않는다. 입력을 변형해 문단을 보충하지 않는다.

HWP 파서의 #2070 조건은 모든 저장 줄의 높이/텍스트 높이가 0일 때 줄 목록을 비우지만,
이 셀에는 레코드 자체가 없어 해당하지 않는다. 파싱 오류 문단을 생략하는 경로도 이 두 문단이
IR에 모두 존재하므로 원인이 아니다. 파서 전체의 무결성을 보증하는 주장은 아니다.

## 4. 2832와 1416은 스케일 차이이며 모순이 아니다

`src/renderer/style_resolver.rs::resolve_single_para_style`는 비-Percent 줄간격을
`hwpunit_to_px(raw, dpi) / 2.0`으로 해소한다. HWPX parser의 case/default 스케일 계약과
대칭이며, 이번에 별도로 나누기 규칙을 추가할 필요가 없다.

96 DPI에서:

- raw Fixed=2832 → 유효 1416 HU → **18.88px**.
- HWPX 저장 줄 높이 1000 + spacing416 = **1416 HU**.
- 두 형식의 실제 `resolve_styles` 결과도 모두 **18.88px**였다.

따라서 앞선 계획의 “2832와 1416 관계 미확정”은 이번 조사로 해소했다. #2146 과거 문서의
37.76px는 당시 /2 적용 전 기록이며 현행 값을 설명하는 수치로 사용하지 않는다.

## 5. 실제 배치와 수치 검산

기존 HWP RenderTree와 추가 HWPX 첫 페이지 RenderTree를 대조했다.

| 항목 (96 DPI, 출력 소수점 1자리) | HWP 기존 출력 | HWPX 추가 출력 |
| --- | --- | --- |
| 셀 y / 높이 | 225.3 / 52.4 | 225.3 / 52.4 |
| 빈 문단 TextLine | 없음 | y=235.3, h=13.3 |
| `직렬` TextLine | y=235.3 | y=254.2 |

차이는 약 18.9px로 해소된 줄간격 18.88px와 일치한다. 셀 크기는 그대로이며 텍스트만 한 줄
진행을 잃었다. 이 수치는 원인을 뒷받침하는 대조이며 문자열별 y 고정값으로 구현하지 않는다.
HWPX CLI 로드는 6쪽으로 보고했다. 새 출력은 첫 쪽만 생성했다. 다른 쪽의 새 시각 판정을
완료했다고 주장하지 않는다.

## 6. 현행 소유 경로와 변경 이력

1. `composer.rs::compose_lines`는 text와 line_segs가 없으면 빈 목록을 반환한다.
   공개 저장소 최초 commit `f0f7f1a4b4`(2026-03-27)에도 존재한다.
2. 같은 최초 commit의 `table_partial.rs`에는 줄 범위가 비고 중첩 표가 아니면 continue하는
   경로가 이미 있었다. `bc1cd4dbab`(2026-05-20, #1022)는 cut 가시 범위만 소비하도록 정리했다.
3. `ce33156969`(2026-07-11, #2169)는 빈 문단의 높이 측정·셀 유닛 계산을 보완했다.
   현재 `calc_para_lines_height`와 `cell_units`는 비마지막 빈 문단의 줄간격을 계상한다.
4. `b37b38d28c`(2026-07-12, #2070)는 비-Percent 줄간격 /2 재제출이다.
5. `3c654028e0`(2026-08-09, #3820)는 일반 문단 배치의 빈 줄 메트릭을 보완했다.
   현재 `paragraph_layout.rs::empty_no_lineseg_paragraph_metrics`와
   `layout_composed_paragraph` 말미 fallback이 글꼴·줄간격으로 빈 줄/캐럿을 만든다.
6. 현재 partial 경로는 호출 직전 `start_line >= end_line`에서 온전한 빈 문단까지 skip한다.
   전체 콘텐츠 높이는 먼저 계산하여 Center 원점을 맞추지만 실제 y 진행은 빠지는 불일치다.

**분류:** #7028의 신규 회귀가 아니라 공개 이력 처음부터 있던 처리 공백이 후속 측정·일반 문단
보완에도 partial 경로에 남아 있는 상태다. 최초 commit을 이 샘플로 빌드·실행한 bisect는 하지
않았으므로 “최초 버전에서 이 문서의 동일 좌표 증상까지 재현했다”는 뜻은 아니다.

## 7. 수정 설계 경계

- composer의 빈 줄 반환을 전역 변경하지 않는다. 일반 문단 fallback을 재사용한다.
- 실제 빈 문단(text 비어 있음/문단 종료만, controls 없음, 저장 줄 정보 없음, 유효 문단)의
  소유권을 줄 벡터 길이와 분리한다.
- cut 없음이면 해당 셀의 문단을 소비한다. cut 있으면 기존 CellUnit이 해당 문단을 현재
  조각에 선택했는지로 결정한다. `(0,0)`이라는 값만으로 양쪽을 같은 것으로 보지 않는다.
- CellUnit은 빈 atom에 `(0, line_count.max(1))`을 줄 수 있다. 따라서 부분 콘텐츠 높이와
  cell_was_split을 물리 줄 목록 길이만으로 계산하면 빈 문단이 0높이/가짜 split이 된다.
  같은 소유 판정을 해당 소비자들에도 일관되게 연결한다.
- 실제 cut 외 문단, control-only, 저장 LINE_SEG, 세로쓰기·기존 HWP3 경계는 보존한다.
  마지막 빈 문단의 trailing spacing은 셀 측정과 대조해 중복 소비하지 않는다.

## 8. 증적·검증 범위

`output/7032/parser_cell_probe.rs`가 parse_document 반환 Cell을 JSON으로 직렬화하고
resolve_styles/compose_paragraph 결과를 추가 기록했다. `hwp-r0c0-parser-ir.json`,
`hwpx-r0c0-parser-ir.json`, `hwp-r0c0-raw-records.json`, `hwpx-r0c0-source.json`과
`before-hwpx/render_tree_001.json`을 생성/확인했다. 실행 명령은 같은 폴더의
`parser-comparison.md`를 참조한다. 이 임시 파일 경로만이 아니라 본 보고서의 핵심 원본 값과
계보를 영구 기록으로 남긴다.

#7028의 기존 native 캐시를 재사용했다. parser/model 및 이번 style/composer/partial 경로는
현재 기준과 같다. renderer 전체에는 중첩 표 lead 보정 차이가 있으나 대상 두 문단 controls=[]여서
대상 경로에는 적용되지 않는다. 현 HEAD 전체를 새 빌드한 결과로 표시하지 않는다.

전체 Rust 회귀·WASM 빌드는 미실행이다. 조사용 작은 실행기만 컴파일했으며 제품 코드를
수정하지 않았다. 다음은 구현계획 승인 후 수정 전 실패 회귀를 준비하는 단계다.
