# 한컴 기준 PDF 변환 기록

2026-09-14 `hwp2024-mcp-convert start → status → download`로 생성했다. 모든 job은 `succeeded/completed` 확인 뒤 내려받았고 서버/로컬 SHA-256이 일치한다. engine `2020`, Hancom `12.0.0.4605`, `hwp-managed-direct-dll-host`, `frame_print_to_pdf_ex_one_up`, print method `0`, input preprocessing `none`, Session 0 글꼴 등록·자식 프로세스 가시성 `verified`다. 저장 메타데이터가 2022 이하이므로 engine 2020을 선택했다. 토큰·서버 주소는 기록하지 않는다.

| PDF | 원본 | 쪽수 | SHA-256 | job |
| --- | --- | ---: | --- | --- |
| [cancel-request-hwp-2020.pdf](cancel-request-hwp-2020.pdf) | `tests/fixtures/issue_7081/3079571-cancel-request-form.hwp` | 1 | `2807271033a1572196836d3a16c4f4bc8e6e647c71ae373b770f9e962207c156` | `d8c3f237-d924-49e9-b201-ae380f120f81` |
| [incheon-trade-hwp-2020.pdf](incheon-trade-hwp-2020.pdf) | `tests/fixtures/issue_7097/148759031-incheon-customs-trade.hwp` | 5 | `ceedb1d01088b5e1425d5a0a5be5c1b8dc96f65951b647ed4fe4ca7787df7423` | `035391e2-19dd-460b-b686-05cf91aaef3f` |
| [objection-form-hwp-2020.pdf](objection-form-hwp-2020.pdf) | `tests/fixtures/issue_7081/3030681-objection-form.hwp` | 1 | `4cf7ccf29b4154bfa77323626791772afc29770fb3e3ee16825162d781c32cd2` | `bf74680e-5357-45c7-ab6e-546ff612831e` |
| [soil-quarry-hwp-2020.pdf](soil-quarry-hwp-2020.pdf) | `tests/fixtures/issue_7092/soil-quarry-permit-standard.hwp` | 6 | `5d2fdb6c82503d2f4908f0781d381490302d5a207333c260b45869090e193b5b` | `b2cede1a-af25-4d37-8158-be1f9e7f0464` |
| [synth-no-lineseg-hwp-2020.pdf](synth-no-lineseg-hwp-2020.pdf) | `tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp` | 3 | `70b63c11514d3d927418c1f116a0cc421614e7db5e9de312bd0d9f964b34e9a3` | `5ace3123-31ed-45b4-abeb-6be057bfe0fb` |
| [vat-bank-hwp-2020.pdf](vat-bank-hwp-2020.pdf) | `tests/fixtures/issue_7081/1319800-vat-bank-designation.hwp` | 12 | `4e8305da812aa2a1ee471103fb001e608501736ccf5d61372e31fb007751ebce` | `a6ef97d7-0bb5-430f-a8ce-109eb6e3abc1` |

이 PDF는 동일 입력의 독립 한컴 출력이다. 테스트 통과나 PDF 생성 성공 자체를 rhwp의 시각 일치 판정으로 쓰지 않는다. 각 원 PR review에서 변경 전·후와 같은 페이지·영역을 직접 대조한다.
