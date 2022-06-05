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
use simd_json::ValueAccess;

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

fn main() {
    // let args = Args::parse();
    // println!("{:?}", args);

    let mut vec = Vec::new();
    File::open("./some.json").unwrap().read_to_end(&mut vec).unwrap();
    let v = simd_json::to_owned_value(&mut vec).unwrap();
     for (k, v) in v.as_object().unwrap() {
        println!("{:?}: {:?}", k, v);
     }
    let c = v.as_object().unwrap().get("a").as_object().unwrap().get("b").unwrap().get("c").unwrap().as_i64().unwrap();
    let pairs = parse_search(".key.b[10].是.abc[20][10]").unwrap();
    let search = SearchRules(pairs);
    for i in search {
       println!("{:?}", i);
    }
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

