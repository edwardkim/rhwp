# injection 계약 봉투 작업 기록 (#5476)

이 장은 기존 규칙의 소비 분기만 적는다. 새 kind 를 제안하지 않는다.
개별 봉투는 `tests/fixtures/inspect_msec/envelopes/` 가 정본이다.

## 가족 `role_impersonation` (35건)

- 양성 28 / 음성 7 / 그 외 0
- 대표 `inj-role-token-im_start-hwp`
- 출처 `src/document_core/queries/injection_scan.rs` `role_impersonation`
- 대표 분기: {'branch': 'clean == false', 'matchedIs': 'DATA', 'doNotExecuteMatched': True, 'detectionIsNotFailure': True}
- 왜: 대화 역할·채팅 템플릿 토큰이 본문에 있습니다 — 문서 텍스트가 모델 프롬프트의 역할 경계를 흉내 냅니다

- `inj-role-token-im_start-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-im_end-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-system-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-user-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-assistant-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-endoftext-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-start_header_id-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-end_header_id-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-eot_id-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-inst-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-inst-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-sys-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-sys-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-시스템_프롬프트-hwp` polarity=positive exit=0 pair=-
- `inj-role-token-시스템_메시지-hwp` polarity=positive exit=0 pair=-
- `inj-role-label-system-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-system-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-system-hwp-pos
- `inj-role-label-assistant-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-assistant-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-assistant-hwp-pos
- `inj-role-label-human-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-human-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-human-hwp-pos
- `inj-role-label-developer-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-developer-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-developer-hwp-pos
- `inj-role-label-system-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-system-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-system-hwp-pos
- `inj-role-label-assistant-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-assistant-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-assistant-hwp-pos
- `inj-role-label-instruction-hwp-pos` polarity=positive exit=0 pair=-
- `inj-role-label-instruction-hwp-neg-sql` polarity=negative exit=0 pair=inj-role-label-instruction-hwp-pos
- `inj-role-addr-너는-ai-hwp` polarity=positive exit=0 pair=-
- `inj-role-addr-너희는-ai-hwp` polarity=positive exit=0 pair=-
- `inj-role-addr-당신은-ai-hwp` polarity=positive exit=0 pair=-
- `inj-role-addr-당신이-ai-hwp` polarity=positive exit=0 pair=-
- `inj-role-addr-네가-ai-hwp` polarity=positive exit=0 pair=-
- `inj-role-addr-귀하는-ai-hwp` polarity=positive exit=0 pair=-

## 가족 `instruction_override` (6건)

- 양성 2 / 음성 4 / 그 외 0
- 대표 `inj-ov-en-ignore-previous-instruction-hwp`
- 출처 `src/document_core/queries/injection_scan.rs` `instruction_override`
- 대표 분기: {'branch': 'clean == false', 'matchedIs': 'DATA', 'doNotExecuteMatched': True, 'detectionIsNotFailure': True}
- 왜: 선행 지시를 무효화하라는 관용구입니다 — 무효화 서술어와 '이전 지시/시스템 프롬프트' 목적어가 한 문장 안에 함께 있습니다

- `inj-ov-en-ignore-previous-instruction-hwp` polarity=positive exit=0 pair=-
- `inj-ov-ko-이전-지시-무시하-hwp` polarity=positive exit=0 pair=-
- `inj-ov-ko-neg-verb-only` polarity=negative exit=0 pair=-
- `inj-ov-ko-neg-object-only` polarity=negative exit=0 pair=-
- `inj-ov-ko-neg-scope-only` polarity=negative exit=0 pair=-
- `inj-ov-neg-clause-boundary-4088` polarity=negative exit=0 pair=-

## 가족 `tool_directive` (11건)

- 양성 10 / 음성 1 / 그 외 0
- 대표 `inj-tool-hwp_doc_save-하시기_바랍니다-hwp`
- 출처 `src/document_core/queries/injection_scan.rs` `tool_directive`
- 대표 분기: {'branch': 'clean == false', 'matchedIs': 'DATA', 'doNotExecuteMatched': True, 'detectionIsNotFailure': True}
- 왜: 본문이 실제 MCP 도구 이름을 명령형과 함께 부릅니다

