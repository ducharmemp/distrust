use std::convert::TryFrom;

use anyhow::{Error, Result};
use std::collections::BTreeMap;

pub enum Command {
    Command,
    Set(String, String),
    Get(String),
    Del(Vec<String>),
    GetSet(String, String),
}

#[derive(Debug)]
pub enum RedisType {
    Integer(i64),
    String(String),
    Nil
}

impl RedisType {
  pub fn respond(self) -> String {
    match self {
      Self::Integer(val) => format!("(integer) {}", val.to_string()),
      Self::String(val) => val,
      Self::Nil => "(nil)".to_string(),
    }
  }
}

impl TryFrom<Vec<String>> for Command {
    type Error = Error;

    fn try_from(other: Vec<String>) -> Result<Command> {
        let (head, tail) = other.split_at(1);
        let head = head.get(0).ok_or(Error::msg("No command found"))?;
        let get_at = |parts: &[String], index: usize| -> Result<String> {
            let part = parts.get(index);
            let part = part.ok_or(Error::msg("Syntax error"));

            Ok(part?.clone())
        };

        let parsed_command = match head.to_lowercase().as_str() {
            "set" => Command::Set(get_at(tail, 0)?, get_at(tail, 1)?),
            "get" => Command::Get(get_at(tail, 0)?),
            "del" => Command::Del(Vec::from(tail)),
            "getset" => Command::GetSet(get_at(tail, 0)?, get_at(tail, 1)?),
            "command" => Command::Command,
            _ => unreachable!(),
        };

        Ok(parsed_command)
    }
}

pub fn dispatch(
    command: Vec<String>,
    dictionary: &mut BTreeMap<String, String>,
) -> Result<Option<RedisType>> {
    let command = Command::try_from(command)?;

    let dispatched = match command {
        Command::Command => None,
        Command::Set(name, value) => {
            dictionary.insert(name, value);
            Some(RedisType::Nil)
        },
        Command::Get(name) => {
          dictionary.get(&name).map(|val| RedisType::String(val.clone())).or(Some(RedisType::Nil))
        },
        Command::GetSet(name, value) => dictionary.insert(name, value).map(|val| RedisType::String(val.clone())).or(Some(RedisType::Nil)),
        Command::Del(keys) => Some(RedisType::Integer(
            keys.iter()
                .map(|key| match dictionary.remove(key) {
                    Some(_) => 1,
                    None => 0,
                })
                .sum::<i64>()
              )
        ),
        _ => unimplemented!(),
    };

    Ok(dispatched)
}
