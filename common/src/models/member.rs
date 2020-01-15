#[cfg(feature = "diesel_impl")]
use diesel_derives::Queryable;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

pub type MemberId = i32;

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone, PartialEq, Eq)]
pub struct Member {
    pub id: MemberId,
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
    pub pid: String,
    pub email: String,
    pub phone_number: String,
    pub address: String,
    pub zip_code: String,
    pub city: String,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct NewMember {
    pub first_name: String,
    pub last_name: String,
    pub nickname: Option<String>,
    pub pid: String,
    pub email: String,
    pub phone_number: String,
    pub address: String,
    pub zip_code: String,
    pub city: String,
}
