use clap::ValueEnum;
use strum::Display;

#[derive(ValueEnum, Display, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Copy, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

impl Visibility {
    pub fn as_arg(&self) -> String {
        match self {
            Visibility::Public => "--public".into(),
            Visibility::Private => "--private".into(),
        }
    }
}
