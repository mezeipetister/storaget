// Copyright (C) 2019 Peter Mezei
//
// This file is part of Project A.
//
// Project A is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// Project A is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Project A.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt;

pub type StorageResult<T> = Result<T, Error>;

pub enum Error {
    InternalError(String),
}

// Well formatted display text for users
// TODO: Use error code and language translation for end-user error messages.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_display() {
        let err = Error::InternalError("test error".into());
        assert_eq!(format!("{}", err), "Internal error: test error");
    }
    #[test]
    fn test_debug() {
        let err = Error::InternalError("test error".into());
        assert_eq!(format!("{:?}", err), "Internal error: test error");
    }
}
