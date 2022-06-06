///   ## json查询工具：
///   功能：
///   - [ ] 精确匹配 如 {"a": 10}  $: jsh a  结果为10
///   - [ ] 联级查询 如 {"a": {"b": 10}} a.b 结果为10
///   - [ ] 支持模糊查询 如 {"a": {"b": {"c": 10}}} a.*.c 结果为10
///   - [ ] 支持多级模糊查询 如 {"a": {"b": {"c": {"d": 10}}}} a.**.c 结果为10
///   - [ ] 数组查询 如 {"a": [10]} a[0] 结果为10
///
///

use clap::{Parser as ClapParser, ArgEnum};

use std::fs::{File};
use std::io::Read;
use simd_json::{self, Array, Value, ValueAccess, value::owned::Value as OwnedValue};

use pest::{Parser};
use pest::iterators::Pairs;

use anyhow::{Result, Context, anyhow};
use thiserror::Error;

use crate::SearchValue::{ArrayIndex, ObjectKey};

#[macro_use]
extern crate pest_derive;
extern crate core;

#[derive(Parser)]
#[grammar = "search.pest"]
struct SearchParser;

#[derive(ClapParser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// rule for search from json
    #[clap(short, long)]
    rule: String,

    /// source of json
    #[clap(short, long)]
    json: String,

    /// file or content
    #[clap(arg_enum, default_value_t = Source::File)]
    source: Source,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum Source {
    File,
    Content,
}

#[derive(Debug)]
enum SearchValue {
    ArrayIndex(usize),
    ObjectKey(String),
}

#[derive(Error, Debug)]
enum SearchError {
    #[error("item you searched is not exit")]
    NoExit,
    #[error("invalid type")]
    InvalidType,
}

struct SearchRules<'a>(Pairs<'a, Rule>);

impl<'a> Iterator for SearchRules<'a> {
    type Item = SearchValue;
    fn next(&mut self) -> Option<Self::Item> {
        let p = self.0.next()?;
        let rule = p.as_rule();
        if  rule != Rule::array_index && rule != Rule::object_key {
            return None;
        }
        let v = p.clone()
            .into_inner()
            .peekable()
            .peek()
            .unwrap()
            .as_str()
            .to_string();
        match p.as_rule() {
            Rule::array_index => Some(ArrayIndex(v.parse::<usize>().unwrap())),
            Rule::object_key => Some(ObjectKey(v)),
            _ => panic!("something wrong case by is neither array or object"),
        }
    }
}


fn parse_search(rule: &str) -> Result<Pairs<Rule>> {
    SearchParser::parse(Rule::search, rule).with_context(|| { format!("parse rule error: {}", rule) })
}

fn read_file(path: &str) -> Result<Vec<u8>> {
    let mut vec = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut vec)?;
    Ok(vec)
}


fn match_rule<'a>(rule: &SearchValue, value: &'a OwnedValue) -> Result<&'a OwnedValue> {
   match rule {
       SearchValue::ArrayIndex(i)  => {
           if value.is_array() {
               value.as_array().unwrap().get(i.clone()).with_context(|| {SearchError::NoExit})
           } else {
              Err(anyhow!(SearchError::InvalidType))
           }

       },
       SearchValue::ObjectKey(k) => {
           if value.is_object() {
               value.as_object().unwrap().get(k).with_context(|| {SearchError::NoExit})
           } else {
               Err(anyhow!(SearchError::InvalidType))
           }
       }
   }
}

fn search(mut bytes: &mut Vec<u8>, rules: &mut SearchRules) -> Result<OwnedValue> {
    let v = simd_json::to_owned_value(&mut bytes).unwrap();
    let mut _v = &v;
    while let Some(r) = rules.next() {
        _v = match_rule(&r, _v)?;
    }
    Ok(_v.clone())
}

fn main() {
    let args: Args = Args::parse();

   let bytes = match args.source {
        Source::File => {
            read_file(&args.json)
        },
        Source::Content => {
            Ok(args.json.as_bytes().to_vec())
        }
    };

    if bytes.is_err() {
        println!("{:?}", bytes.unwrap_err().to_string());
        return;
    }

    let rules = parse_search(&args.rule);

    if rules.is_err() {
        println!("{:?}", rules.unwrap_err().to_string());
        return;
    }
    let mut bytes = bytes.unwrap();
    let mut rules = SearchRules( rules.unwrap());

    match search(&mut bytes, &mut rules) {
        Ok(v) => println!("{}",v),
        Err(e) => println!("{}",e),
    }
}

#[cfg(test)]
mod test {
    use crate::{parse_search, search, SearchRules};

    #[test]
    fn test_search() {
        let mut b = br#"{ "a":{ "b":{ "c":10 } }, "a1":[{ "b1":{ "c1":"c1" } }, { "b2":{ "c2":"c2" } } ] }"#.to_vec();
        match parse_search(".a.b.c") {
            Ok(rule) => {
                let mut search_rules = SearchRules(rule);
                match search(&mut b, &mut search_rules) {
                    Ok(v) => println!(".a.b.c : {}", v),
                    Err(e) => panic!("{}", e),
                }
            },
            Err(e) => {
                panic!("{}", e);
            }
        }
        match parse_search(".a1[0]") {
            Ok(rule) => {
                let mut search_rules = SearchRules(rule);
                match search(&mut b, &mut search_rules) {
                    Ok(v) => println!(".a1[0] : {}", v),
                    Err(e) => panic!("{}", e),
                }
            },
            Err(e) => {
                panic!("{}", e);
            }
        }

        match parse_search(".a1[0].b1") {
            Ok(rule) => {
                let mut search_rules = SearchRules(rule);
                match search(&mut b, &mut search_rules) {
                    Ok(v) => println!(".a1[0].b1 : {}", v),
                    Err(e) => panic!("{}", e),
                }
            },
            Err(e) => {
                panic!("{}", e);
            }
        }

        match parse_search(".a1[0].b1.c1") {
            Ok(rule) => {
                let mut search_rules = SearchRules(rule);
                match search(&mut b, &mut search_rules) {
                    Ok(v) => println!(".a1[0].b1.c1 : {}", v),
                    Err(e) => panic!("{}", e),
                }
            },
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}
