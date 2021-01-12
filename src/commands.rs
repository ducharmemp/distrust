use std::collections::VecDeque;
use std::convert::TryFrom;
use std::cow::Cow;

use anyhow::{ensure, Error, Result};

use crate::types::{RedisType, UnwrappedDictionary};

struct Foo { bar: i64; }

pub enum Command<'a> {
    Command,
    Set(&'a str, &'a str),
    Get(&'a str),
    Del(&'a[&'a str]),
    GetSet(&'a str, &'a str),
    LPush(&'a str, &'a[&'a str]),
    RPush(&'a str, &'a[&'a str]),
    LPop,
    RPop
}

impl<'a> TryFrom<&'a[&'a str]> for Command<'a> {
    type Error = Error;

    fn try_from(other: &'a[&'a str]) -> Result<Command<'a>> {
        let (head, tail) = other.split_at(1);
        let head = &head.get(0).ok_or(Error::msg("No command found"))?;
        let get_at = |parts: &'a[&'a str], index: usize| -> Result<&str> {
            let part = parts.get(index);
            let part = part.ok_or(Error::msg("Syntax error"));

            Ok(part?)
        };

        let parsed_command = match head.to_lowercase().as_str() {
            "set" => Command::Set(&get_at(tail, 0)?, &get_at(tail, 1)?),
            "get" => Command::Get(&get_at(tail, 0)?),
            "del" => Command::Del(&tail),
            "getset" => Command::GetSet(&get_at(tail, 0)?, &get_at(tail, 1)?),
            "lpush" => Command::LPush(
                &get_at(tail, 0)?,
                &tail[1..],
            ),
            "rpush" => Command::RPush(
                &get_at(tail, 0)?,
                &tail[1..],
            ),
            "command" => Command::Command,
            _ => unreachable!(),
        };

        Ok(parsed_command)
    }
}

pub fn dispatch(
    command: Vec<String>,
    dictionary: &UnwrappedDictionary,
) -> Result<Option<RedisType>> {
    let command = Command::try_from(command)?;
    let mut dictionary = dictionary
        .lock()
        .map_err(|_| Error::msg("Could not acquire global dictionary"))?;

    let dispatched = match command {
        Command::Command => None,
        Command::Set(name, value) => {
            dictionary.insert(name, value.into());
            Some(RedisType::Nil)
        }
        Command::Get(name) => dictionary
            .get(&name)
            .map(|val| val.clone())
            .or(Some(RedisType::Nil)),
        Command::GetSet(name, value) => dictionary
            .insert(name, value.into())
            .map(|val| val.clone())
            .or(Some(RedisType::Nil)),
        Command::Del(keys) => Some(RedisType::Integer(
            keys.iter()
                .map(|key| match dictionary.remove(key) {
                    Some(_) => 1,
                    None => 0,
                })
                .sum::<i64>(),
        )),
        Command::LPush(key, value) => {
            let entry = dictionary
                .entry(key.clone())
                .or_insert(VecDeque::default().into());
            ensure!(entry.is_list(), "Value stored is not a list");
            let mut entry = entry.clone().unwrap_list();

            for elem in value {
                entry.push_front(elem.into());
            }

            let res = Some(RedisType::Integer(entry.len() as i64));
            dictionary.insert(key, entry.into());
            res
        }
        Command::RPush(key, value) => {
            let entry = dictionary
                .entry(key.clone())
                .or_insert(VecDeque::default().into());
            ensure!(entry.is_list(), "Value stored is not a list");
            let mut entry = entry.clone().unwrap_list();

            for elem in value {
                entry.push_back(elem.into());
            }

            let res = Some(RedisType::Integer(entry.len() as i64));
            dictionary.insert(key, entry.into());
            res
        }
        _ => unimplemented!(),
    };

    Ok(dispatched)
}
