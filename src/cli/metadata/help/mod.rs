//! 사람용 `--help` 출력의 순서 보존 조립 경계.

mod edit;
mod protocol;
mod public;

pub(crate) fn print_help() {
    public::print();
    edit::print();
    protocol::print();
}
