use crate::Float;

// no need for derivative as no back propagation is planned

// ANSWER START
pub type Activation = fn(Float) -> Float;

pub fn sigmoid(x: Float) -> Float {
    1.0 / (1.0 + (-x).exp())
}

pub fn relu(x: Float) -> Float {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}
// ANSWER END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid() {
        assert_eq!(sigmoid(0.0), 0.5);
        assert_eq!(sigmoid(1.0), 0.7310586);
        assert_eq!(sigmoid(-1.0), 0.26894143);
        assert_eq!(sigmoid(0.5), 0.62245935);
    }

    #[test]
    fn test_relu() {
        assert_eq!(relu(0.0), 0.0);
        assert_eq!(relu(1.0), 1.0);
        assert_eq!(relu(-1.0), 0.0);
        assert_eq!(relu(0.5), 0.5);
    }
}
