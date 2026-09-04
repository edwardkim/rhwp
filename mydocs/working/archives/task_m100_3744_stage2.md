# Stage 2 보고 — task_m100_3744 red 고정·정책 비교

- **일자**: 2026-08-03
- **이슈**: #3744
- **기준**: `upstream/devel` `0889974a01db3585df8ad2c1f13203e3cb9f51f8`
- **브랜치**: `codex/issue-3744-clause-context-confidence`
- **승인 범위**: 영구 red, 정책 비교, 단계 보고까지. 제품 구현 제외
- **단계 결론**: 정책 선택 완료, 승인 B 대기

## 1. 영구 red 판별력

`tests/issue_3744_structure_clause_confidence.rs`에 8개 테스트를 추가했다.

| 결과 | 테스트 | 의미 |
| --- | --- | --- |
| fail | `stale_anchor_does_not_promote_oracle_sql_steps` | Oracle SQL 2303이 여전히 `호` |
| fail | `compound_number_boundary_expires_only_the_current_weak_anchor` | `2.1`과 경계 뒤 `1)`이 여전히 `호` |
| fail | `revision_date_inside_article_remains_body_text` | 날짜가 두 번째 node |
| fail | `direct_mok_under_section_is_a_heading` | synthetic direct `목` 누락 |
| fail | `real_handbook_keeps_representative_direct_mok_headings` | 편람 44개 군 대표 누락 |
| fail | `real_market_report_keeps_matching_direct_mok_heading` | 다른 shape-compatible 실문서 대표 누락 |
| pass | `toc_mok_with_page_tail_remains_body_text` | TOC negative 현행 보존 |
| pass | `agreement_keeps_normal_ho_under_article` | 협정서 `호` 14개 positive 보존 |

실행 결과는 **2 passed / 6 failed**, exit 101이다. 실패는 구현 전 의도한 red이며 제품 코드 회귀가
아니다. 각 negative는 node 부재뿐 아니라 body text 보존을 함께 단언한다.

## 2. 앵커 정책 비교

recursive sample 668개 중 665개를 파싱했고 현행 `호`는 6,219개다. password 문서 3개는 기존과
같이 실패 목록으로 분리했다.

| 후보 | 전체 제거 | 변경 문서 | Oracle `호` | 협정서 `호` | 편람 `호` | 판정 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| section reset | 388 | 12 | 1,228 | 14 | 57 | 기각 — SQL을 못 막고 편람 손실 |
| 거리 32 | 5,321 | 41 | 5 | 14 | 13 | 기각 — 임의 threshold·대량 손실 |
| strict sequence | 5,456 | 55 | 29 | 14 | 17 | 기각 — 재개 가능하면서 대량 손실 |
| restart `1` 허용 | 1,050 | 51 | 1,146 | 14 | 238 | 기각 — target SQL 잔존 |
| 첫 불연속 terminal | 5,461 | 41 | 4 | 14 | 6 | 기각 — 반복 목록 손실 |
| **`.` 복합 번호 경계** | **4,351** | **6** | **4** | **14** | **240** | **채택 권고** |

### 2.1 선택 규칙

Oracle에서 첫 네 후보는 `1.`~`4.`이고, 다음 문단은 `4.1 create the standby database`다. 현행
classifier는 이를 marker `4.`인 `호`로 잘라 이후 SQL 목록까지 같은 `항` anchor 아래에서 허용한다.

- `.` marker 직후가 숫자이면 해당 `N.N` 문단을 복합 번호 boundary로 거부한다.
- 현재 nearest `조|항`의 weak-`호` 상태를 만료해 후속 후보를 body로 보낸다.
- 새 `조|항`이 열리면 새 anchor 상태로 시작한다.
- `)` 뒤 숫자로 본문이 시작하는 `1)1920년…`은 boundary가 아니다.

