use std::fmt;
use std::hash::Hash;

use irc::client::data;
use serde::{Deserialize, Serialize};

use crate::buffer;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "String")]
#[serde(try_from = "String")]
pub struct User(data::User);

impl Eq for User {}

impl Hash for User {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.get_nickname().hash(state);
        self.0.get_username().hash(state);
        self.0.get_hostname().hash(state);
    }
}

impl Ord for User {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.highest_access_level()
            .cmp(&other.highest_access_level())
            .reverse()
            .then_with(|| self.nickname().cmp(&other.nickname()))
    }
}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<String> for User {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl<'a> TryFrom<&'a str> for User {
    type Error = &'static str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(data::User::new(value)))
    }
}

impl From<User> for String {
    fn from(user: User) -> Self {
        let nick = user.nickname();

        match (user.username(), user.hostname()) {
            (None, None) => nick.to_string(),
            (None, Some(host)) => format!("{nick}@{host}",),
            (Some(user), None) => format!("{nick}!{user}"),
            (Some(user), Some(host)) => format!("{nick}!{user}@{host}"),
        }
    }
}

impl User {
    pub fn new(nick: Nick, user: Option<&str>, host: Option<&str>) -> Self {
        let formatted = match (user, host) {
            (None, None) => nick.to_string(),
            (None, Some(host)) => format!("{nick}@{host}"),
            (Some(user), None) => format!("{nick}!{user}"),
            (Some(user), Some(host)) => format!("{nick}!{user}@{host}"),
        };

        Self(data::User::new(&formatted))
    }

    pub fn color_seed(&self, color: &buffer::Color) -> Option<String> {
        match color {
            buffer::Color::Solid => None,
            buffer::Color::Unique => Some(
                self.hostname()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| self.nickname().as_ref().to_string()),
            ),
        }
    }

    pub fn username(&self) -> Option<&str> {
        self.0.get_username()
    }

    pub fn nickname(&self) -> NickRef {
        NickRef(self.0.get_nickname())
    }

    pub fn hostname(&self) -> Option<&str> {
        self.0.get_hostname()
    }

    pub fn highest_access_level(&self) -> AccessLevel {
        AccessLevel(self.0.highest_access_level())
    }

    pub fn formatted(&self) -> String {
        let user = self.username();
        let host = self.hostname();
        let nick = self.nickname();

        match (user, host) {
            (None, None) => nick.to_string(),
            (None, Some(host)) => format!("{nick} ({host})"),
            (Some(user), None) => format!("{nick} ({user})"),
            (Some(user), Some(host)) => format!("{nick} ({user}@{host})"),
        }
    }
}

impl From<data::User> for User {
    fn from(user: data::User) -> Self {
        Self(user)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.nickname())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nick(String);

impl fmt::Display for Nick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> From<&'a str> for Nick {
    fn from(nick: &'a str) -> Self {
        Nick(nick.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NickRef<'a>(&'a str);

impl<'a> From<&'a str> for NickRef<'a> {
    fn from(nick: &'a str) -> Self {
        NickRef(nick)
    }
}

impl<'a> NickRef<'a> {
    pub fn to_owned(self) -> Nick {
        Nick(self.0.to_string())
    }
}

impl<'a> fmt::Display for NickRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> AsRef<str> for NickRef<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> PartialOrd for NickRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.to_lowercase().cmp(&other.0.to_lowercase()))
    }
}

impl<'a> Ord for NickRef<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.to_lowercase().cmp(&other.0.to_lowercase())
    }
}

impl<'a> PartialEq<Nick> for NickRef<'a> {
    fn eq(&self, other: &Nick) -> bool {
        self.0.eq(other.0.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct AccessLevel(data::AccessLevel);

impl std::fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let access_level = match self.0 {
            data::AccessLevel::Owner => "~",
            data::AccessLevel::Admin => "&",
            data::AccessLevel::Oper => "@",
            data::AccessLevel::HalfOp => "%",
            data::AccessLevel::Voice => "+",
            data::AccessLevel::Member => "",
        };

        write!(f, "{}", access_level)
    }
}

impl From<data::AccessLevel> for AccessLevel {
    fn from(access_level: data::AccessLevel) -> Self {
        Self(access_level)
    }
}

impl PartialEq for AccessLevel {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for AccessLevel {}

impl Ord for AccessLevel {
    fn cmp(&self, other: &AccessLevel) -> std::cmp::Ordering {
        use std::cmp::Ordering::{Equal, Greater, Less};

        use irc::client::data::AccessLevel::{Admin, HalfOp, Member, Oper, Owner, Voice};

        if self == other {
            return Equal;
        }

        let other = other.0;
        match self.0 {
            Owner => Greater,
            Admin => {
                if other == Owner {
                    Less
                } else {
                    Greater
                }
            }
            Oper => {
                if other == Owner || other == Admin {
                    Less
                } else {
                    Greater
                }
            }
            HalfOp => {
                if other == Voice || other == Member {
                    Greater
                } else {
                    Less
                }
            }
            Voice => {
                if other == Member {
                    Greater
                } else {
                    Less
                }
            }
            Member => Less,
        }
    }
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
