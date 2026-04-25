# PR #282 처리 결과 보고서 (인수·머지 완료)

## PR 정보

| 항목 | 내용 |
|------|------|
| PR 번호 | [#282](https://github.com/edwardkim/rhwp/pull/282) |
| 작성자 | [@seanshin](https://github.com/seanshin) (Shin hyoun mouk) |
| 이슈 | [#279](https://github.com/edwardkim/rhwp/issues/279) |
| 처리 | **Merge (admin)** — 메인테이너 인수 후 마무리 |
| 처리일 | 2026-04-25 |
| Merge commit | `edcf361` |

## 처리 경위

1. **2026-04-24**: 작성자 PR 제출 (분석 정확, 구현 OK, 그러나 devel 충돌 + 범위 외 파일 다수 + 24h+ 응답 부재)
2. **2026-04-25 1차**: 메인테이너가 close 처리 + 인수 결정
3. **2026-04-25 2차**: 작업지시자 의견으로 PR reopen + 작성자 fork 직접 정리 인수 방식으로 전환
4. **2026-04-25 3차**: `local/task279` 브랜치 (origin/devel 기준) 에 작성자 핵심 3 커밋 cherry-pick + 메인테이너 5 커밋 추가 → force-push → admin merge

## 작성자 분석 (정확하고 가치 있음)

[@seanshin](https://github.com/seanshin) 의 핵심 분석 2가지:

1. **리더 도트 모양**: `fill_type=3` (점선) 가 사각 대시처럼 렌더되는 버그 → `dasharray="0.1 3" stroke-linecap="round" width="1.0"` 로 원형 점 표현 (한컴 동등)
2. **소제목 right tab 정렬**: `find_next_tab_stop` 의 일률 `available_width` 클램핑이 들여쓰기 문단의 RIGHT 탭 (`tab_type=1`) 위치를 잘못 좌측으로 이동시키는 문제 → `tab_type != 1` 가드 추가

**HWP 스펙은 데이터 포맷 정의일 뿐 한컴 조판 알고리즘은 비공개** 라는 본질적 통찰을 바탕으로 한 분석.

## 메인테이너 추가 보강 (6가지)

작성자 분석 위에 시각 검증 사이클로 6가지 추가 결함 식별·해결:

3. **trailing 공백 \t 케이스**: `run.text.ends_with('\t')` 가드가 `\t ` 형태 (한컴 목차 소제목) 를 놓침 — `trim_end_matches(' ').ends_with('\t')` 로 정밀화. cross-run RIGHT 진입 활성화
4. **리더 시멘틱**: 한컴은 리더 (`fill_type ≠ 0`) RIGHT 탭을 "이 줄 우측 끝까지" 의미로 재해석 — `effective_pos = effective_margin_left + available_width` (cell inner 우측 끝 강제). 셀 padding_right 영역 침범 해소
5. **페이지번호 폭별 leader 길이**: 한 자리/두 자리 무관 leader 끝이 같은 x 라 페이지번호와 겹침 — cross-run RIGHT take 시점에 leader.end_x 단축
6. **공백 only run carry-over**: 장제목 케이스 (`"...\t" + " " + "3"`) 의 ` ` 단독 run 은 정렬 단위 아님 — pending 을 다음 의미있는 run 으로 carry-over
7. **leader-bearing TextRun 검색**: 공백 carry-over 후 leader.end_x 보정 시 직전 TextRun = 공백 run (leader 없음) 이라 한 단계 위 leader 가진 TextRun 을 검색해야 함
8. **선행 공백 시각 보정**: 장제목 두 자리 케이스 (`" 16"` 한 run) — `next_w` 를 trim_start 가 아닌 전체 run 폭 으로 사용 (draw_text 가 공백 포함 텍스트 그릴 때 페이지번호 right edge 가 effective_pos 와 정확히 일치)

## 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 992 passed / 0 failed |
| `cargo test --test svg_snapshot` | ✅ 6 passed (UPDATE_GOLDEN 후) |
| `cargo test --test issue_301` | ✅ |
| `cargo clippy --lib -- -D warnings` | ✅ |
| `cargo check --target wasm32` | ✅ |
| 7 핵심 샘플 페이지 수 회귀 | ✅ 모두 무변화 |
| KTX 목차 시각 검증 (작업지시자) | ✅ 한컴과 동등 |

## 기여 인정 (7가지 산출물)

1. **Cherry-pick author 보존**: 3 커밋 (`f27477e` / `2eb1be5` / `4770a8a`) author=hyoun mouk shin
2. **Co-Authored-By 체인**: 메인테이너 5 신규 커밋에 `Co-Authored-By: hyoun mouk shin` trailer
3. **CHANGELOG.md**: "분석·구현 by [@seanshin](https://github.com/seanshin)"
4. **HWP Spec Errata entry 30**: TabDef.position 시멘틱 — 발견·구현 본인 명기
5. **위키 페이지 신규**: `HWP-Tab-Leader-Rendering.md` (본인 크레딧 머리말)
6. **트러블슈팅 문서**: `toc_leader_right_tab_alignment.md` (외부 기여 인정 섹션)
7. **최종 보고서**: 머리말 + stage3 보고서 인용 보존

## 머지 정보

- merge commit: `edcf361208886f6f52d46d08e1829a28b0f4f5e8`
- merge type: admin merge (metadata 유지, squash 미사용 — 작성자 author 보존)
- 이슈 #279 close (commit message `closes #279` 인식 안 됨 → 수동 close)

## 참고 링크

- [PR #282](https://github.com/edwardkim/rhwp/pull/282)
- [메인테이너 인수 안내 코멘트](https://github.com/edwardkim/rhwp/pull/282#issuecomment-4318148845)
- [최종 머지 안내 코멘트](https://github.com/edwardkim/rhwp/pull/282#issuecomment-4318579479)
- 최종 보고서: `mydocs/report/task_m100_279_report.md`
- 트러블슈팅: `mydocs/troubleshootings/toc_leader_right_tab_alignment.md`
- HWP Spec Errata entry 30
- 위키: [HWP Tab Leader Rendering](https://github.com/edwardkim/rhwp/wiki/HWP-Tab-Leader-Rendering)