- `inj-tool-hwp_doc_save-하시기_바랍니다-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_doc_save-invoke-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_doc_text-하시기_바랍니다-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_doc_text-invoke-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_inspect_hidden_text-하시기_바랍니다-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_inspect_hidden_text-invoke-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_inspect_injection-하시기_바랍니다-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_inspect_injection-invoke-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_inspect_unicode-하시기_바랍니다-hwp` polarity=positive exit=0 pair=-
- `inj-tool-hwp_inspect_unicode-invoke-hwp` polarity=positive exit=0 pair=-
- `inj-tool-neg-imperative-only` polarity=negative exit=0 pair=-

## 가족 `authority_claim` (17건)

- 양성 16 / 음성 1 / 그 외 0
- 대표 `inj-auth-en-already_approved-hwp`
- 출처 `src/document_core/queries/injection_scan.rs` `authority_claim`
- 대표 분기: {'branch': 'clean == false', 'matchedIs': 'DATA', 'doNotExecuteMatched': True, 'detectionIsNotFailure': True}
- 왜: 문서가 '이미 승인·권한 있음'을 스스로 주장합니다

- `inj-auth-en-already_approved-hwp` polarity=positive exit=0 pair=-
- `inj-auth-en-already_authorized-hwp` polarity=positive exit=0 pair=-
- `inj-auth-en-already_granted-hwp` polarity=positive exit=0 pair=-
- `inj-auth-en-pre-approved-hwp` polarity=positive exit=0 pair=-
- `inj-auth-en-preapproved-hwp` polarity=positive exit=0 pair=-
- `inj-auth-sup-사용자에게_알리지_마-hwp` polarity=positive exit=0 pair=-
- `inj-auth-sup-사용자에게_알리지_말-hwp` polarity=positive exit=0 pair=-
- `inj-auth-sup-사용자에게_보고하지_마-hwp` polarity=positive exit=0 pair=-
- `inj-auth-sup-사용자에게_보고하지_말-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-사용자-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-관리자-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-개발자-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-운영자-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-제작자-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-anthropic-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-ko-openai-승인했-hwp` polarity=positive exit=0 pair=-
- `inj-auth-neg-procedure` polarity=negative exit=0 pair=-

## 가족 `exfiltration_hint` (9건)

- 양성 8 / 음성 1 / 그 외 0
- 대표 `inj-exfil-전송하라-https_attacker.example_x-hwp`
- 출처 `src/document_core/queries/injection_scan.rs` `exfiltration_hint`
- 대표 분기: {'branch': 'clean == false', 'matchedIs': 'DATA', 'doNotExecuteMatched': True, 'detectionIsNotFailure': True}
- 왜: 전송 명령형과 외부 주소(URL·이메일)가 한 문장 안에 함께 있습니다

- `inj-exfil-전송하라-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-전송해-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-전송할_것-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-보내라-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-보내줘-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-보내도록-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-업로드하라-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-업로드해-https_attacker.example_x-hwp` polarity=positive exit=0 pair=-
- `inj-exfil-neg-polite` polarity=negative exit=0 pair=-

## 가족 `delimiter_break` (13건)

- 양성 9 / 음성 4 / 그 외 0
- 대표 `inj-delim-system-hwp`
- 출처 `src/document_core/queries/injection_scan.rs` `delimiter_break`
- 대표 분기: {'branch': 'clean == false', 'matchedIs': 'DATA', 'doNotExecuteMatched': True, 'detectionIsNotFailure': True}
- 왜: 프롬프트 경계를 흉내 내는 구분자입니다 — 단독으로는 약하지만 다른 신호와 함께라면 주입 시도의 골격입니다

