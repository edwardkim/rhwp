/** 글자 서식 경계에서만 사용자에게 안내하는 오류. 다른 편집 오류는 숨기지 않는다. */
export class CharFormatError extends Error {}

export class CharFormatRecoveryError extends AggregateError {
  constructor(errors: unknown[]) {
    super(errors, '글자 서식 복원이 일부 완료되지 않았습니다. 실행 취소(Undo)로 복원을 다시 시도해 주세요.');
  }
}

export function isCharFormatError(error: unknown): error is CharFormatError | CharFormatRecoveryError {
  return error instanceof CharFormatError || error instanceof CharFormatRecoveryError;
}
