# 검증 구현과 최종 문서의 동일성

이 동일성은 제출 후보 `e68f0e1236`까지의 문서 정리에 적용한다. 이후 [진단 출력 보안 보정](ci-security-followup.md)은 로그 세 곳과 CLI 회귀를 별도로 변경하며 해당 검증 기록을 따른다.

확인 시각 `2026-11-04T00:23:31+09:00`. 검증 head `c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775`에서 최종 증적 정리까지 제품 source·Cargo 설정은 동일하다. 테스트 파일의 변경은 아래 첫머리 네 줄 중 갱신 시각과 증거 링크 두 줄뿐이며 네 줄 뒤의 바이트는 같다. 실제 assertion·fixture include·실행 경로를 바꾸지 않았다.

따라서 이미 통과한 전체 12,465건·Native·최적화 WASM 결과는 같은 구현에 대한 실행 증거로 유지한다. 최종 문서·저장 증적 입력과 첫머리 링크는 별도 fmt·policy·workspace lint·경로 검사를 수행한다. GitHub에서는 실제 제출 head의 새 CI를 확인한다.

| 파일 | 검증 당시 전체 SHA-256 | 최종 전체 SHA-256 | 동일한 5행 이후 SHA-256 |
| --- | --- | --- | --- |
| `tests/cases/no_lineseg_inline_object_caret_anchor.rs` | `3582e3db5ab6c2031acc710cc521dbafa210123d3af074a8c1b6f7fc0219fffd` | `93ab3f47ac192064dca12cace112cc4b393cb53d6dd978439420d7b76810de44` | `d7aff2df8bca45ee8af15d97c69f9e10b811f53e42c0796593e19e5cac66d376` |
| `tests/cases/table_insert_body_flow.rs` | `83f6a566360d61bfaf433edf88584a73184ed2e5f4529eb5e545d2caecffa317` | `8b915a8cb4077d2fa0092ea8580d6334faf8bc89bceff132b6980a00323c16d2` | `aa92368abf7c34b9f03b2a12a425455c49cb001b3bfe2c709f67bcc88e28b875` |
| `tests/cases/nested_wide_table_clip.rs` | `cbe585c540de2e637867e31078ebb845a727b39ac01735b476e0fea3d13fadab` | `d9878a6996bd5ce3548b18e98f697ec1da9d61937fd5fce11302561f077f6c61` | `fbf13a5f986b9f0a8f9ea955683e072fabf7a421dfb7b08ae8d534f3d9aa72d4` |
| `tests/cases/blank_nested_table_html.rs` | `7301fa5caa0586795e68c67d7ce1ae7b338c6085a2f497f2dbe9b66f84741cbd` | `dd4e4bc929ce79a2d6ff1e57aae5e64b8129bfca56eb8d6dfaa191dab2aeba95` | `8c8d739e15b6f957ebcc45fb0196807ef555cb1373975659733bb7ac5db10e19` |
| `tests/cases/form_inline_editing.rs` | `44b82b9d5e284ddcdc973aba906d4104cdcb0d31e1b0ad06a4a6a17dbb2480a4` | `2a319fa93d04ee6aac12a503e9983e6d73ee6c22f62ece6bbdee8c72129f4d1e` | `1b63e4e07890744d34ab5a7b107b5ebe54cf4b50629fcf4b1f30ff1fbefbeb30` |
| `tests/cases/polygon_transform_roundtrip.rs` | `1c1e829338378094abe7b8bca8649b3a81d2150646d13f36ad9b9b5720b0763a` | `a7bf0eda494a25068cfe93245c8d6263575fbbf6ffffac49ec1b6c6d0adf9281` | `5c15351200fe6ed448c5fec1b23d606b32d9fc976986ac13be3b07e752878451` |
| `tests/cases/table_insert_measurement_ownership.rs` | `d80ff3d97abe1f3786b71e5b8be37b63eb3e0bed1f346bf788659b4fadebf3e7` | `ce3b4bb4fb3b666aecb3b2703fd476c5f8b0af978277710672ac22257c91b25d` | `7de72185e4bba4dcaa4dfa590b56e0814b322e850e72ea50ec1db2a4e2c32770` |
| `tests/cases/split_row_clip_hit.rs` | `530595e0bfbcf7a05f2435c3829507e8f313abea0b4c022dc25e2f5a4a1f0d6d` | `84c55a2ac254d34a005e769af0d9c75028355b2b5492b784dc43d7d1047c2c4f` | `fcfe2f03a8b72079044f1a903f533ddd1652d3db4d3bb63b6f2a41530e5687df` |
| `tests/cases/header_overflow_paragraphs_render.rs` | `343dd8e256cefbc7d89b6dc333a7a707ea2a7d7f4a477b0735624b272bdc54ab` | `e02bf995a4b8cd6977f0f982731ccef8bccaf3432a9f8c04271609e63cfe33d5` | `bc6b335b267e153a744092e431f7c15f403a635f9ad74e789fab59aaff7bf942` |
| `tests/cases/hancom_table_grow_saves_table_height.rs` | `7da3481bd9a833a65249b1fa94c71fab4bce1bbd7b97d4fc3cb1ac24861dbb72` | `530adb3cbdb580893e2f276a591634e4783f36e01e4d5ba0376444105423d7dc` | `684f9becc046af488b44aad0db6c31766f93a04f1fc3cd03b2816e282ddba667` |
| `tests/cases/composer_without_stored_lines.rs` | `85c850f3baac513a43dc7f228af5be60e0c7c16f3a7b3a35721d9711a7935e2d` | `9f3fd4b9f2574b0a696b3ee4f45e53ea4bd6f3767c845268bdabe0495380996a` | `6b85e64bb35096d1cc688d67fae15ebe477f271bf7f9f40063adea66f0e52556` |
| `tests/cases/paragraph_spacing_moves_stored_lines.rs` | `d662348714a686edc067ffbb6ad2881316e4cc3ec26cc79ea1fa888fcd720a9f` | `a5aa2586d5c988a312a8b763739d9fdddc657732a6d64dc33b26e990faf8ea00` | `9ee2b8378febbf5088ff71594513cb53bff5eeeee21a1e7af44f7bf7b3b2f188` |
