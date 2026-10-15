# 초기 #7653 바깥 여백 편집 입력 보존

기존 정식 검사의 실제 저장본이 ignored capture 경로에만 있어 PR 재현을 위해 **같은 바이트**를 보존했다. 새 합성 입력이나 기대값을 만든 것이 아니다.
현재 production ddf17f864의 초기48개 입력 Native/fresh 비교는 원래 capture 파일과 같은 독립 Print를 사용한다. 완료 결과는 통합 review에 기록하며 파일 보존 자체를 최종 시각 승인으로 취급하지 않는다.

- `outer-margin-edit.hwp`: 입력 SHA-256 `c5d54efe1d6d10a58d200250c3a2fb409144efa05b332e8361f5bcd760c750ad`; 독립 Print `pdf/semanticist21-20261008/table_properties_outer_margin_sync-0522-c5d54efe1d6d-2020.pdf`, SHA-256 `6a53375c45a68d6bf5fb53bdb8f7572e97fb04cd4955660d26def1c5aa6ec604`. 원 capture `output/pr-review/semanticist21-20261008/captured-fixtures/table_properties_outer_margin_sync--outer_margin_edit_moves_table_now_and_after_reopen/0522.hwp`.
- `outer-margin-edit.hwpx`: 입력 SHA-256 `c083046f7120f86b4e39cbd1e524fee6b1be789ca1680cc0f5e702fe52b610d4`; 독립 Print `pdf/semanticist21-20261008/table_properties_outer_margin_sync-0523-c083046f7120-2020.pdf`, SHA-256 `3a6ccff438807eb571e9468312ba80fa2d68ac3f5fcb18c343653b3e39fb8e58`. 원 capture `output/pr-review/semanticist21-20261008/captured-fixtures/table_properties_outer_margin_sync--outer_margin_edit_moves_table_now_and_after_reopen/0523.hwpx`.
