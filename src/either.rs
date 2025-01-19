
pub enum Either<T, U> {
    Left(T),
    Right(U)
}

impl<T, U> Either<T, U> {
    pub fn is_left(&self) -> bool {
        match self {
            Either::Left(_) => true,
            Either::Right(_) => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            Either::Left(_) => false,
            Either::Right(_) => true,
        }
    }

    pub fn left(&self) -> Option<&T> {
        match self {
            Either::Left(x) => Some(x),
            Either::Right(_) => None,
        }
    }

    pub fn right(&self) -> Option<&U> {
        match self {
            Either::Left(_) => None,
            Either::Right(y) => Some(y),
        }
    }
}