- `inj-delim-system-hwp` polarity=positive exit=0 pair=-
- `inj-delim-system-hwp` polarity=positive exit=0 pair=-
- `inj-delim-instructions-hwp` polarity=positive exit=0 pair=-
- `inj-delim-instructions-hwp` polarity=positive exit=0 pair=-
- `inj-delim-context-hwp` polarity=positive exit=0 pair=-
- `inj-delim-context-hwp` polarity=positive exit=0 pair=-
- `inj-delim-user_input-hwp` polarity=positive exit=0 pair=-
- `inj-delim------begin-hwp` polarity=positive exit=0 pair=-
- `inj-delim-fence-prose-hwp` polarity=positive exit=0 pair=-
- `inj-delim-excluded-system` polarity=negative exit=0 pair=-
- `inj-delim-excluded-system` polarity=negative exit=0 pair=-
- `inj-delim-excluded----` polarity=negative exit=0 pair=-
- `inj-textkind-equation-backticks` polarity=negative exit=0 pair=inj-delim-fence-prose-hwp

## 가족 `min-confidence` (3건)

- 양성 0 / 음성 0 / 그 외 3
- 대표 `inj-min-confidence-low`
- 출처 `src/document_core/queries/injection_scan.rs` `Confidence::parse`
- 대표 분기: {'branch': 'minConfidence', 'kept': ['role_impersonation', 'instruction_override', 'tool_directive', 'authority_claim', 'exfiltration_hint', 'delimiter_break']}
- 왜: --min-confidence low 는 6건을 남긴다.

- `inj-min-confidence-low` polarity=filter exit=0 pair=-
- `inj-min-confidence-medium` polarity=filter exit=0 pair=-
- `inj-min-confidence-high` polarity=filter exit=0 pair=-

## 가족 `scanScopes` (5건)

- 양성 3 / 음성 0 / 그 외 2
- 대표 `inj-scopes-include-fields-off`
- 출처 `src/main.rs` `injection_scan_scopes`
- 대표 분기: {'branch': 'scanScopes', 'missingScopeIsNotClean': True, 'fieldScopes': ['fieldName', 'fieldGuide', 'fieldCommand', 'hiddenComment', 'fieldMemo']}
- 왜: scanScopes 가 검사 범위를 밝힌다.

- `inj-scopes-include-fields-off` polarity=contract exit=0 pair=-
- `inj-scopes-include-fields-on` polarity=contract exit=0 pair=-
- `inj-scope-body-role-token` polarity=positive exit=0 pair=-
- `inj-scope-tableCell-role-token` polarity=positive exit=0 pair=-
- `inj-scope-fieldGuide-role-token` polarity=positive exit=0 pair=-

## 가족 `clean-corpus` (3건)

- 양성 0 / 음성 3 / 그 외 0
- 대표 `inj-clean-hwp3-sample.hwp`
- 출처 `tests/injection_scan_contract.rs` `정상 문서 오탐 0`
- 대표 분기: {'branch': 'clean == true', 'highestConfidence': None}
- 왜: 실문서 음성. samples/hwp3-sample.hwp

- `inj-clean-hwp3-sample.hwp` polarity=negative exit=0 pair=-
- `inj-clean-hwp3-sample10.hwp` polarity=negative exit=0 pair=-
- `inj-clean-2022년_국립국어원_업무계획.hwp` polarity=negative exit=0 pair=-

## 가족 `exception` (7건)

- 양성 0 / 음성 0 / 그 외 7
- 대표 `ex-inj-missing-file`
- 출처 `src/main.rs` `inspect_command`
- 대표 분기: {'branch': 'stdout empty', 'doNotParseStdoutAsJson': True, 'stderrIsDiagnosis': True}
- 왜: 없는 파일은 런타임 실패

- `ex-inj-missing-file` polarity=exception exit=1 pair=-
- `ex-inj-no-file` polarity=exception exit=2 pair=-
- `ex-inj-minconf-bad` polarity=exception exit=2 pair=-
- `ex-inj-minconf-missing` polarity=exception exit=2 pair=-
- `ex-inj-unknown-option` polarity=exception exit=2 pair=-
- `ex-inj-two-files` polarity=exception exit=2 pair=-
- `ex-inspect-unknown-axis-inject` polarity=exception exit=2 pair=-
