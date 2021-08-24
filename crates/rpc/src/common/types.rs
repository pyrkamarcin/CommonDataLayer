use crate::codegen::common::{logic_operator, search_for};

rpc_enum! {
    SearchFor,
    search_for::Direction,
    search_for,
    "relation direction",
    "search_for_enum",
    [
        Parents,
        Children
    ]
}

rpc_enum! {
    LogicOperator,
    logic_operator::Operator,
    operator,
    "logic operator",
    "logic_operator_enum",
    [
        And,
        Or
    ]
}
