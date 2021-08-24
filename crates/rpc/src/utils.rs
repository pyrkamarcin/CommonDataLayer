#[macro_export]
macro_rules! rpc_enum {
    (
        $name: ident,
        $inner: path,
        $inner_field: ident,
        $display: literal,
        $sql: literal,
        [ $($variant: ident),* ]
    ) => {

        #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, sqlx::Type, async_graphql::Enum, derive_more::Display)]
        #[sqlx(type_name = $sql, rename_all = "lowercase")]
        #[serde(rename_all = "UPPERCASE")]
        pub enum $name {
            $($variant),*
        }

        impl From<$inner> for $name {
            fn from(op: $inner) -> Self {
                match op {
                    $(<$inner>::$variant => Self::$variant),*
                }
            }
        }

        impl From<$name> for super::$name {
            fn from(op: $name) -> Self {
                Self {
                    $inner_field: match op {
                        $(<$name>::$variant => <$inner>::$variant.into()),*
                    },
                }
            }
        }

        impl std::str::FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($variant) => Ok(Self::$variant),
                    )*
                    invalid => Err(anyhow::anyhow!("Invalid {}: {}", $display, invalid)),
                }
            }
        }

        impl From<$name> for i32 {
            fn from(o: $name) -> i32 {
                let o: super::$name = o.into();
                o.$inner_field
            }
        }

        impl std::convert::TryFrom<super::$name> for $name {
            type Error = anyhow::Error;

            fn try_from(variant: super::$name) -> Result<Self, Self::Error> {
                if false { unreachable!() } //In case there are no variants
                $( if variant.$inner_field == { let v: i32 = <$inner>::$variant.into(); v } { return Ok(Self::$variant) })*
                else {  Err(anyhow::anyhow!(concat!("Invalid ", $display))) }
            }
        }

    };
}
