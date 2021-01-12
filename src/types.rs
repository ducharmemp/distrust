use std::collections::{VecDeque, BTreeMap};
use std::sync::{Arc, Mutex};

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

  pub fn _is_integer(&self) -> bool {
      match self {
          RedisType::Integer(_) => true,
          _ => false
      }
  }

  pub fn _is_string(&self) -> bool {
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
  
  pub fn _is_nil(&self) -> bool {
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

pub type UnwrappedDictionary = Mutex<BTreeMap<String, RedisType>>;
pub type Dictionary = Arc<UnwrappedDictionary>;