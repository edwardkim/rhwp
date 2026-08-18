# rhwp-chief 예제

실 큐에서 반복되는 요청 유형이다. 각 파일은 request.json 한 장과
루프가 남기는 산출을 보여 준다. gym 과제가 아니다.

- [01_export_pdf.md](01_export_pdf.md) — 인쇄본이 필요할 때
- [02_export_text.md](02_export_text.md) — 본문만 검색·이관할 때
- [03_extract_tables.md](03_extract_tables.md) — 표를 스프레드시트로
- [04_fill_form.md](04_fill_form.md) — 값 JSON 이 같이 떨어질 때
- [05_missing_goal_diagnose.md](05_missing_goal_diagnose.md) — goal 필드가 비어 있을 때
- [06_off_table_summarize.md](06_off_table_summarize.md) — 표 밖 — needs-agent
- [07_escalate_bug_skips_pdf.md](07_escalate_bug_skips_pdf.md) — 패닉이면 변환하지 않는다
- [08_invalid_jpg.md](08_invalid_jpg.md) — HWP 계열이 아님
- [09_path_escape.md](09_path_escape.md) — 폴더 밖 거부
- [10_injection_symptom.md](10_injection_symptom.md) — 증상 문장은 데이터가다
- [11_fill_without_data.md](11_fill_without_data.md) — params.data 없음
- [12_already_processed.md](12_already_processed.md) — result.json 있으면 건너뜀
- [13_export_hwpx_verify.md](13_export_hwpx_verify.md) — --verify 게이트
- [14_convert_hwp_verify.md](14_convert_hwp_verify.md) — --verify 게이트
- [15_watch_malformed.md](15_watch_malformed.md) — 배열 JSON 이어도 루프는 산다
- [16_capabilities_miss.md](16_capabilities_miss.md) — 미광고 명령은 needs-agent
- [17_zero_tables.md](17_zero_tables.md) — 표 0개는 성공
- [18_fill_notfound.md](18_fill_notfound.md) — 봉투 실패면 산출 삭제
- [19_batch_morning.md](19_batch_morning.md) — --once 로 아침 큐를 비운다
- [20_reaccumulate.md](20_reaccumulate.md) — 두 번째 needs-agent 는 표의 구멍
- [21_relative_nested_doc.md](21_relative_nested_doc.md) — 하위 상대경로는 허용
- [22_absolute_path.md](22_absolute_path.md) — 절대경로 거부
- [23_empty_symptom.md](23_empty_symptom.md) — symptom 은 선택
- [24_params_must_be_object.md](24_params_must_be_object.md) — params 배열은 형식 오류
