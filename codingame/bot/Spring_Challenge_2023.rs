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
            neighbor: i.map(|s| s.parse::<usize>().unwrap()).collect::<Vec<usize>>(), // does a cell always has 6 neighbors? // TODO
            my_ant: 0,
            opp_ant: 0
        })
    }
}

impl Env {
	    fn new() -> Self {
		        let mut buf = String::new();

				        stdin().read_line(&mut buf).unwrap();
						        let n_cell = buf.parse::<usize>();
								        
										        let cell = Vec::with_capacity(n_cell);
												        
														        for i in 0..n_cell {
																            stdin().read_line(&mut buf).unwrap();
																			            
																						            cell[i] = buf.parse::<Cell>();
																									        }
																											        
																													        stdin().read_line(&mut buf).unwrap();
																															        let n_base = buf.parse::<usize>();
																																	        stdin().read_line(&mut buf).unwrap();
																																			        let my_base = buf.split_whitespace().map(|s| s.parse::<usize>().unwrap()).collect();
																																					        stdin().read_line(&mut buf).unwrap();
																																							        let my_base = buf.split_whitespace().map(|s| s.parse::<usize>().unwrap()).collect();
																																									        
																																											        Env {
																																													            cell,
																																																            n_base,
																																																			            my_base,
																																																						            opp_base
																																																									        }
																																																											    }
																																																												}