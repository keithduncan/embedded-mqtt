#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}