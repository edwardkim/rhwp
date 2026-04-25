# Task #279 Stage 3 — 시각 검증 + 좌표 측정 + 회귀

원본 [@seanshin](https://github.com/seanshin) 의 stage3 보고서 (`d48af5c` cherry-pick) 를 메인테이너가 인수 후 추가 분석·수정 결과로 보강.

## 작성자 stage3 보고 (인용)

작성자 보고 결과:
- 장제목 페이지번호 x: 717.5 (변화 없음 보고)
- 소제목 페이지번호 x: ~700 → 717.9 (보고)
- 차이 ~17px → 0.4px (보고)

→ 메인테이너 재실측 결과 작성자 보고와 차이 발견 (다음 섹션 참조).

## 메인테이너 재실측 (작성자 분석 검증 + 한컴 의도 추가 분석)

### 좌표 측정 (KTX 목차 페이지)

| 항목 | Devel (Before) | After 1차 (작성자 fix) | After 최종 (메인테이너 보강) |
|------|---------------|----------------------|--------------------------|
| 장제목 ("3") x | 709.76 | 727.47 | **690.76** |
| 소제목 한 자리 ("4") x | 689.88 | 689.88 (무변화) | 690.76 |
| 소제목 두 자리 ("14") x | 681.43 | 681.43 (무변화) | 680.76 |
| 장제목 두 자리 ("16") 첫글자 x | 690.09 | 690.09 (무변화) | 681.09 |

→ **모든 페이지번호 right edge ≈ 700.0 으로 정렬 통일**.

### 작성자 stage3 와 실측 충돌 사유

작성자 보고는 자기 환경 SVG 의 다른 측정 기준 (line bbox 우측 edge 또는 다른 페이지) 으로 추정. 메인테이너 실측에서는 작성자 변경만으로 소제목 정렬은 **미해결** 이었다. 이는 cross-run RIGHT tab pending 가드 (`run.text.ends_with('\t')`) 가 trailing 공백 케이스 (`\t `) 를 놓쳐 소제목 path 가 cross-run 진입 자체를 못 했기 때문.

### 추가 식별된 7가지 결함 + 각 보강

| # | 결함 | 메인테이너 보강 |
|---|------|---------------|
| 1 | (작성자) 리더 도트 사각 대시 | (작성자) `dasharray="0.1 3" linecap="round" width="1.0"` |
| 2 | (작성자) RIGHT 탭 일률 클램핑 | (작성자) `tab_type != 1` 가드 |
| 3 | trailing 공백 \t 케이스 누락 | `trim_end_matches(' ').ends_with('\t')` 가드 정밀화 (est/render) |
| 4 | 리더 시멘틱 부재 (셀 padding 침범) | `resolve_last_tab_pending` 시그니처에 `fill_type` 추가, leader 있는 RIGHT 탭은 `effective_margin_left + available_width` 로 강제 |
| 5 | 리더 길이가 페이지번호 폭 무시 | cross-run RIGHT take 시점에 leader-bearing TextRun 검색 + leader.end_x 단축 |
| 6 | 공백 only run 정렬 부적합 | 공백 only run 은 carry-over (정렬 단위 아님) |
| 7 | 선행 공백 (`" 16"`) 시각 보정 부재 | `next_w` 를 trim 하지 않은 전체 run 폭으로 |

## 회귀 샘플 (페이지 수)

| 샘플 | Devel | After | 결과 |
|------|-------|-------|------|
| 21_언어_기출_편집가능본.hwp | 15 | 15 | ✅ |
| exam_math.hwp | 20 | 20 | ✅ |
| exam_kor.hwp | 24 | 24 | ✅ |
| exam_eng.hwp | 9 | 9 | ✅ |
| basic/KTX.hwp | 1 | 1 | ✅ |
| aift.hwp | 74 | 74 | ✅ |
| biz_plan.hwp | 6 | 6 | ✅ |

모두 무변화.

## 골든 svg_snapshot 영향

| 골든 | 영향 | 처리 |
|------|------|------|
| `issue-267/ktx-toc-page.svg` | KTX 목차 (의도된 변경 — 본 task 핵심) | UPDATE_GOLDEN |
| `issue-147/aift-page3.svg` | aift 표 안 leader (의도된 통일 — dasharray 만 변경, 좌표 동일) | UPDATE_GOLDEN |
| 기타 4건 | 무영향 | 통과 |

## WASM 시각 검증

WASM Docker 빌드 (`pkg/rhwp_bg.wasm`) 후 작업지시자 직접 시각 확인:
- 장제목/소제목 페이지번호 정렬 일치 ✅
- 한 자리/두 자리 페이지번호 leader 길이 차등화 ✅
- 셀 padding_right 영역 침범 해소 ✅
- 두 자리 페이지번호 (장제목 16/20/24) 정렬 일치 ✅

## 트러블슈팅 등록

본 task 의 7가지 결함 + 보강 패턴 + 교훈 (HWP 스펙 ≠ 한컴 조판 알고리즘) 을 `mydocs/troubleshootings/toc_leader_right_tab_alignment.md` 에 등록.

## Stage 3 완료 조건 점검

- [x] 좌표 측정 결과 한컴 의도와 정합 (작업지시자 시각 확인)
- [x] 7 핵심 샘플 회귀 0
- [x] svg_snapshot 6/6 통과
- [x] WASM 빌드 + 시각 확인 통과
- [x] 트러블슈팅 등록

## 다음 단계

Stage 4 — 최종 보고서 + CHANGELOG + 위키 + Errata + orders + force-push + admin merge.
