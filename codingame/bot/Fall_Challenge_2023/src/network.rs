pub struct Network {
    pub layers: Vec<Layer>,
}

pub struct Layer {
    pub weight: Matrix,
    pub bias: Matrix,
}
