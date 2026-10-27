---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase13-survey-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase13-survey-report.md` · 원문 SHA-256: `cea32e1f2fc99f4af36da7f978cb1aeb244d72ba4fb6b083220cbd153cbd9bdf`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## 추가 조사: 잘못된 셀을 검사하던 회귀 발견 — 2026-10-26

기존 검사의 PASS를 그대로 승인하지 않고 원 결함과 실제 관측 대상을 대조했습니다.

`tests/issue_table_vpos_01_page5_cell_hit_test.rs`의 `page5_inner_11x3_char_overlap_marker_advances_one_box`는 10번 글자겹침을 검사한다고 되어 있지만, 경로가 **19번 셀의 일반 숫자4**를 가리키고 있었습니다. 10번의 두 PUA 제어는 **22번 셀의 첫 문단**에 있습니다.

| 현재 실제 원본/커서 | 기존 검사의19번 셀 | 실제10번의22번 셀 |
| --- | --- | --- |
| 원본 내용 | 일반 텍스트 `4`, CharOverlap 없음 | `U+F02BA + U+F02C3` CharOverlap |
| 원본 글자 크기 | 3100HU | 1400HU |
| offset0→1 전진 | 23.9px | 18.7px |
| 기존 `16<delta<30` | PASS | 이 셀은 기존 검사에서 관측하지 않음 |

독립 CFB/DocInfo의 CHAR_SHAPE 레코드에서도 id79=3100HU, id68=1400HU를 확인했습니다. 실제 마커의 한 em은18.6666667 CSSpx입니다. 현재 API의18.7은 소수 첫째 자리 출력과 구분했습니다. 단순히16~30 범위를 조정하는 것으로는 잘못된 소유자 선택을 고칠 수 없습니다.

추가로 같은 실제 마커를 현재 일반 scalar 경로로 계산하면 첫 전진이20.5333333px가 되어 기존16~30 범위가 이 잘못된 경로까지 받아들임을 확인했습니다. 실제 한 상자18.6666667px와 구분하지 못하므로 셀 번호만22로 바꿔도 충분하지 않습니다. 같은 소유자의 paint 끝과 커서 끝을 연결하는 관계는 정상 끝203.1과 선언 끝203.08을 받아들이면서 일반 scalar 경로의1.8666667px 오차를 검출했습니다. 이 정밀도는 cursor JSON의 소수 첫째 자리 직렬화 오차에서 도출했으며 PDF 절대 위치 핀이 아닙니다. 현재 경로의 관계식 반례 관찰이고, 과거 전체 바이너리/production mutation 또는 새 정식 회귀 실행으로 주장하지 않습니다.


원 해결은 [6c887fc79887d16eded0d8fadae174e7350bad1b](https://github.com/edwardkim/rhwp/commit/6c887fc79887d16eded0d8fadae174e7350bad1b), 설명은 `mydocs/report/archives/task_m100_1023_report.md`입니다. 최초 추가 때부터 검사 본문이 현재와 동일하고 HWP 입력 SHA도 바뀌지 않았습니다. 후속 #3386의 클릭점 관계화 및 #7941 수정에서도 이 경로는 남았습니다. 해당 커밋과 뒤의 v0.7.14 Release 집계 PR #1294 merge가 현재 head의 조상임을 확인했습니다. #1294를 이 기능의 개별 해결 PR이라고 부르지 않습니다.

기존 두 모듈은 **35 PASS / 0 FAIL**이지만 이 잘못된 검사도 포함되어 있습니다. 따라서 PASS만으로 원 결함 보호를 입증할 수 없다는 실제 사례입니다. `issue_8232` 기준으로 재작성할 대상으로 확정했으며, 실제 원본 제어/payload·논리offset·같은 셀 소유·한 상자 전진을 검사할 계획입니다. 제품/테스트 source 수정과 개명, 실제 mutation 전후 검증은 아직 수행하지 않았습니다.

### 이번에 확보한 시각/단언 근거

현재 head는 `238de6de41a208648f4ade6b494fe433a81c1fce`이며 작업 트리는 clean입니다.

| 독립 Print와 같은 정확한 입력 | 전쪽 | Native 최저 raw | fresh WASM 최저 raw | 미달/누락/추가 |
| --- | ---: | ---: | ---: | --- |
| table-vpos-01.hwp | 5 | 99.93951% | 99.94159% | 0/0/0 |
| indent20444 편집 | 1 | 100.00000% | 100.00000% | 0/0/0 |
| 폭12000HU·접두문자 편집 표 | 1 | 100.00000% | 100.00000% | 0/0/0 |

PDFium5.13.0/96CSSdpi/2px raw의 전쪽 TSV입니다. review 그림에 이미 적힌 반올림 수치가 아닌 별도 전쪽 TSV 원값을 기록했습니다. 두 편집 HWP를 현재 Native 라이브러리로 재생성하여 기존 독립 Print 입력과 바이트 동일함을 확인했습니다. Mac fresh WASM은 web/no-opt 로컬 대체 빌드입니다.

기존 #7490 제목/표 ±2px 검사2개는 현재 자격을 확인했습니다. 제목의 독립 원점은 `(413.535888671875,167.81656901041666)` CSSpx이고, 표의 두 의미 셀 clip 합집합은 `(326.064,630.3506666666667,159.91466666666665,132.17466666666672)`입니다. 현재 구현 bbox를 정답으로 복사하지 않았습니다. 양쪽 review/overlay 및 Native→WASM→독립 PDF의2배 확대에서 원문·줄 소속·prefix/표 순서·내용 보존을 확인했습니다. 글리프 획/힌팅·table-vpos 제목/번호의 글꼴 모양 차이는 남습니다.

다른 prefix/Enter 성장 입력의 쪽수 근거와 로그의 다른 문서 overflow 진단은 별도 미완료입니다. 이 두 위치 검사 유지 근거를 파일 전체 완료로 확대하지 않습니다.

### 원장과 남은 작업

배치·CAS·CLI 직접 단언87그룹과 SVG/LayerTree의 현재 호출 결정성·직접 선언 clip/paint·글꼴 원본 단위 계약을 추가 분류했습니다. 현재 원장은 **9,899그룹·2,103파일**, 미분류는 **4,985그룹**입니다. 이 숫자는 위반 건수나 PASS 수가 아닙니다. 시각 증거 목록은 **91조건·각 backend2,493쪽**입니다.

조사 기록은 ignored `output/issue8232-audit/`의 `table-vpos-marker-audit-finding.json`, `table-vpos-raw-char-shapes.json`, `phase13-existing-pins-qualification.json`, `phase13-independent-indent-oracles.json`, 전쪽 TSV와 review/overlay, 실제 실행 로그에 보존했습니다.

제품·테스트 보정/개명은 아직0건이고 PR은 생성하지 않았습니다. 전수 판정 후 실제 보정·등록/호출 갱신·필수 재검증까지 계속 진행합니다.
