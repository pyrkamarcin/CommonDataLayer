use uuid::Uuid;

#[must_use]
#[derive(Clone, Debug, PartialEq)]
pub enum Resolution {
    StorageLayerFailure {
        description: String,
        object_id: Uuid,
    },
    CommandServiceFailure {
        object_id: Uuid,
    },
    UserFailure {
        description: String,
        context: String,
        object_id: Uuid,
    },
    Success,
}