최종 영향 6개는 `hwp3-sample10`의 HWP/HWP5/HWPX 3개와 `hwp3-sample11`의 세 형식 변형이다.
두 원본 모두 Oracle/Unix 기술문서이며 첫 boundary 뒤 제거량은 각각 1,224와 227/226/226개다.
초기 probe에서 `)`까지 넓혔을 때 `SO-SUEOP` 2개가 잘못 바뀌었고, `.`로 제한한 뒤 영향에서
사라졌다. 편람 240개와 협정서 14개는 모두 보존된다.

잔여 trade-off는 Oracle boundary 전 `1.`~`4.` 네 개가 여전히 `호`라는 점이다. 거리나 strict
sequence로 이를 함께 없애면 편람의 실제 반복 목록을 대량 손실하므로 이번 정책은 좁은 경계를 택한다.

## 3. 날짜 정책

`YYYY. M. D.`를 연도 4자리, 월 1~12, 일 1~31 범위로 파싱하는 lexical negative를 선택한다.
suffix는 판정에 쓰지 않으며 날짜 거부로 anchor를 만료하지 않는다. recursive corpus의 현행 날짜형
`호`가 0개이므로 synthetic `제1조 → 2022. 1. 1. 일부개정`이 최소 oracle이다.

## 4. `목` 정책 비교

열린 `호`가 없고 `장|절` 조상이 있는 미채택 `목` 후보를 전수 측정했다.

| 후보 | 수 | 문서 수 | 비고 |
| --- | ---: | ---: | --- |
| broad parent 허용 | 304 | 4 | 편람 HWP/HWPX 각 128, 연구보고서 25, 시장구조조사 23 |
| TOC tail 제외 | 304 | 4 | 실물 후보의 tail은 0; synthetic guard 필요 |
| `margin_left=0` | 117 | 3 | 편람의 깊은 hanging indent 6개 포함 |
| **margin 0 + indent ≥ -1280 + level 0** | **111** | **3** | **채택 권고** |

선택 111개는 편람 HWP/HWPX 각각 44개와 시장구조조사 23개다. 편람 수치는 발견 코멘트와 정확히
일치하며 `가. ‘업무’의 개념`, `나. 문서의 필요성`, `가. 어문 규범에 맞게 쓰기`를 포함한다.
시장구조조사의 `가. 표준산업분류`, `나. 재화특성별 분류표`도 실제 본문 제목임을 대조했다.

다음은 보수 정책의 잔여 false negative다.

- 편람의 `margin_left=3000`인 더 깊은 `가)` 하위 항목
- `issue2006/1790387_prep_final_report.hwpx`의 `margin_left=1000`, `indent=-2486` 제목 25개
- 그 밖의 broad 후보 중 direct 44 범위보다 깊은 항목

이번 이슈의 명시 완료 조건은 장/절 direct 44개 회복이므로 이들을 무조건 함께 열지 않는다. TOC
쪽번호 tail은 실물 후보에서 없었지만 `가. 개요\t9` synthetic negative로 고정한다.

## 5. 구현 경계

승인 B 뒤 Stage 3에서만 다음을 제품 코드에 넣는다.

1. `clause_heading_allowed()`에 원문·ParaShape·nearest anchor 만료 상태 전달
2. `N.N` boundary 거부와 anchor별 weak 상태 만료/새 `조|항` reset
3. 유효 날짜 lexical reject
4. direct `목`의 tail/ParaShape gate

공개 JSON, auto selector, explicit outline, parser/render/serializer는 변경하지 않는다.

## 6. 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --test issue_3744_structure_clause_confidence` | 의도한 red: 2 passed / 6 failed, exit 101 |
| `cargo test --lib document_core::queries::structure` | 6 passed |
| `cargo test --test issue_3693_structure_clause_context` | 3 passed |
| `cargo clippy --test issue_3744_structure_clause_confidence -- -D warnings` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |

Stage 2는 red checkpoint이므로 전체 release-test는 실행하지 않는다. Stage 3 green 뒤 focused 전체와
Stage 4 release-test를 수행한다.

## 7. 승인 지점

제품 소스는 변경하지 않았다. 조사 probe는 삭제했고 영구 red와 계획·보고서만 남겼다. 위 세 선택
정책과 명시한 trade-off를 작업지시자가 승인해야 Stage 3 구현을 시작한다.
