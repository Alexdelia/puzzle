use std::str::FromStr;

type Ressource = u32;
type Ant = u32

enum CellType {
    None,
    Egg,
    Crystal
}

struct Cell {
    type: CellType,
    ressource: Ressource,
    neighbor: Vec<usize>,
    my_ant: Ant,
    opp_ant: Ant
}

struct Env {
    cell: Vec<Cell>,
    n_base: usize,
    my_base: Vec<usize>,
    opp_base: Vec<usize>
}

impl FromStr for CellType {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::None),
            "1" => Ok(Self::Egg),
            "2" => Ok(Self::Crystal),
            _ => Err(())
        }
    }
}

impl FromStr for Cell {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i = s.split_whitespace();
        
        Ok(Self {
            type: i.next().parse::<CellType>().unwrap(),
            ressource: i.next().parse::<Ressource>().unwrap(),
            neighbor: i.map(|s| s.parse::<usize>().unwrap()).collect::<Vec<usize>>(),
            my_ant: 0,
            opp_ant: 0
        })
    }
}