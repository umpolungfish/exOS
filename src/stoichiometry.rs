/// Stoichiometric mode for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stoichiometry {
    /// 1:1 - Exclusive, high-isolation resources
    OneOne = 0,
    /// n:n - Pooled, homogeneous resources
    NN = 1,
    /// n:m - Shared, heterogeneous resources
    NM = 2,
}

impl Stoichiometry {
    pub fn name(&self) -> &'static str {
        match self {
            Self::OneOne => "1:1",
            Self::NN => "n:n",
            Self::NM => "n:m",
        }
    }
    pub fn from_primitive_index(idx: u8) -> Self {
        match idx {
            0 => Self::OneOne,
            1 => Self::NN,
            _ => Self::NM,
        }
    }
}
