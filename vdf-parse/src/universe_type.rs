use crate::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum UniverseType {
    Invalid = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
    Max = 5,  
}