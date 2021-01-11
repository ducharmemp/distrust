use std::convert::TryFrom;
use std::collections::VecDeque;

use anyhow::{Error, Result, ensure};
use std::collections::BTreeMap;


pub enum Command {
    Command,
    Set(String, String),
    Get(String),
    Del(Vec<String>),
    GetSet(String, String),
    LPush(String, Vec<String>),
    RPush(String, Vec<String>),
    LPop,
    RPop,
}

#[derive(Debug, Clone)]
pub enum RedisType {
    Integer(i64),
    String(String),
    List(VecDeque<RedisType>),
    Nil
}

impl RedisType {
  pub fn respond(&self) -> String {
    match self {
      Self::Integer(val) => format!(":{}\r\n", val.to_string()),
      Self::String(val) => format!("+{}\r\n", val),
      Self::List(val) => {
        let mut response = vec![];
        response.push(format!("*{}\r\n", val.len()));
        for value in val {
            response.push(value.respond());
        }

        response.join("")
      },
      Self::Nil => "$-1\r\n".to_string(),
    }
  }

  pub fn is_integer(&self) -> bool {
      match self {
          RedisType::Integer(_) => true,
          _ => false
      }
  }

  pub fn is_string(&self) -> bool {
    match self {
        RedisType::String(_) => true,
        _ => false
    }
  }

  pub fn is_list(&self) -> bool {
    match self {
        RedisType::List(_) => true,
        _ => false
    }
  }
  
  pub fn is_nil(&self) -> bool {
    match self {
        RedisType::Nil => true,
        _ => false
    }
  }

  pub fn unwrap_list(self) -> VecDeque<RedisType> {
      match self {
          RedisType::List(val) => val,
          _ => panic!("Cannot unwrap non list type")
      }
  }
}

impl From<String> for RedisType {
    fn from(other: String) -> Self {
        RedisType::String(other)
    }
}

impl From<VecDeque<RedisType>> for RedisType {
    fn from(other: VecDeque<RedisType>) -> Self {
        RedisType::List(other)
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
            "lpush" => Command::LPush(get_at(tail, 0)?, Vec::from(tail[1..].iter().cloned().collect::<Vec<_>>())),
            "rpush" => Command::RPush(get_at(tail, 0)?, Vec::from(tail[1..].iter().cloned().collect::<Vec<_>>())),
            "command" => Command::Command,
            _ => unreachable!(),
        };

        Ok(parsed_command)
    }
}

pub fn dispatch(
    command: Vec<String>,
    dictionary: &mut BTreeMap<String, RedisType>,
) -> Result<Option<RedisType>> {
    let command = Command::try_from(command)?;

    let dispatched = match command {
        Command::Command => None,
        Command::Set(name, value) => {
            dictionary.insert(name, value.into());
            Some(RedisType::Nil)
        },
        Command::Get(name) => {
          dictionary.get(&name).map(|val| val.clone()).or(Some(RedisType::Nil))
        },
        Command::GetSet(name, value) => dictionary.insert(name, value.into()).map(|val| val.clone()).or(Some(RedisType::Nil)),
        Command::Del(keys) => Some(RedisType::Integer(
            keys.iter()
                .map(|key| match dictionary.remove(key) {
                    Some(_) => 1,
                    None => 0,
                })
                .sum::<i64>()
              )
        ),
        Command::LPush(key, value) => {
            let entry = dictionary.entry(key.clone()).or_insert(VecDeque::default().into());
            ensure!(entry.is_list(), "Value stored is not a list");
            let mut entry = entry.clone().unwrap_list();
            
            for elem in value {
                entry.push_front(elem.into());
            }

            let res = Some(RedisType::Integer(entry.len() as i64));
            dictionary.insert(key, entry.into());
            res
        },
        Command::RPush(key, value) => {
            let entry = dictionary.entry(key.clone()).or_insert(VecDeque::default().into());
            ensure!(entry.is_list(), "Value stored is not a list");
            let mut entry = entry.clone().unwrap_list();
            
            for elem in value {
                entry.push_back(elem.into());
            }

            let res = Some(RedisType::Integer(entry.len() as i64));
            dictionary.insert(key, entry.into());
            res
        },
        _ => unimplemented!(),
    };

    Ok(dispatched)
}
