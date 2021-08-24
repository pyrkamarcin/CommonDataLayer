use crate::codegen::edge_registry::simple_filter_side;

rpc_enum! {
    SimpleFilterSide,
    simple_filter_side::Side,
    side,
    "simple filter side",
    "simple_filter_side_enum",
    [
        InParentObjIds,
        InChildObjIds
    ]
}
