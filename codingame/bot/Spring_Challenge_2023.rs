use std::str::FromStr;

type Ressource = u32;
type Ant = u32;

enum CellType {
    None,
    Egg,
    Crystal
}

struct Cell {
    r#type: CellType,
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
        let mut i = s.split_whitespace();
        
        Ok(Self {
            r#type: i.next().unwrap().parse::<CellType>().unwrap(),
            ressource: i.next().unwrap().parse::<Ressource>().unwrap(),
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

																						
																																																												#[cfg(test)]
																																																												mod test {
																																																												    use super::*;
																																																													    
																																																														    fn test_cell_parse() {
																																																															        assert_eq!(
																																																																	            "0 0 1 2 3 4 5 6".parse::<Cell>(),
																																																																				            Cell {
																																																																							                type: CellType::None,
																																																																											                ressource: 0,
																																																																															                neighbor: vec![1, 2, 3, 4, 5, 6],
																																																																																			                my_ant: 0,
																																																																																							                opp_ant: 0
																																																																																											            }
																																																																																														        )
																																																																																																        
																																																																																																		        assert_eq!(
																																																																																																				            "1 0 7 8 9 10 11 12".parse::<Cell>(),
																																																																																																							            Cell {
																																																																																																										                type: CellType::Egg,
																																																																																																														                ressource: 0,
																																																																																																																		                neighbor: vec![7, 8, 9, 10, 11, 12],
																																																																																																																						                my_ant: 0,
																																																																																																																										                opp_ant: 0
																																																																																																																														            }
																																																																																																																																	        )
																																																																																																																																			        
																																																																																																																																					        assert_eq!(
																																																																																																																																							            "2 42 13 14 15 2 1 0".parse::<Cell>(),
																																																																																																																																										            Cell {
																																																																																																																																													                type: CellType::Crystal,
																																																																																																																																																	                ressource: 42,
																																																																																																																																																					                neighbor: vec![13, 14, 15, 2, 1, 0],
																																																																																																																																																									                my_ant: 0,
																																																																																																																																																													                opp_ant: 0
																																																																																																																																																																	            }
																																																																																																																																																																				        )
																																																																																																																																																																						    }
																																																																																																																																																																							}