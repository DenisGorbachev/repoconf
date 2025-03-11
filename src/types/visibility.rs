use clap::ValueEnum;

#[derive(ValueEnum, Ord, PartialOrd, Eq, PartialEq, Default, Hash, Clone, Copy, Debug)]
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
