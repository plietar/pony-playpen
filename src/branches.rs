use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Branch {
    Release,
    Regions,
}

impl Branch {
    pub fn image(&self) -> &'static str {
        match *self {
            Branch::Release => "ponylang-playpen:latest",
            Branch::Regions => "plietar/ponylang-playpen:regions",
        }
    }
}

impl FromStr for Branch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "release" => Ok(Branch::Release),
            "regions" => Ok(Branch::Regions),
            _ => Err(format!("unknown branch {}", s)),
        }
    }
}
