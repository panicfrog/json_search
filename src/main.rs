use std::fmt::{Display, Formatter};
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
use simd_json;

use std::fs::{File};
use std::io::Read;
use simd_json::{Array, Value, ValueAccess};

use pest::{Parser};
use pest::iterators::Pairs;
use crate::SearchValue::{ArrayIndex, ObjectKey};

#[macro_use]
extern crate pest_derive;

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

fn parse_search(rule: &str) -> Result<Pairs<Rule>, pest::error::Error<Rule>> {
    SearchParser::parse(Rule::search, rule)
}

fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut vec = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut vec)?;
    Ok(vec)
}

#[derive(Debug)]
enum SearchValue {
    ArrayIndex(usize),
    ObjectKey(String),
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

enum SearchError {
    NoExit, InvalidType,
}

impl Display for SearchError  {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       match self {
           SearchError::NoExit => {
               write!(f, "item you searched is not exit")
           },
           SearchError::InvalidType => {
               write!(f, "invalid type")
           }
       }
    }
}

fn match_rule<'a>(rule: &SearchValue, value: &'a simd_json::value::owned::Value) -> Result<&'a simd_json::value::owned::Value, SearchError> {
   match rule {
       SearchValue::ArrayIndex(i)  => {
           if value.is_array() {
               value.as_array().unwrap().get(i.clone()).ok_or(SearchError::NoExit)
           } else {
              Err(SearchError::InvalidType)
           }

       },
       SearchValue::ObjectKey(k) => {
           if value.is_object() {
               value.as_object().unwrap().get(k).ok_or(SearchError::NoExit)
           } else {
               Err(SearchError::InvalidType)
           }
       }
   }
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

    let v = simd_json::to_owned_value(&mut bytes).unwrap();

    let mut _v = &v;

    while let Some(r) = rules.next() {
        match  match_rule(&r, _v) {
            Ok(r) => {
               _v = r;
            },
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
    println!("{}", _v);
}


