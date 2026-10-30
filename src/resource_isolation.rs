
/// Stoichiometry (S) accessor
impl crate::aleph_kernel_types::AlephKernelType {
    pub fn stoichiometry(&self) -> u8 { self.tuple[10] }
    pub fn interaction_grammar(&self) -> u8 { self.tuple[7] }
}

/// Resource isolation via Stoichiometry.
pub enum ResourceIsolation {
    Exclusive, // 1:1
    Homogeneous, // n:n
    Heterogeneous, // n:m
}

impl ResourceIsolation {
    pub fn from_type(aleph_type: &crate::aleph_kernel_types::AlephKernelType) -> Self {
        match aleph_type.stoichiometry() {
            0 => Self::Exclusive,
            1 => Self::Homogeneous,
            _ => Self::Heterogeneous,
        }
    }
}
