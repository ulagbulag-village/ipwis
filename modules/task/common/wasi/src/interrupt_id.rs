#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterruptId(pub &'static str);

impl ::core::fmt::Display for InterruptId {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "InterruptHandler({})", &self.0)
    }
}
