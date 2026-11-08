---
kind: reference
status: active
canonical: mydocs/tech/native_worker_recovery.md
last_verified: 2026-11-07
---

# Native worker 자동 복구와 복구 파일 열기

P95는 P85의 prepare → publish → token cleanup 흐름을 재사용한다.
`qt_native_host.py --recovery-dir <전용 폴더>`로 활성화하며 기본 간격은 30초다.
깨끗한 문서, 같은 revision, IME 조합 중, 저장·문서 교체·다른 I/O·입력 대기 중에는 시작하지 않는다.
실패도 간격 제한을 적용한다. GUI 이벤트 루프는 prepare/poll과 buffer 복사·해제만 담당하고
파일 쓰기·검증은 기존 단일 I/O executor에서 실행한다.

복구는 HWPX bytes와 해당 bytes의 손실 보고·revision·원본 fingerprint를 RHWP-R1 파일에
함께 보관한다. 손실이 있더라도 원본을 덮어쓰지 않고 복구 가능한 내용을 보관하며,
복구 파일을 열 때 그 보고를 사용자에게 보여 준다. 완료 시 `cancelSave`를 받고,
`acknowledgeSave`를 호출하지 않는다. 현재 dirty 상태·원본 경로·저장 checkpoint를 바꾸지 않는다.
오래된 문서의 cleanup 오류는 이미 게시한 파일을 보존하고, 같은 token을 두 번 취소하지 않는다.

전용 폴더는 최대 256개 항목만 조사하고 복구 파일 32개·총 256 MiB를 admission 한도로 사용한다.
한도에 도달하면 기존 복구 파일을 보존한 채 새 보관을 중단하고 상태 메시지를 표시한다.
자동 삭제는 하지 않는다. 이 검사는 단일 참조 호스트의 직렬 I/O를 위한 것으로, 여러 프로세스의
동시 쓰기에 대한 원자적 디스크 quota는 아니다. 개별 payload 128 MiB와 metadata 64 KiB
제한, exclusive publication·동기화·재읽기 검증은 기존 저장 계약을 따른다.

‘복구 파일 열기’는 사용자가 명시적으로 고른 파일의 signature·길이·SHA-256·metadata를
I/O executor에서 검사한다. 기존 문서의 미저장 변경 확인과 교체 revision barrier도 적용한다.
metadata의 sourcePath는 provenance로만 표시하며 원본을 읽거나 덮어쓸 경로로 채택하지 않는다.
확인 후 `NativeRuntimeCommand::OpenRecovery` / `workerOpenRecovery`로 payload를 열어
revision 0·saved_revision=None인 새 문서를 만든다. 편집을 꾸며 dirty 상태를 만들지 않으므로
가짜 undo 기록도 없다. 이후 사용자가 저장 위치를 골라 정상 저장해야 checkpoint가 생긴다.

검증은 worker adapter·실제 파일 I/O·Qt의 busy/IME/timer/실패/복구 선택 계약과 공개 Rust
recovery import·잘못된 replacement 보존 회귀를 포함한다. 실제 OS 종료·전원 손실·전체 한컴
문서군 검증은 별도 qualification이다.
